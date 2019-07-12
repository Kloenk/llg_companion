#[derive(Debug)]
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
    pub fn from_dsb_str(input: &str) -> Self {
        if input.len() != 4 {
            eprintln!("Error: DSB: Room: from_str: {} not len 4", input);
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
            eprintln!("Error: DSB: Room: from_str: could not parse {{{}}}", input);
            return Room::None;
        }
    }
}

impl Default for Room {
    fn default() -> Self {
        Room::None
    }
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

impl Default for Teacher {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
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
    pub fn parse_planinfo_teacher(&mut self, input: &str, teacher: &str) {
        self.teacher.name = teacher.to_string();
        if input.is_empty() {
            return;
        }
        let sClass: u32 = (input.as_bytes()[0] as u32 - '0' as u32) as u32;
        if sClass > 0 || sClass < 9 {
            self.course = Course::Sec1 {
                name: input.to_string(),
            };
        }
        eprintln!("implement all other parsing function for planinfo_teacher!!!");
    }
    pub fn parse_planinfo_room(&mut self, input: &str, room: &str) {
        self.room = Room::from_dsb_str(input);
        eprintln!("implement all other parsing function for planinfo_room!!!");
    }
    pub fn parse_planinfo_student(&mut self, input: &str, courseString: &str) {
        self.is_tutor = true;
        eprintln!("implement all other parsing function for planinfo_room!!!");
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

#[derive(Debug)]
pub enum Course {
    None,
    Sec1 {
        name: String,
    },
    Sec2 {
        track: u8,
        name: String,
        kind: CourseKind,
    },
    Sec2Exam {
        track: u8,
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
    pub fn from_dsb_str(class: &str, course: &str) -> Self {
        let course = course.trim().trim_matches('_');
        if course.is_empty() {
            return Course::None;
        }

        if class.is_empty() && class.contains("---") {
            eprintln!(
                "Error: DSB: Course: could not parse {} (empty string)",
                class
            );
            return Course::None;
        }
        if class.to_lowercase().contains("klausur") {
            eprintln!("Error: DSB: Course: could not parse exam: {{{}}}", course);
            return Course::Sec2Exam {
                track: 0,
                name: String::new(),
                kind: CourseKind::GK { number: 0 },
            };
        }

        let sClass: u32 = (class.as_bytes()[0] as u32 - '0' as u32) as u32;
        if sClass > 0 || sClass < 9 {
            return Course::Sec1 {
                name: course.to_string(),
            };
        }

        let course: Vec<&str> = course.split("-").collect();
        if course.len() != 2 {
            eprintln!(
                "Error: DSB: Course: could not parse {} (wrong number of arguments)",
                course.connect(" ")
            );
            return Course::None;
        }
        let name = course[0].to_ascii_uppercase();
        let kind: CourseKind = CourseKind::from_dsb_str(course[1]);

        Course::Sec2 {
            track: 0,
            name,
            kind,
        }
    }
}

impl Default for Course {
    fn default() -> Self {
        Course::None
    }
}

#[derive(Debug)]
pub enum CourseKind {
    None,
    GK { number: u8 },
    LK { number: u8 },
}

impl CourseKind {
    /// create new non Course
    pub fn new() -> Self {
        CourseKind::None
    }
    /// parse dsb courseKind string
    pub fn from_dsb_str(kind: &str) -> Self {
        let kind = kind.trim();
        if kind.is_empty() {
            return CourseKind::None;
        }
        if kind.len() != 3 {
            eprintln!(
                "Error: DSB: CourseKind: kind string is not len 3 {{{}}}",
                kind
            );
            return CourseKind::None;
        }
        let number: u8 = (kind.as_bytes()[2] as u32 - '0' as u32) as u8;
        if kind.starts_with("GK") {
            return CourseKind::GK { number };
        } else if kind.starts_with("LK") {
            return CourseKind::LK { number };
        } else {
            eprintln!("Error: DSB: CourseKind: error parsing {{{}}}", kind);
            return CourseKind::None;
        }
    }
}
