use html5ever::parse_document;
use html5ever::rcdom::{Handle, Node, NodeData, RcDom};
use html5ever::tendril::TendrilSink;
use reqwest::header;

pub use super::error::Error;
#[doc(inline)]
pub use super::error::Result;
use super::storage::MongoDB;

use serde::Serialize;

pub use super::common::{Hour, Room, Teacher};

/// config struct for planinfo
#[derive(Clone)]
pub struct Config {
    /// baseurl for planifo
    /// is a hidden setting, and default to `https://selbstlernportal.de/html/planinfo/planinfo_start.php`
    pub base_url: String,

    /// school id to identify the school to planinfo
    pub school_id: String,

    /// cookies for auth at planinfo
    pub cookies: String,

    /// delay between hits
    pub delay_hits: std::time::Duration,

    /// max times of miss
    pub max_misses: usize,

    /// where to start in the database
    /// this value is 1 less than the actual value
    pub start: usize,

    /// where to end
    pub end: usize,

    /// verbose level
    pub verbose: u8,
}

impl Config {
    /// create a new instance of Config
    pub fn new() -> Self {
        Self {
            base_url: String::from("https://selbstlernportal.de/html/planinfo/planinfo_start.php"),
            school_id: String::new(),
            cookies: String::new(),
            delay_hits: std::time::Duration::from_secs(20),
            max_misses: 5,
            start: 0,
            end: 0,
            verbose: 0,
        }
    }

    /// start parsing
    pub fn run(&self, db: MongoDB) -> Result<()> {
        let conf = self.clone();
        std::thread::spawn(move || {
            conf.run_int(db);
        });
        Ok(())
    }

    /// internal running function
    fn run_int(mut self, db: MongoDB) {
        loop {
            let planinfo = self.run_get(db.clone()).unwrap();
            std::thread::sleep(std::time::Duration::from_secs(86400)); // sleep for one day
        }
    }

    /// redownload page
    fn run_get(&self, db: MongoDB) -> Result<PlanInfo> {
        let mut planinfo = PlanInfo::new();
        let mut hits = self.max_misses;
        let mut dbidx: usize = self.start;

        // build client for http
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::COOKIE,
            header::HeaderValue::from_str(&self.cookies).unwrap(),
        );

        // get a client builder
        let client: reqwest::Client = reqwest::Client::builder()
            .default_headers(headers)
            .cookie_store(true)
            .build()?;

        while hits > 0 {
            dbidx += 1;
            if self.verbose >= 3 {
                println!("Debug3: PlanInfo: hit dbidx {}", dbidx);
            }
            let mut body: reqwest::Response = client
                .get(&format!(
                    "{}?ug={}&dbidx={}",
                    self.base_url, self.school_id, dbidx
                ))
                .send()?;

            if !body.status().is_success() {
                eprintln!("Error: PlanInfo: GET: {}", body.status());
                hits -= 1;
            } else {
                let body: String = body.text()?;
                let ret = planinfo.parse_str(&body, self.verbose);
                if let Err(err) = ret {
                    eprintln!("Error: Planinfo: pars: {}", err);
                    hits -= 1;
                } else if let Ok((table, kind)) = ret {
                    db.planinfo_write_table(&table, &kind);
                }
            }
            if dbidx == self.end {
                hits = 0;
            }
            // wait befor doing next hit
            std::thread::sleep(self.delay_hits);
        }

        Ok(planinfo)
    }
}

fn createTable() -> [[Hour; 12]; 5] {
    [
        [
            Hour::new(),
            Hour::new(),
            Hour::new(),
            Hour::new(),
            Hour::new(),
            Hour::new(),
            Hour::new(),
            Hour::new(),
            Hour::new(),
            Hour::new(),
            Hour::new(),
            Hour::new(),
        ],
        [
            Hour::new(),
            Hour::new(),
            Hour::new(),
            Hour::new(),
            Hour::new(),
            Hour::new(),
            Hour::new(),
            Hour::new(),
            Hour::new(),
            Hour::new(),
            Hour::new(),
            Hour::new(),
        ],
        [
            Hour::new(),
            Hour::new(),
            Hour::new(),
            Hour::new(),
            Hour::new(),
            Hour::new(),
            Hour::new(),
            Hour::new(),
            Hour::new(),
            Hour::new(),
            Hour::new(),
            Hour::new(),
        ],
        [
            Hour::new(),
            Hour::new(),
            Hour::new(),
            Hour::new(),
            Hour::new(),
            Hour::new(),
            Hour::new(),
            Hour::new(),
            Hour::new(),
            Hour::new(),
            Hour::new(),
            Hour::new(),
        ],
        [
            Hour::new(),
            Hour::new(),
            Hour::new(),
            Hour::new(),
            Hour::new(),
            Hour::new(),
            Hour::new(),
            Hour::new(),
            Hour::new(),
            Hour::new(),
            Hour::new(),
            Hour::new(),
        ],
    ]
}

