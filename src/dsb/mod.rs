use super::error::Error;
use chrono::prelude::*;
use serde_json::json;
use std::clone::Clone;
use std::io::prelude::*;
use std::thread;

use html5ever::parse_document;
use html5ever::rcdom::{Handle, Node, NodeData, RcDom};
use html5ever::tendril::TendrilSink;

#[doc(inline)]
pub use super::error::Result;

/// config struct for dsb informations
#[derive(Clone)]
pub struct Config {
    /// userid to use
    pub user_id: String,

    /// password for dsb
    pub password: String,

    /// cookie for dsb authentification
    pub cookie: String,

    pub verbose: u8,

    /// url for dsb
    /// only use when you use another host for dsb
    /// defaults to `https://www.dsbmobile.de/JsonHandlerWeb.ashx/GetData`
    pub url: String,
}

impl Config {
    /// create a new instance of Config
    pub fn new() -> Self {
        Self {
            user_id: String::new(),
            password: String::new(),
            cookie: String::new(),
            verbose: 0,
            url: String::from("https://www.dsbmobile.de/JsonHandlerWeb.ashx/GetData"),
        }
    }

    /// start parser
    pub fn run(&self) -> Result<()> {
        let conf = self.clone();
        thread::spawn(move || {
            conf.run_int();
        });
        Ok(())
    }

    /// internal run function holding the mail loop of the thread
    fn run_int(self) {
        self.get();
        loop {}
    }

    /// get dsb content
    fn get(&self) -> Result<()> {
        let data = self.gen_request_payload()?;

        let client = reqwest::Client::new();
        let mut dsb = client.post(&self.url)
            .header("Cookie", self.cookie.clone())
            .header("User-Agent", String::from("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/62.0.3202.94 Safari/537.36"))
            .header("Bundle_ID", "de.heinekingmedia.inhouse.dsbmobile.web")
            .header("Content-Type", "application/json;charset=UTF-8")
            .header("X-Requested-With", "XMLHttpRequest")
            .header("Referer", "https://www.dsbmobile.de/default.aspx")
            .body(data)
            .send()?;

        let body = dsb.text().unwrap();

        let url = self.decode_dsb_payload(&body).unwrap();
        let mut html = client.get(&url)
            .header("Cookie", self.cookie.clone())
            .header("User-Agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/62.0.3202.94 Safari/537.36")
            .header("Bundle_ID", "de.heinekingmedia.inhouse.dsbmobile.web")
            .header("Accept-Encoding", "latin1")
            .send()?;
        if html.status().as_u16() != 200 {
            return Err(Error::new_field_not_exists("not 200 foo".to_string()));
        }
        for (h, v) in html.headers().iter() {
            if self.verbose >= 3 {
                println!("Debug3: DSB: header: {}: {:?}", h, v);
            }
        }

        let html = html.text().unwrap();
        self.parse(&html);

        Ok(()) // change
    }

    /// create request payload
    fn gen_request_payload(&self) -> Result<String> {
        let now: DateTime<Utc> = Utc::now();
        let now = now.to_rfc3339();
        let data = json!({
            "UserId": self.user_id.clone(),
            "UserPw": self.password.clone(),
            "Abos": [],
            "AppVersion": "2.3",
            "Language": "de",
            "OsVersion": "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/62.0.3202.94 Safari/537.36",
            "AppId": "",
            "Device": "WebApp",
            "PushId": "",
            "BundleId": "de.heinekingmedia.inhous.dsbmobile.web",
            "Date": now,
            "LastUpdate": now,
        }).to_string();

        let mut e = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        e.write_all(data.as_bytes())?;
        let data = e.finish()?;
        let data = base64::encode(&data);

        let data = json!({
            "req": {
                "Data": data,
                "DataType": 1,
            }
        });

        Ok(data.to_string())
    }

