#[derive(Debug)]
pub(crate) struct Settings {
    pub case_insensitive: bool,
    pub color: String,
}

impl Settings {
    pub fn new(case_insensitive: bool, color: String) -> Self {
        Settings {
            case_insensitive,
            color,
        }
    }

    pub fn default() -> Self {
        Settings {
            case_insensitive: true,
            color: String::from("red"),
        }
    }
}