#[derive(Debug, Clone, Serialize)]
pub struct Table {
    pub name: String,
    pub table_a: [[Hour; 12]; 5],
    pub table_b: [[Hour; 12]; 5],
    //pub date: chrono::DateTime<chrono::Utc>,
}

impl Table {
    pub fn new() -> Self {
        Default::default()
    }
}

impl Default for Table {
    fn default() -> Self {
        Self {
            name: String::new(),
            table_a: createTable(),
            table_b: createTable(),
            //date: chrono::Utc::now(),
        }
    }
}

#[derive(Debug)]
pub struct PlanInfo {
    /// teachers in planinfo
    pub teachers: Vec<Table>,

    /// tables for rooms
    pub rooms: Vec<Table>,

    /// tables for students
    pub students: Vec<Table>,
}

impl PlanInfo {
    /// create new empty PlanInfo
    pub fn new() -> Self {
        Self {
            teachers: Vec::new(),
            rooms: Vec::new(),
            students: Vec::new(),
        }
    }

    /// parse string into PlanInfo
    pub fn parse_str(&mut self, html: &str, verbose: u8) -> Result<(Table, String)> {
        let html = html.replace("&nbsp;", " ");
        let html = html.trim();
        let dom = parse_document(RcDom::default(), Default::default())
            .from_utf8()
            .read_from(&mut html.as_bytes())?;
        self.parse_dom(&dom.document, verbose)
    }

    /// parse RcDom into PlanInfo
    pub fn parse_dom(&mut self, handle: &Handle, verbose: u8) -> Result<(Table, String)> {
        let node: &Node = handle;
        let node: &Node = &node.children.borrow()[1];
        if node.children.borrow().len() < 2 {
            return Err(Error::new_field_not_exists(
                "planinfo html head|body".to_string(),
            ));
        }
        if !self.check_title(&node.children.borrow()[0]) {
            return Err(Error::new_field_not_exists("planinfo auth".to_string()));
        }

        let node: &Node = &node.children.borrow()[2];
        for v in node.children.borrow().iter() {
            let v: &Node = v;
            if let NodeData::Element {
                ref name,
                ref attrs,
                ..
            } = v.data
            {
                let name: &html5ever::QualName = name;
                let attrs: &Vec<html5ever::Attribute> = &attrs.borrow();
                if name.local.to_string() == "div" {
                    for attr in attrs.iter() {
                        let attr: &html5ever::Attribute = attr;
                        if attr.name.local.to_string() == "class"
                            && attr.value.to_string() == "plan"
                        {
                            return self.parse_dom_div(v, verbose);
                        }
                    }
                }
            }
        }
        Err(Error::new_field_not_exists(
            "not found for parse_dom (planinfo)".to_string(),
        ))
    }

