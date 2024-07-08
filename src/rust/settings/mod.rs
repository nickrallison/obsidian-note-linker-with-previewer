#[derive(Debug)]
pub(crate) struct Settings {
    pub case_insensitive: bool,
    pub link_to_self: bool,
    pub color: String,
}

impl Settings {
    pub fn new(case_insensitive: bool, link_to_self: bool, color: String) -> Self {
        Settings {
            case_insensitive,
            link_to_self,
            color,
        }
    }

    pub fn default() -> Self {
        Settings {
            case_insensitive: true,
            link_to_self: false,
            color: String::from("red"),
        }
    }
}
