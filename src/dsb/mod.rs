use serde_json::json;
use std::io::prelude::*;
use std::clone::Clone;
use std::thread;
use chrono::prelude::*;
use super::error::Error;

use html5ever::parse_document;
use html5ever::rcdom::{Handle, NodeData, RcDom, Node};
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
            url: String::from("https://www.dsbmobile.de/JsonHandlerWeb.ashx/GetData"),
        }
    }


    /// start parser
    pub fn run(&self) -> Result<()> {
        let conf = self.clone();
        thread::spawn(move || {
            println!("start");
            conf.run_int();
        });
        Ok(())
    }

    /// internal run function holding the mail loop of the thread
    fn run_int(self) {
        self.get();
        loop {

        }
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

        println!("content: {:?}", dsb);
        let body = dsb.text().unwrap();
        println!("body: {}", body.clone());

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
            println!("header: {}: {:?}", h, v);
        }

        let html = html.text().unwrap();
        self.parse(&html);
        

        Ok(())  // change
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

        println!("data: {}", String::from_utf8_lossy(&data));
        let json: serde_json::Value = serde_json::from_str(&String::from_utf8_lossy(&data))?;

        if json.get("ResultMenuItems") == None {
            return Err(super::error::Error::new_field_not_exists("data.ResultMenuItems".to_string()));
        }
        let json = json.get("ResultMenuItems").unwrap();

        if !json.is_array() {
            return Err(super::error::Error::new_field_not_exists("data.ResultMenuItems".to_string()));
        }
        let json = json.as_array().unwrap();

        let mut x = false;
        let mut index = 0;
        while !x {
            if json.get(index) == None {
                return Err(super::error::Error::new_field_not_exists(format!("data.ResultMenuItems.{}", index)));
            }
            let json = json.get(index).unwrap();
            if json.get("Title") == None {
                return Err(super::error::Error::new_field_not_exists(format!("data.ResultMenuItems.{}.Title", index)));
            }
            let title = json.get("Title").unwrap();
            if let Some(title) = title.as_str() {
                if title == "Inhalte" {
                    x = true;

                    if json.get("Childs") == None {
                        return Err(
                            super::error::Error::new_field_not_exists(
                                format!("data.ResultMenuItems.{}.Childs", index)
                            )
                        );
                    }
                    let json = json.get("Childs").unwrap();
                    if !json.is_array() {
                        return Err(
                            super::error::Error::new_field_not_exists(
                                format!("data.ResultMenuItems.{}.Childs", index)
                            )
                        );
                    }
                    let json = json.as_array().unwrap();
                    let mut y = false;
                    let mut indexy = 0;
                    for v in json {
                        indexy += 1;
                        let title = v.get("Title");
                        if title == None {
                            return Err(
                                super::error::Error::new_field_not_exists(
                                    format!("data.ResultMenuItems.{}.Childs.{}.Title", index, indexy)
                                )
                            );
                        }
                        let title = title.unwrap();
                        if !title.is_string() {
                            return Err(
                                super::error::Error::new_field_not_exists(
                                    format!("data.ResultMenuItems.{}.Childs.{}.Title", index, indexy)
                                )
                            );
                        }
                        let title = title.as_str().unwrap().to_string();
                        if title == "Pläne" {
                            let v = v.get("Root");
                            if v == None {
                                return Err(
                                    super::error::Error::new_field_not_exists(
                                        format!("data.ResultMenuItems.{}.Childs.{}.Root", index, indexy)
                                    )
                                );
                            }
                            let v = v.unwrap();
                            let v = v.get("Childs");
                            if v == None {
                                return Err(
                                    super::error::Error::new_field_not_exists(
                                        format!("data.ResultMenuItems.{}.Childs.{}.Root.Childs", index, indexy)
                                    )
                                );
                            }
                            let v = v.unwrap();
                            if !v.is_array() {
                                return Err(
                                    super::error::Error::new_field_not_exists(
                                        format!("data.ResultMenuItems.{}.Childs.{}.Root.Childs", index, indexy)
                                    )
                                );
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
                return Err(super::error::Error::new_field_not_exists(format!("data.ResultMenuItems.{}", index)));
            }
        }
        Ok(url)
    }


    /// parse dsb content
    fn parse(&self, html: &str) -> Result<()> {
        //let mut html = html.to_string();
        let html = html.replace("&nbsp;", " ");
        let dom = parse_document(RcDom::default(), Default::default())
            .from_utf8()
            .read_from(&mut html.as_bytes()).unwrap();
        let mut dsb: Vec<DSB> = Vec::new();
        self.parse_dom(&dom.document).unwrap();
        Ok(())
    }
    
    fn parse_dom(&self, handle: &Handle) -> Result<Vec<DSB>> {
        let mut dsb_return: Vec<DSB> = Vec::new();
        let node: &Node = handle;
        let nodeVec = node.children.borrow();
        let node: &Node = &nodeVec[0];

        for v in node.children.borrow().iter() {
            let v: &Node = v;
            if let NodeData::Element {
                ref name,
                ..
            } = v.data {
                let name: &html5ever::QualName = name;
                if name.local.to_string() == "body" {
                    let mut found_mod_head = false;
                    for w in v.children.borrow().iter() {
                        let w: &Node = w;
                        if let NodeData::Element {
                                ref name,
                                ref attrs,
                                ..
                            } = w.data {
                                let name: &html5ever::QualName = name;
                                let attrs: &Vec<html5ever::Attribute> = &attrs.borrow();
                                if name.local.to_string() == "table" {
                                    for attr in attrs.iter() {
                                        if attr.name.local.to_string() == "class" && attr.value.to_string() == "mon_head" {
                                            dsb_return.push(DSB::new());
                                            
                                            found_mod_head = true;
                                        }
                                    }
                                } else if name.local.to_string() == "center" && found_mod_head {
                                    if let Some(dsb) = dsb_return.last_mut() {
                                        let dsb: &mut DSB = dsb;
                                        eprintln!("not implemented dsb center parse");
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

    // walk throud dsb html
    /*fn parse_walk(&self, handle: &Handle, dsb: &mut Vec<DSB>, job: &Job) {
        let mut jobNew = job.clone();
        println!("job: {:?}", jobNew);
        let node = handle;
        match node.data {
            NodeData::Text { ref contents } => {
                println!("text: {:?}", job);
                match job {
                    Job::MON_TITLE => {
                        if let Some(dsbEntry) = dsb.last_mut() {
                            dsbEntry.parse_mon_title(&escape_default(&contents.borrow()));
                        }
                        //jobNew = Job::NOOP;
                    },
                    Job::InfoName => {
                        let meta = &escape_default(&contents.borrow());
                        if meta.to_lowercase().contains("unterrichtsfrei") {
                            jobNew = Job::InfoFreeLessons;
                            println!("unterrichtsfrei: {:?}", jobNew.clone());
                        } else {
                            println!("something else");
                            jobNew = Job::MON_TITLE;
                        }
                    },
                    Job::InfoFreeLessons => {
                        let content = escape_default(&contents.borrow());
                        println!("content: {}", content);
                        if let Some(dsbEntry) = dsb.last_mut() {
                            println!("Free Lessons: {}", content);
                            dsbEntry.FreeLessons = Some(content);
                        }
                        //jobNew = Job::NOOP;
                    },
                    _ => (),
                }
            },
            NodeData::Element {
                ref name,
                ref attrs,
                ..
            } => {
                println!("element: {:?}", job);
                if name.local.to_string() == "div" {
                    for attr in attrs.borrow().iter() {
                        if attr.name.local.to_string() == "class" && attr.value.to_string() == "mon_title" {
                            dsb.push(DSB::new());
                            jobNew = Job::MON_TITLE;
                        }
                    }
                }
                if name.local.to_string() == "td" {
                    for attr in attrs.borrow().iter() {
                        if attr.name.local.to_string() == "class" && attr.value.to_string() == "info" /*&& job == &Job::NOOP*/ {
                            println!("found info: {:?}", job);
                            jobNew = Job::InfoName;
                        }
                    }
                }
            },
            _ => println!("other: {:?}", job),
        }
        println!("give: {:?}", jobNew);
        if let Some(handle) = &node.children.borrow().last() {
            self.parse_walk(handle, dsb, &jobNew);
        } else {
            println!("ended");
        }
    }*/
}

/// enum for previos cell to determ its content
#[derive(Debug, PartialEq, Clone)]
enum Job {
    /// no valid data
    NOOP,
    /// MON_TITLE day time week string
    MON_TITLE,
    /// info Name for td.info
    InfoName,
    /// td.info field Unterrichtsfrei
    InfoFreeLessons,
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
pub enum Building {
    A,
    B,
    C,
    D,
    E,
}

pub struct Teacher {
    pub name: String
}

pub struct Room {
    pub building: Building,
    pub room: i16,
}

pub struct Class {

}

pub struct DSB {
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
            date: NaiveDate::from_ymd(1970, 1, 1),
            week: Week::A,
            FreeLessons: None,
            missing_teachers: Vec::new(),
            blocked_rooms: Vec::new(),
            affected_classes: Vec::new(),
            entries: Vec::new(),
        }
    }

    /// parse mon_title string to DSB info
    fn parse_mon_title(&mut self, info: &str) -> Result<()> {

        self.week = Week::parse(info.as_bytes()[info.len()-1] as char);
        let strs = info.split_ascii_whitespace().collect::<Vec<&str>>();
        self.date = chrono::NaiveDate::parse_from_str(strs[0], "%d.%m.%Y").unwrap();    //FIXME: unwrap
        Ok(())
    }
}

pub struct Hour {
    pub string: String,
    pub start: chrono::DateTime<Utc>,
    pub duration: chrono::Duration,
}

pub struct Entry {
    pub course: Course,
    pub time: chrono::Duration,
    pub new_teacher: Option<Teacher>,
    pub old_teacher: Teacher,
}

pub struct Course {
    pub name: String,
}

// FIXME: Copy of str::escape_default from std, which is currently unstable
pub fn escape_default(s: &str) -> String {
    s.chars().flat_map(|c| c.escape_default()).collect()
}