    /// parse PlanInfo plan div content
    fn parse_dom_div(&mut self, node: &Node, verbose: u8) -> Result<(Table, String)> {
        let mut kind = 0;
        let node: &Node = node;
        for v in node.children.borrow().iter() {
            let v: &Node = v;
            if let NodeData::Element { ref name, .. } = v.data {
                let name: &html5ever::QualName = name;
                if name.local.to_string() == "table" {
                    let mut A = true;
                    let mut first_run = true;
                    let mut entryName = String::new();
                    let mut courseString = String::new();
                    for v in v.children.borrow().iter() {
                        let v: &Node = v;
                        if let NodeData::Element { ref name, .. } = v.data {
                            let name: &html5ever::QualName = name;
                            if name.local.to_string() == "thead" {
                                for v in v.children.borrow().iter() {
                                    let v: &Node = v;
                                    if let NodeData::Element { ref name, .. } = v.data {
                                        let name: &html5ever::QualName = name;
                                        if name.local.to_string() == "tr" {
                                            for v in v.children.borrow().iter() {
                                                let v: &Node = v;
                                                if let NodeData::Element {
                                                    ref name,
                                                    ref attrs,
                                                    ..
                                                } = v.data
                                                {
                                                    let name: &html5ever::QualName = name;
                                                    let attrs: &Vec<html5ever::Attribute> =
                                                        &attrs.borrow();
                                                    for attr in attrs.iter() {
                                                        let attr: &html5ever::Attribute = attr;
                                                        if attr.name.local.to_string() == "class"
                                                            && attr.value.to_string() == "titel"
                                                        {
                                                            for v in v.children.borrow().iter() {
                                                                let v: &Node = v;
                                                                if let NodeData::Text {
                                                                    ref contents,
                                                                } = v.data
                                                                {
                                                                    let contents: &str =
                                                                        &contents.borrow();
                                                                    let contents = contents.trim();
                                                                    if contents.starts_with("A") {
                                                                        A = true;
                                                                    } else if contents
                                                                        .starts_with("B")
                                                                    {
                                                                        A = false;
                                                                    }
                                                                    if contents
                                                                        .contains("Lehrer/in")
                                                                    {
                                                                        let names: Vec<&str> =
                                                                            contents
                                                                                .split(" ")
                                                                                .collect();
                                                                        if let Some(names) =
                                                                            names.last()
                                                                        {
                                                                            entryName =
                                                                                names.to_string();
                                                                            if first_run {
                                                                                let mut table =
                                                                                    Table::new();
                                                                                table.name =
                                                                                    entryName
                                                                                        .clone();
                                                                                self.teachers
                                                                                    .push(table);
                                                                            }
                                                                        }
                                                                        kind = 0;
                                                                    } else if contents
                                                                        .contains("Raum")
                                                                    {
                                                                        kind = 1;
                                                                        let names: Vec<&str> =
                                                                            contents
                                                                                .split(" ")
                                                                                .collect();
                                                                        if let Some(names) =
                                                                            names.last()
                                                                        {
                                                                            entryName =
                                                                                names.to_string();
                                                                            if first_run {
                                                                                let mut table =
                                                                                    Table::new();
                                                                                table.name =
                                                                                    entryName
                                                                                        .clone();
                                                                                self.rooms
                                                                                    .push(table);
                                                                            }
                                                                        }
                                                                    } else if contents
                                                                        .contains("Schüler/in")
                                                                    {
                                                                        kind = 2;
                                                                        if first_run {
                                                                            let mut table =
                                                                                Table::new();
                                                                            table.name =
                                                                                entryName.clone();
                                                                            self.students
                                                                                .push(table);
                                                                        }
                                                                    } else if contents.ends_with(
                                                                        "-Woche-Stundenplan von",
                                                                    ) {
                                                                        return Err(Error::new_field_not_exists(("PlanInfo empty".to_string())));
                                                                    } else if contents
                                                                        .starts_with("(")
                                                                        && contents.ends_with(")")
                                                                    {
                                                                        let contents: &str =
                                                                            contents.trim();
                                                                        let contents: &str =
                                                                            contents
                                                                                .trim_matches('(')
                                                                                .trim_matches(')');
                                                                        courseString =
                                                                            contents.to_string();
                                                                    } else if contents
                                                                        .contains("Klasse")
                                                                    {
                                                                        // FIXME: implement class
                                                                        /* let names: Vec<&str> =
                                                                            contents
                                                                                .split(" ")
                                                                                .collect();
                                                                        if let Some(names) =
                                                                            names.last()
                                                                        {
                                                                            entryName =
                                                                                names.to_string();
                                                                            if first_run {
                                                                                let mut table = ClassTable::new();
                                                                                table.name =
                                                                                    entryName;
                                                                                self.classes
                                                                                    .push(table);
                                                                            }
                                                                        } */
                                                                        kind = 3;
                                                                    } else {
                                                                        eprintln!("Error: PlanInfo: parse_dom_div: unknown kind: {{{}}}", contents);
                                                                        return Err(Error::new_field_not_exists("PlanInfo header kind".to_string()));
                                                                    }
                                                                } else if let NodeData::Element {
                                                                    ref name,
                                                                    ..
                                                                } = v.data
                                                                {
                                                                    let name: &html5ever::QualName =
                                                                        &name;
                                                                    if name.local.to_string()
                                                                        == "span"
                                                                    {
                                                                        for v in v
                                                                            .children
                                                                            .borrow()
                                                                            .iter()
                                                                        {
                                                                            let v: &Node = v;
                                                                            if let NodeData::Element { ref name, ref attrs, .. } = v.data {
                                                                                let name: &html5ever::QualName = &name;
                                                                                let attrs: &Vec<html5ever::Attribute> = &attrs.borrow();
                                                                                for attr in attrs.iter() {
                                                                                    let attr: &html5ever::Attribute = attr;
                                                                                    if attr.name.local.to_string() == "title" {
                                                                                        if kind == 2 {
                                                                                            let name: String = attr.value.to_string();
                                                                                            let name = name.trim_start_matches("Schüler/in ");
                                                                                            let name: Vec<&str> = name.split("(").collect();
                                                                                            if let Some(name) = name.first() {
                                                                                                entryName = name.to_string();
                                                                                            }
                                                                                            if let Some(course) = name.last() {
                                                                                                let course: &str = course.trim();
                                                                                                let course: &str = course.trim_matches(')');
                                                                                                if verbose >= 1 {
                                                                                                    eprintln!("Error1: PlanInfo: not implemented: parse course from header: {{{}}}", course);
                                                                                                }
                                                                                            }
                                                                                        }
                                                                                    }
                                                                                }
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                            first_run = false;
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            } else if name.local.to_string() == "tbody" {
                                let mut x = 0;
                                let mut y = 0;
                                for v in v.children.borrow().iter() {
                                    let v: &Node = v;
                                    if let NodeData::Element { ref name, .. } = v.data {
                                        let name: &html5ever::QualName = name;
                                        if name.local.to_string() == "tr" {
                                            x = 0;
                                            for v in v.children.borrow().iter() {
                                                let v: &Node = v;
                                                if let NodeData::Element { ref name, .. } = v.data {
                                                    let name: &html5ever::QualName = name;
                                                    if name.local.to_string() == "td" {
                                                        for v in v.children.borrow().iter() {
                                                            if let NodeData::Text { ref contents } =
                                                                v.data
                                                            {
                                                                let contents: &str =
                                                                    &contents.borrow();
                                                                let contents = contents.trim();
                                                                if kind == 0 {
                                                                    // teachers
                                                                    if let Some(table) =
                                                                        self.teachers.last_mut()
                                                                    {
                                                                        let table: &mut Table =
                                                                            table;
                                                                        if A {
                                                                            table.table_a[x][y]
                                                                                .parse_planinfo_teacher(
                                                                                    contents, &entryName, verbose
                                                                                );
                                                                        } else {
                                                                            table.table_b[x][y]
                                                                                .parse_planinfo_teacher(
                                                                                    contents, &entryName, verbose
                                                                                );
                                                                        }
                                                                    }
                                                                } else if kind == 1 {
                                                                    // Room
                                                                    if let Some(table) =
                                                                        self.rooms.last_mut()
                                                                    {
                                                                        let table: &mut Table =
                                                                            table;
                                                                        if A {
                                                                            table.table_a[x][y].parse_planinfo_room(contents, &entryName, verbose);
                                                                        } else {
                                                                            table.table_b[x][y].parse_planinfo_room(contents, &entryName, verbose);
                                                                        }
                                                                    }
                                                                } else if kind == 2 {
                                                                    // student
                                                                    if let Some(table) =
                                                                        self.students.last_mut()
                                                                    {
                                                                        let table: &mut Table =
                                                                            table;
                                                                        table.name =
                                                                            entryName.clone();
                                                                        if A {
                                                                            table.table_a[x][y].parse_planinfo_student(contents, &courseString, verbose);
                                                                        } else {
                                                                            table.table_b[x][y].parse_planinfo_student(contents, &courseString, verbose);
                                                                        }
                                                                    }
                                                                } else if kind == 3 {
                                                                    // Class
                                                                    /*if let Some(table) = self.classes.last_mut() {
                                                                        let table: &mut ClassTable = table;
                                                                        if A {
                                                                            table.table_a[x][y].parse_planinfo(contents)
                                                                        }
                                                                    }*/
                                                                    if verbose >= 1 {
                                                                        eprintln!("Error1: PlanInfo: parser class parser not implemented");
                                                                    }
                                                                } else {
                                                                    return Err(
                                                                        Error::new_field_not_exists(
                                                                            format!(
                                                                                "kind: {}",
                                                                                kind
                                                                            ),
                                                                        ),
                                                                    );
                                                                }
                                                            }
                                                        }
                                                        x += 1;
                                                    }
                                                }
                                            }
                                            y += 1;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        let mut table: Table = Table::default();
        let mut kindString = String::from("none");
        if kind == 0 {
            //teachers
            if let Some(teacher) = self.teachers.last() {
                table = teacher.clone();
                kindString = "teachers".to_string();
            }
        } else if kind == 1 {
            // room
            if let Some(room) = self.rooms.last() {
                table = room.clone();
                kindString = "room".to_string();
            }
        } else if kind == 2 {
            // student
            if let Some(student) = self.students.last() {
                table = student.clone();
                kindString = "students".to_string();
            }
        }
        Ok((table.clone(), kindString))
    }

    /// test if auth was successfully
    fn check_title(&self, handle: &Handle) -> bool {
        let node: &Node = handle;
        for v in node.children.borrow().iter() {
            let v: &Node = v;
            if let NodeData::Element { ref name, .. } = v.data {
                let name: &html5ever::QualName = name;
                if name.local.to_string() == "title" {
                    let node: &Node = &v.children.borrow()[0];
                    if let NodeData::Text { ref contents } = node.data {
                        let contents = escape_default(&contents.borrow());
                        if contents.contains("Anzeige") {
                            return true;
                        } else {
                            return false;
                        }
                    }
                }
            }
        }

        return false;
    }
}

// FIXME: Copy of str::escape_default from std, which is currently unstable
pub fn escape_default(s: &str) -> String {
    s.chars().flat_map(|c| c.escape_default()).collect()
}
