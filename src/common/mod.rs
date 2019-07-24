use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Room {
    None,
    A { room: i16 },
    B { room: i16 },
    C { room: i16 },
    D { room: i16 },
    E { room: i16 },
}

impl Room {
    /// create new none room
    pub fn new() -> Self {
        Default::default()
    }
    /// parse dbs string
    pub fn from_dsb_str(input: &str, verbose: u8) -> Self {
        if input.len() != 4 {
            if verbose >= 1 {
                eprintln!("Error1: DSB: Room: from_str: {} not len 4", input);
            }
            return Room::None;
        }

        let room: &str = input.split_at(1).1;
        let room: i16 = room.parse().unwrap_or(0);

        if input.to_lowercase().trim().starts_with("a") {
            return Room::A { room };
        } else if input.to_lowercase().trim().starts_with("b") {
            return Room::B { room };
        } else if input.to_lowercase().trim().starts_with("c") {
            return Room::C { room };
        } else if input.to_lowercase().trim().starts_with("d") {
            return Room::D { room };
        } else if input.to_lowercase().trim().starts_with("e") {
            return Room::E { room };
        } else {
            if verbose >= 1 {
                eprintln!("Error1: DSB: Room: from_str: could not parse {{{}}}", input);
            }
            return Room::None;
        }
    }
}

