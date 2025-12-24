use chrono::{Utc, FixedOffset};
use super::module::ModuleInfo;

const TIMEZONE_OFFSET: FixedOffset = FixedOffset::east_opt(3 * 60 * 60).expect("Not a valid offset");

pub struct ClockModule;

impl ClockModule {
    pub fn new() -> Self { Self {} }
}

impl ModuleInfo for ClockModule {
    fn display(&mut self) -> String {
        let now = Utc::now().with_timezone(&TIMEZONE_OFFSET);
        now.format("%d %H %M %S").to_string()
    }
}
