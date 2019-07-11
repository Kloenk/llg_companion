use html5ever::parse_document;
use html5ever::rcdom::{Handle, Node, NodeData, RcDom};
use html5ever::tendril::TendrilSink;
use reqwest::header;

pub use super::error::Error;
#[doc(inline)]
pub use super::error::Result;

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
            verbose: 0,
        }
    }

    /// start parsing
    pub fn run(&self) -> Result<()> {
        let conf = self.clone();
        std::thread::spawn(move || {
            conf.run_int();
        });
        Ok(())
    }

    /// internal running function
    fn run_int(mut self) {
        loop {
            self.run_get().unwrap();
            std::thread::sleep(std::time::Duration::from_secs(86400)); // sleep for one day
        }
    }

    /// redownload page
    fn run_get(&self) -> Result<PlanInfo> {
        let mut planinfo = PlanInfo::new();
        let mut hits = self.max_misses;
        let mut dbidx: usize = 0;

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
            let mut body: reqwest::Response = client
                .get(&format!(
                    "{}?ug={}&dbidx={}",
                    self.base_url, self.school_id, dbidx
                ))
                .send()?;

            if !body.status().is_success() {
                eprintln!("Error: PlanInfo: GET: {}", body.status());
                hits -= 1;
                continue;
            }
            let body: String = body.text()?;
            if let Err(err) = planinfo.parse_str(&body) {
                eprintln!("Error: Planinfo: pars: {}", err);
                hits -= 1;
            }

            // FIXME: remove
            hits = 0;
            // wait befor doing next hit
            std::thread::sleep(self.delay_hits);
        }
        println!("Planinfo: {:?}", planinfo);

        Ok(planinfo)
    }
}

#[derive(Debug)]
pub struct TeacherTable {
    pub name: String,
    pub table_a: [[Hour; 12]; 5],
    pub table_b: [[Hour; 12]; 5],
}

impl TeacherTable {
    pub fn new() -> Self {
        Default::default()
    }
}

impl Default for TeacherTable {
    fn default() -> Self {
        Self {
            name: String::new(),
            table_a: [
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
            ],
            table_b: [
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
            ],
        }
    }
}

#[derive(Debug)]
pub struct PlanInfo {
    /// teachers in planinfo
    pub teachers: Vec<TeacherTable>,
}

impl PlanInfo {
    /// create new empty PlanInfo
    pub fn new() -> Self {
        Self {
            teachers: Vec::new(),
        }
    }

    /// parse string into PlanInfo
    pub fn parse_str(&mut self, html: &str) -> Result<()> {
        let html = html.replace("&nbsp;", " ");
        let html = html.trim();
        let dom = parse_document(RcDom::default(), Default::default())
            .from_utf8()
            .read_from(&mut html.as_bytes())?;
        self.parse_dom(&dom.document)
    }

    /// parse RcDom into PlanInfo
    pub fn parse_dom(&mut self, handle: &Handle) -> Result<()> {
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
                            return self.parse_dom_div(v);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// parse PlanInfo plan div content
    fn parse_dom_div(&mut self, node: &Node) -> Result<()> {
        let node: &Node = node;
        for v in node.children.borrow().iter() {
            let v: &Node = v;
            if let NodeData::Element { ref name, .. } = v.data {
                let name: &html5ever::QualName = name;
                if name.local.to_string() == "table" {
                    let mut A = true;
                    let mut kind = 0;
                    let mut first_run = true;
                    let mut entryName = String::new();
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
                                                                                let mut table = TeacherTable::new();
                                                                                table.name =
                                                                                    entryName;
                                                                                self.teachers
                                                                                    .push(table);
                                                                            }
                                                                        }
                                                                        kind = 0;
                                                                    } else if contents
                                                                        .contains("Raum")
                                                                    {
                                                                        kind = 1;
                                                                    } else if contents
                                                                        .contains("Schüler/in")
                                                                    {
                                                                        kind = 2;
                                                                    } else if contents.ends_with(
                                                                        "-Woche-Stundenplan von",
                                                                    ) {
                                                                        return Err(Error::new_field_not_exists(("PlanInfo empty".to_string())));
                                                                    } else if contents
                                                                        .starts_with("(")
                                                                        && contents.ends_with(")")
                                                                    {
                                                                        // name attribut, do nothing
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
                                                                                        println!("name-orig: {}", attr.value.to_string());
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
                                                                                                eprintln!("Error: PlanInfo: not implemented: parse course from header: {{{}}}", course);
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
                                                                    if let Some(table) =
                                                                        self.teachers.last_mut()
                                                                    {
                                                                        let table: &mut TeacherTable = table;
                                                                        if A {
                                                                            table.table_a[y][x]
                                                                                .parse_planinfo(
                                                                                    contents,
                                                                                );
                                                                        } else {
                                                                            table.table_b[y][x]
                                                                                .parse_planinfo(
                                                                                    contents,
                                                                                );
                                                                        }
                                                                    }
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
        Ok(())
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
