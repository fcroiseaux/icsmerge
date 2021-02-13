use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct IcsCal {
    //The list of ical urls we want to merge
    pub name: String,
    pub is_private: bool,
    pub ics_url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct MergeConf {
    //The list of ical urls we want to merge
    pub name: String,
    pub url: String,
    pub calendars: Vec<IcsCal>,
}

pub mod ejdb2;
pub mod sled2;