    /// decode dsb payload
    fn decode_dsb_payload(&self, payload: &str) -> Result<String> {
        let mut url = String::new();
        let json: serde_json::Value = serde_json::from_str(payload)?;

        // check that d exists and is a string
        if json.get("d") == None {
            return Err(super::error::Error::new_field_not_exists("d".to_string()));
        }
        let d = json.get("d").unwrap();

        if !d.is_string() {
            return Err(super::error::Error::new_field_not_exists("d".to_string()));
        }
        let d = d.as_str().unwrap();

        let data = base64::decode(d)?;
        let mut e = flate2::write::GzDecoder::new(Vec::new());
        e.write_all(&data)?;
        let data = e.finish()?;

        if self.verbose >= 5 {
            println!("Debug5: DSB: Json: {}", String::from_utf8_lossy(&data));
        }
        let json: serde_json::Value = serde_json::from_str(&String::from_utf8_lossy(&data))?;

        if json.get("ResultMenuItems") == None {
            return Err(super::error::Error::new_field_not_exists(
                "data.ResultMenuItems".to_string(),
            ));
        }
        let json = json.get("ResultMenuItems").unwrap();

        if !json.is_array() {
            return Err(super::error::Error::new_field_not_exists(
                "data.ResultMenuItems".to_string(),
            ));
        }
        let json = json.as_array().unwrap();

        let mut x = false;
        let mut index = 0;
        while !x {
            if json.get(index) == None {
                return Err(super::error::Error::new_field_not_exists(format!(
                    "data.ResultMenuItems.{}",
                    index
                )));
            }
            let json = json.get(index).unwrap();
            if json.get("Title") == None {
                return Err(super::error::Error::new_field_not_exists(format!(
                    "data.ResultMenuItems.{}.Title",
                    index
                )));
            }
            let title = json.get("Title").unwrap();
            if let Some(title) = title.as_str() {
                if title == "Inhalte" {
                    x = true;

                    if json.get("Childs") == None {
                        return Err(super::error::Error::new_field_not_exists(format!(
                            "data.ResultMenuItems.{}.Childs",
                            index
                        )));
                    }
                    let json = json.get("Childs").unwrap();
                    if !json.is_array() {
                        return Err(super::error::Error::new_field_not_exists(format!(
                            "data.ResultMenuItems.{}.Childs",
                            index
                        )));
                    }
                    let json = json.as_array().unwrap();
                    let mut y = false;
                    let mut indexy = 0;
                    for v in json {
                        indexy += 1;
                        let title = v.get("Title");
                        if title == None {
                            return Err(super::error::Error::new_field_not_exists(format!(
                                "data.ResultMenuItems.{}.Childs.{}.Title",
                                index, indexy
                            )));
                        }
                        let title = title.unwrap();
                        if !title.is_string() {
                            return Err(super::error::Error::new_field_not_exists(format!(
                                "data.ResultMenuItems.{}.Childs.{}.Title",
                                index, indexy
                            )));
                        }
                        let title = title.as_str().unwrap().to_string();
                        if title == "Pläne" {
                            let v = v.get("Root");
                            if v == None {
                                return Err(super::error::Error::new_field_not_exists(format!(
                                    "data.ResultMenuItems.{}.Childs.{}.Root",
                                    index, indexy
                                )));
                            }
                            let v = v.unwrap();
                            let v = v.get("Childs");
                            if v == None {
                                return Err(super::error::Error::new_field_not_exists(format!(
                                    "data.ResultMenuItems.{}.Childs.{}.Root.Childs",
                                    index, indexy
                                )));
                            }
                            let v = v.unwrap();
                            if !v.is_array() {
                                return Err(super::error::Error::new_field_not_exists(format!(
                                    "data.ResultMenuItems.{}.Childs.{}.Root.Childs",
                                    index, indexy
                                )));
                            }
                            let v = v.as_array().unwrap();
                            let mut indexz = 0;
                            for v in v {
                                indexz += 1;
                                let title = v.get("Title");
                                if title == None {
                                    return Err(
                                        super::error::Error::new_field_not_exists(
                                            format!(
                                                "data.ResultMenuItems.{}.Childs.{}.Root.Childs.{}.Title",
                                                index, indexy, indexz
                                            )
                                        )
                                    );
                                }
                                let title = title.unwrap();
                                let title = title.as_str();
                                if title == None {
                                    return Err(
                                        super::error::Error::new_field_not_exists(
                                            format!(
                                                "data.ResultMenuItems.{}.Childs.{}.Root.Childs.{}.Title",
                                                index, indexy, indexz
                                            )
                                        )
                                    );
                                }
                                if title.unwrap() == "DSBSchueler" {
                                    if let Some(childs) = v.get("Childs") {
                                        if let Some(nul) = childs.get(0) {
                                            if let Some(detail) = nul.get("Detail") {
                                                if let Some(detail) = detail.as_str() {
                                                    url = detail.to_string();
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            } else {
                return Err(super::error::Error::new_field_not_exists(format!(
                    "data.ResultMenuItems.{}",
                    index
                )));
            }
        }
        Ok(url)
    }

    /// parse dsb content
    fn parse(&self, html: &str) -> Result<Vec<DSB>> {
        //let mut html = html.to_string();
        let html = html.replace("&nbsp;", " ");
        let dom = parse_document(RcDom::default(), Default::default())
            .from_utf8()
            .read_from(&mut html.as_bytes())
            .unwrap();
        let dsb = self.parse_dom(&dom.document).unwrap();
        Ok(dsb)
    }

    fn parse_dom(&self, handle: &Handle) -> Result<Vec<DSB>> {
        let mut dsb_return: Vec<DSB> = Vec::new();
        let node: &Node = handle;
        let nodeVec = node.children.borrow();
        let node: &Node = &nodeVec[0];

        for v in node.children.borrow().iter() {
            let v: &Node = v;
            if let NodeData::Element { ref name, .. } = v.data {
                let name: &html5ever::QualName = name;
                if name.local.to_string() == "body" {
                    let mut found_mod_head = false;
                    for w in v.children.borrow().iter() {
                        let w: &Node = w;
                        if let NodeData::Element {
                            ref name,
                            ref attrs,
                            ..
                        } = w.data
                        {
                            let name: &html5ever::QualName = name;
                            let attrs: &Vec<html5ever::Attribute> = &attrs.borrow();
                            if name.local.to_string() == "table" {
                                for attr in attrs.iter() {
                                    if attr.name.local.to_string() == "class"
                                        && attr.value.to_string() == "mon_head"
                                    {
                                        dsb_return.push(DSB::new_mon_head(w));

                                        found_mod_head = true;
                                    }
                                }
                            } else if name.local.to_string() == "center" && found_mod_head {
                                if let Some(dsb) = dsb_return.last_mut() {
                                    let dsb: &mut DSB = dsb;
                                    self.parse_center(w, dsb);
                                }
                                found_mod_head = false;
                            }
                        }
                    }
                }
            }
        }

        Ok(dsb_return)
    }

    /// parse dsb center
    fn parse_center(&self, node: &Node, dsb: &mut DSB) {
        let mon_title: &Node = &node.children.borrow()[1];
        let mon_title: &Node = &mon_title.children.borrow()[0];
        if let NodeData::Text { ref contents } = mon_title.data {
            let contents = escape_default(&contents.borrow());
            dsb.parse_mon_title(&contents);
        }

        let info: &Node = &node.children.borrow()[3];
        dsb.parse_info_table(info);

        let rows: &Node = &node.children.borrow()[5];
        let rows: &Node = &rows.children.borrow()[1];
        let rows: &Node = &rows.children.borrow()[1];
        for v in rows.children.borrow().iter() {
            let v: &Node = v;
            if v.children.borrow().len() == 8 {
                let class: &Node = &v.children.borrow()[0];
                let class: &Node = &class.children.borrow()[0];
                let mut new = false;
                if let NodeData::Text { ref contents } = class.data {
                    let contents = escape_default(&contents.borrow());
                    if contents.contains("Klasse") {
                        continue;
                    }
                    if contents != " " {
                        dsb.entries.push(Entry::new_from_str(&contents));
                        new = true;
                    }
                }
                let mut entrie: &mut Entry = dsb.entries.last_mut().unwrap();
                if new {
                    let hour: &Node = &v.children.borrow()[1];
                    let hour: &Node = &hour.children.borrow()[0];
                    let hour: &Node = &hour.children.borrow()[0];
                    if let NodeData::Text { ref contents } = hour.data {
                        let contents = escape_default(&contents.borrow());
                        if contents.contains("-") {
                            let contents: Vec<&str> = contents.split(" - ").collect();
                            entrie.time.from = contents[0].parse().unwrap_or(0);
                            entrie.time.to = contents[1].parse().unwrap_or(0);
                        } else {
                            let contents = contents.chars().next().unwrap();
                            let t: u8 = (contents as u32 - '0' as u32) as u8;
                            entrie.time.from = t;
                            entrie.time.to = t;
                        }
                    }

                    let substitute: &Node = &v.children.borrow()[2];
                    let substitute: &Node = &substitute.children.borrow()[0];
                    let substitute: &Node = &substitute.children.borrow()[0];
                    if let NodeData::Text { ref contents } = substitute.data {
                        let contents = escape_default(&contents.borrow());
                        let mut teacher: Teacher = Teacher::new();
                        teacher.name = contents.trim().trim_matches('-').to_string();
                        entrie.old_teacher = teacher;
                    }
                    let course: &Node = &v.children.borrow()[3];
                    let course: &Node = &course.children.borrow()[0];
                    if let NodeData::Text { ref contents } = course.data {
                        let contents = escape_default(&contents.borrow());
                        let mut course: Course = Course::new();
                        course.name = contents.trim().trim_matches('-').to_string();
                        entrie.course = course;
                    }
                    let course: &Node = &v.children.borrow()[4];
                    let course: &Node = &course.children.borrow()[0];
                    if let NodeData::Text { ref contents } = course.data {
                        let contents = escape_default(&contents.borrow())
                            .trim()
                            .trim_matches('-')
                            .to_string();
                        if contents.is_empty() {
                            entrie.old_course = None;
                        } else {
                            let mut course: Course = Course::new();
                            course.name = contents;
                            entrie.old_course = Some(course);
                        }
                    }
                    let message: &Node = &v.children.borrow()[5];
                    let message: &Node = &message.children.borrow()[0];
                    if let NodeData::Text { ref contents } = message.data {
                        entrie.message = escape_default(&contents.borrow())
                            .trim()
                            .trim_matches('-')
                            .to_string();
                    }
                    let kind: &Node = &v.children.borrow()[6];
                    let kind: &Node = &kind.children.borrow()[0];
                    if let NodeData::Text { ref contents } = kind.data {
                        let kind = escape_default(&contents.borrow())
                            .trim()
                            .trim_matches('-')
                            .to_string();
                        entrie.kind = EntryKind::parse_from_str(&kind);
                    }
                    let room: &Node = &v.children.borrow()[7];
                    let room: &Node = &room.children.borrow()[0];
                    let room: &Node = &room.children.borrow()[0];
                    if let NodeData::Text { ref contents } = room.data {
                        let room: String = escape_default(&contents.borrow())
                            .trim()
                            .trim_matches('-')
                            .to_string();
                        if !room.is_empty() {
                            entrie.room = Some(Room::from_str(&room));
                        }
                    }
                } else {
                    let message: &Node = &v.children.borrow()[5];
                    let message: &Node = &message.children.borrow()[0];
                    if let NodeData::Text { ref contents } = message.data {
                        let message: String = escape_default(&contents.borrow())
                            .trim()
                            .trim_matches('-')
                            .to_string();
                        entrie.message = entrie.message.clone() + &message;
                    }
                }
            } else if v.children.borrow().len() == 1 {
                // header
            }
        }
    }
}

/// enum for A and B week
#[derive(Debug)]
pub enum Week {
    A,
    B,
    NoWeek(char),
}

impl Week {
    /// parse char to week
    fn parse(input: char) -> Self {
        match input {
            'A' => return Week::A,
            'B' => return Week::B,
            _ => return Week::NoWeek(input),
        }
    }
}

/// enum for buildings
#[derive(Debug)]
pub enum Building {
    A,
    B,
    C,
    D,
    E,
    Other,
}

#[derive(Debug)]
pub struct Teacher {
    pub name: String,
}

impl Teacher {
    pub fn new() -> Self {
        Self {
            name: String::new(),
        }
    }
}

#[derive(Debug)]
pub struct Room {
    pub building: Building,
    pub room: i16,
}

impl Room {
    pub fn new() -> Self {
        Self {
            building: Building::Other,
            room: 0,
        }
    }
    pub fn from_str(input: &str) -> Self {
        let mut room = Self::new();
        if input.len() != 4 {
            eprintln!("Error: DSB: Room: from_str: {} not len 4", input);
            return room;
        }
        if input.to_lowercase().trim().starts_with("a") {
            room.building = Building::A;
        } else if input.to_lowercase().trim().starts_with("b") {
            room.building = Building::B;
        } else if input.to_lowercase().trim().starts_with("c") {
            room.building = Building::C;
        } else if input.to_lowercase().trim().starts_with("d") {
            room.building = Building::D;
        } else if input.to_lowercase().trim().starts_with("e") {
            room.building = Building::E;
        }
        let input: &str = input.split_at(1).1;
        room.room = input.parse().unwrap_or(0);

        room
    }
}

#[derive(Debug)]
pub struct Class {
    pub name: String,
}

#[derive(Debug)]
pub struct DSB {
    /// school name
    pub school: String,

    /// school year
    pub year: String,

    /// valid from header field
    pub valid_from: chrono::NaiveDate,

    /// updated at header field
    pub updated_at: chrono::NaiveDateTime,

    /// date of the entry
    pub date: chrono::NaiveDate,

    /// week type
    pub week: Week,

    /// Free lessons for everyone
    pub FreeLessons: Option<String>,

    /// teachers registerd not there
    pub missing_teachers: Vec<Teacher>,

    /// rooms registered blocked
    pub blocked_rooms: Vec<Room>,

    /// classes affected at this day
    pub affected_classes: Vec<Class>,

    /// entries in this day
    pub entries: Vec<Entry>,
}

impl DSB {
    pub fn new() -> Self {
        Self {
            school: String::new(),
            year: String::new(),
            valid_from: NaiveDate::from_ymd(1970, 1, 1),
            updated_at: NaiveDateTime::from_timestamp(0, 0),
            date: NaiveDate::from_ymd(1970, 1, 1),
            week: Week::A,
            FreeLessons: None,
            missing_teachers: Vec::new(),
            blocked_rooms: Vec::new(),
            affected_classes: Vec::new(),
            entries: Vec::new(),
        }
    }

    /// create new instance from mon_head table dom tree
    fn new_mon_head(handle: &Node) -> Self {
        let mut dsb: DSB = DSB::new();
        let node: &Node = handle;
        let node: &Node = &node.children.borrow()[1];
        let node: &Node = &node.children.borrow()[0];
        let node: &Node = &node.children.borrow()[5];
        let node: &Node = &node.children.borrow()[1];
        let schule: &Node = &node.children.borrow()[0];

        if let NodeData::Text { ref contents } = schule.data {
            dsb.school = escape_default(&contents.borrow());
        }
        drop(schule);

        let year: &Node = &node.children.borrow()[4];
        if let NodeData::Text { ref contents } = year.data {
            let year = escape_default(&contents.borrow());
            let year: Vec<&str> = year.split(" ").collect();
            if let Some(year) = year.last() {
                dsb.year = year.to_string();
            }
        }
        drop(year);

        let date: &Node = &node.children.borrow()[6];

        if let NodeData::Text { ref contents } = date.data {
            let date = escape_default(&contents.borrow());
            let date = date.trim();
            let date: Vec<&str> = date.split(" ").collect();
            if let Some(date) = date.last() {
                dsb.valid_from = NaiveDate::parse_from_str(date, "%d.%m.%Y")
                    .unwrap_or(NaiveDate::from_ymd(1870, 1, 1));
            }
        }

        let date: &Node = &node.children.borrow()[8];

        if let NodeData::Text { ref contents } = date.data {
            let date = escape_default(&contents.borrow());
            let date = date.trim();
            let date: Vec<&str> = date.split(" ").collect();
            if let Some(date) = date.get(date.len() - 2..date.len()) {
                dsb.updated_at = chrono::NaiveDateTime::parse_from_str(
                    &format!("{} {}", date[0], date[1]),
                    "%d.%m.%Y %k:%M",
                )
                .unwrap_or(NaiveDateTime::from_timestamp(0, 0));
            }
        }
        dsb
    }

    /// parse mon_title string to DSB info
    fn parse_mon_title(&mut self, info: &str) -> Result<()> {
        self.week = Week::parse(info.as_bytes()[info.len() - 1] as char);
        let strs = info.split_ascii_whitespace().collect::<Vec<&str>>();
        self.date = chrono::NaiveDate::parse_from_str(strs[0], "%d.%m.%Y").unwrap(); //FIXME: unwrap
        Ok(())
    }

    /// parse info table
    fn parse_info_table(&mut self, node: &Node) {
        let node: &Node = &node.children.borrow()[1];

        for v in node.children.borrow().iter() {
            let v: &Node = v;
            if v.children.borrow().len() != 2 {
                continue;
            } else {
                let infoType: &Node = &v.children.borrow()[0];
                let infoType: &Node = &infoType.children.borrow()[0];
                let mut infoString = String::new();
                if let NodeData::Text { ref contents } = infoType.data {
                    infoString = escape_default(&contents.borrow());
                }
                let infoString = infoString.trim();

                let content: &Node = &v.children.borrow()[1];
                let content: &Node = &content.children.borrow()[0];
                let mut contentString = String::new();
                if let NodeData::Text { ref contents } = content.data {
                    contentString = escape_default(&contents.borrow());
                }
                let contentString = contentString.trim();

                if infoString.to_lowercase() == "abwesende lehrer" {
                    let contentString: Vec<&str> = contentString.split(", ").collect();
                    for v in contentString.iter() {
                        let v: &str = v.trim();
                        let v: Vec<&str> = v.split(" ").collect();
                        if v.len() != 1 {
                            eprintln!("Error: DSB: unimplemented: td.info.{{Abwesende Lehrer}}.len {{{}}} {:?}", v.len(), v);
                        }
                        self.missing_teachers.push(Teacher {
                            name: v[0].to_string(),
                        });
                    }
                } else if infoString.to_lowercase() == "betroffene klassen" {
                    let contentString: Vec<&str> = contentString.split(", ").collect();
                    for v in contentString.iter() {
                        let v: &str = v.trim();
                        self.affected_classes.push(Class {
                            name: v.to_string(),
                        });
                    }
                } else {
                    eprintln!(
                        "Error: DSB: unimplemented: td.info.{{{}}} {{{}}}",
                        infoString, contentString
                    );
                }
            }
        }
    }
}

pub struct Hour {
    pub string: String,
    pub start: chrono::DateTime<Utc>,
    pub duration: chrono::Duration,
}

#[derive(Debug)]
pub struct Duration {
    pub from: u8,
    pub to: u8,
}

impl Duration {
    pub fn new() -> Self {
        Self { from: 0, to: 0 }
    }
}

#[derive(Debug)]
pub enum EntryKind {
    Unknow(String),
    Substitution,
    Dropped,
    Special,
    Changed,
}

impl EntryKind {
    pub fn new() -> Self {
        EntryKind::Unknow(String::new())
    }
    fn parse_from_str(input: &str) -> Self {
        if input.to_lowercase().contains("vertr") {
            return EntryKind::Substitution;
        } else if input.to_lowercase().contains("entf\\u{fffd}lllt") {
            return EntryKind::Dropped;
        } else if input.to_lowercase().contains("sondereins") {
            return EntryKind::Special;
        } else if input.to_lowercase().contains("ge\\u{fffd}ndert") {
            return EntryKind::Changed;
        } else if input.to_lowercase().contains("betreuung") {
            return EntryKind::Special;
        } else {
            eprintln!("Error: DBS: EntryKind: could not parse {}", input);
            return EntryKind::Unknow(input.to_string());
        }
    }
}

#[derive(Debug)]
pub struct Entry {
    pub name: String,
    pub course: Course,
    pub old_course: Option<Course>,
    pub time: Duration,
    pub new_teacher: Option<Teacher>,
    pub old_teacher: Teacher,
    pub message: String,
    pub kind: EntryKind,
    pub room: Option<Room>,
}

impl Entry {
    /// create new skeleton
    pub fn new() -> Self {
        Self {
            name: String::new(),
            course: Course::new(),
            old_course: None,
            time: Duration::new(),
            new_teacher: None,
            old_teacher: Teacher::new(),
            message: String::new(),
            kind: EntryKind::new(),
            room: None,
        }
    }

    pub fn new_from_str(name: &str) -> Self {
        let mut entry = Self::new();
        entry.name = name.to_string();
        entry
    }
}

#[derive(Debug)]
pub struct Course {
    pub name: String,
}

impl Course {
    /// create new instance
    pub fn new() -> Self {
        Self {
            name: String::new(),
        }
    }
}

// FIXME: Copy of str::escape_default from std, which is currently unstable
pub fn escape_default(s: &str) -> String {
    s.chars().flat_map(|c| c.escape_default()).collect()
}