impl Default for Room {
    fn default() -> Self {
        Room::None
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

impl Default for Teacher {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hour {
    /// room where the period takes place
    pub room: Room,

    /// Teacher of this course
    pub teacher: Teacher,

    /// is tutor course
    pub is_tutor: bool,

    /// course
    pub course: Course,
}

impl Hour {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn parse_planinfo_teacher(&mut self, input: &str, teacher: &str, verbose: u8) {
        let input = input.trim();
        self.teacher.name = teacher.to_string();
        self.is_tutor = false;
        if input.is_empty() {
            return;
        }
        if input.contains("LZ")
            || input.contains("BS")
            || input.contains("VBS")
            || input.contains("UEMI")
            || input.contains("SPI")
            || input.contains("AG")
        {
            if verbose >= 2 {
                eprintln!("Error2: Hour: planInfo_Teacher: no parser for {}", input);
            }
            self.course = Course::Sec1 {
                name: input.to_string(),
            };
            return;
        }

        let inVec: Vec<&str> = input.split_ascii_whitespace().collect();
        if inVec.len() != 3 {
            if verbose >= 1 {
                eprintln!(
                    "Error1: Hour: planInfo_Teacher: vec len not 3 but {}",
                    inVec.len()
                );
            }
            return;
        }

        let sClass: u32 = (input.as_bytes()[0] as u32 - '0' as u32) as u32;
        if sClass > 0 && sClass < 9 {
            self.course = Course::Sec1 {
                name: format!("{} {}", inVec[1], inVec[0]),
            };
        } else {
            self.course = Course::from_planinfo_teacher_str(inVec[0], inVec[1]);
        }
        self.room = Room::from_dsb_str(inVec[2].trim(), verbose);
    }
    pub fn parse_planinfo_room(&mut self, input: &str, room: &str, verbose: u8) {
        let input = input.trim();
        let room = room.trim();
        self.is_tutor = false;
        if input.contains("LZ")
            || input.contains("BS")
            || input.contains("VBS")
            || input.contains("UEMI")
            || input.contains("SPI")
            || input.contains("AG")
        {
            if verbose >= 1 {
                eprintln!("Error1: Hour: planInfo_Room: no parser for {}", input);
            }
            self.course = Course::Sec1 {
                name: input.to_string(),
            };
            return;
        }

        self.room = Room::from_dsb_str(input, verbose);

        let inVec: Vec<&str> = input.split_ascii_whitespace().collect();
        if inVec.len() != 3 {
            if verbose >= 1 {
                eprintln!(
                    "Error: Hour: planInfo_Room: vec len not 3 but {}",
                    inVec.len()
                );
            }
            return;
        }

        let sClass: u32 = (input.as_bytes()[0] as u32 - '0' as u32) as u32;
        if sClass > 0 && sClass < 9 {
            self.course = Course::Sec1 {
                name: format!("{} {}", inVec[1], inVec[0]),
            };
        } else {
            self.course = Course::from_planinfo_room_str(inVec[0], inVec[1]);
        }
    }
    pub fn parse_planinfo_student(&mut self, input: &str, courseString: &str, verbose: u8) {
        let input = input.trim();
        let courseString = courseString.trim();

        let inVec: Vec<&str> = input.split_ascii_whitespace().collect();
        if inVec.len() != 4 {
            if verbose >= 1 {
                eprintln!(
                    "Error: Hour: planInfo_Student: vec len not 4 but {}",
                    inVec.len()
                );
            }
            return;
        }
        self.room = Room::from_dsb_str(inVec[3], verbose);

        self.teacher.name = inVec[2].to_string();
        self.is_tutor = courseString.contains(&self.teacher.name);
        self.course = Course::from_planinfo_students_str(inVec[0], inVec[1]);
    }
}

impl Default for Hour {
    fn default() -> Self {
        Self {
            room: Default::default(),
            teacher: Default::default(),
            is_tutor: false,
            course: Default::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Course {
    None,
    Sec1 {
        name: String,
    },
    Sec2 {
        track: i16,
        name: String,
        kind: CourseKind,
    },
    Sec2Exam {
        track: i16,
        name: String,
        kind: CourseKind,
    },
}

impl Course {
    // create new Course
    pub fn new() -> Self {
        Course::None
    }
    /// parse dsb to Course
    pub fn from_dsb_str(class: &str, course: &str, verbose: u8) -> Self {
        let course = course.trim().trim_matches('_');
        if course.is_empty() {
            return Course::None;
        }

        if class.is_empty() && class.contains("---") {
            if verbose >= 1 {
                eprintln!(
                    "Error: DSB: Course: could not parse {} (empty string)",
                    class
                );
            }
            return Course::None;
        }
        if class.to_lowercase().contains("klausur") {
            if verbose >= 1 {
                eprintln!("Error1: DSB: Course: could not parse exam: {{{}}}", course);
            }
            return Course::Sec2Exam {
                track: 0,
                name: String::new(),
                kind: CourseKind::GK { number: 0 },
            };
        }

        let sClass: u32 = (class.as_bytes()[0] as u32 - '0' as u32) as u32;
        if sClass > 0 && sClass < 9 {
            return Course::Sec1 {
                name: course.to_string(),
            };
        }

        let course: Vec<&str> = course.split("-").collect();
        if course.len() != 2 {
            if verbose >= 1 {
                eprintln!(
                    "Error: DSB: Course: could not parse {} (wrong number of arguments)",
                    course.connect(" ")
                );
            }
            return Course::None;
        }
        let name = course[0].to_ascii_uppercase();
        let kind: CourseKind = CourseKind::from_dsb_str(course[1], verbose);

        Course::Sec2 {
            track: 0,
            name,
            kind,
        }
    }
    /// parse planinfo teacher string
    /// this function is indetend to be run after deciding that it is not a Sec 1 course
    pub fn from_planinfo_teacher_str(course: &str, namein: &str) -> Self {
        let course = course.trim();
        let namein = namein.trim();
        let kind = CourseKind::from_planinfo_teacher_str(namein);
        let mut track: i16 = 0;
        if let Some(nr) = course.as_bytes().last() {
            track = (*nr as u32 - '0' as u32) as i16;
        }
        let mut name = String::new();
        let data: Vec<&str> = namein.split("-").collect();
        if let Some(data) = data.first() {
            name = data.to_string() + " ";
        }
        let data: Vec<&str> = course.split("-").collect();
        if let Some(data) = data.first() {
            name += data;
        }
        Course::Sec2 { track, name, kind }
    }
    /// parse planinfo teacher string
    /// this function is indetend to be run after deciding that it is not a Sec 1 course
    pub fn from_planinfo_room_str(course: &str, namein: &str) -> Self {
        let course = course.trim();
        let namein = namein.trim();
        let kind = CourseKind::from_planinfo_teacher_str(namein);
        let track = 0;
        let mut name = String::new();
        let data: Vec<&str> = namein.split("-").collect();
        if let Some(data) = data.first() {
            name = data.to_string() + " ";
        }
        let data: Vec<&str> = course.split("-").collect();
        if let Some(data) = data.first() {
            name += data;
        }
        Course::Sec2 { track, name, kind }
    }
    /// parse planinfo teacher string
    /// this function is indetend to be run after deciding that it is not a Sec 1 course
    pub fn from_planinfo_students_str(course: &str, namein: &str) -> Self {
        let course = course.trim();
        let namein = namein.trim();
        let kind = CourseKind::from_planinfo_teacher_str(namein);
        let mut track: i16 = 0;
        let course = course.trim_matches('(');
        let course = course.trim_matches(')');
        if let Some(nr) = course.as_bytes().last() {
            track = (*nr as u32 - '0' as u32) as i16;
        }

        let namein: Vec<&str> = namein.split('-').collect();
        let name = namein[0].to_string();

        Course::Sec2 { track, name, kind }
    }
}

impl Default for Course {
    fn default() -> Self {
        Course::None
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CourseKind {
    None,
    GK { number: i16 },
    LK { number: i16 },
}

impl CourseKind {
    /// create new non Course
    pub fn new() -> Self {
        CourseKind::None
    }
    /// parse dsb courseKind string
    pub fn from_dsb_str(kind: &str, verbose: u8) -> Self {
        let kind = kind.trim();
        if kind.is_empty() {
            return CourseKind::None;
        }
        if kind.len() != 3 && kind.len() != 4 {
            if verbose >= 1 {
                eprintln!(
                    "Error1: DSB: CourseKind: kind string is not len 3 {{{}}}",
                    kind
                );
            }
            return CourseKind::None;
        }
        //let number: i16 = (kind.as_bytes()[2] as u32 - '0' as u32) as i16;
        let number: &str = &kind[2..kind.len()];
        let number: i16 = number.parse().unwrap_or(0);
        if kind.starts_with("GK") {
            return CourseKind::GK { number };
        } else if kind.starts_with("LK") {
            return CourseKind::LK { number };
        } else {
            if verbose >= 1 {
                eprintln!("Error1: DSB: CourseKind: error parsing {{{}}}", kind);
            }
            return CourseKind::None;
        }
    }
    /// parse planinfo teacher str
    pub fn from_planinfo_teacher_str(course: &str) -> Self {
        let course = course.trim();
        if course.is_empty() {
            return CourseKind::None;
        }
        let mut number: i16 = 0;
        if let Some(nr) = course.as_bytes().last() {
            number = (*nr as u32 - '0' as u32) as i16;
        }
        if course.to_lowercase().contains("lk") {
            return CourseKind::LK { number };
        }
        return CourseKind::GK { number };
    }
}
