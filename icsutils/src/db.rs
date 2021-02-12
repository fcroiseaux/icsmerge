use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IcsCal {
    //The list of ical urls we want to merge
    pub name: String,
    pub ics_url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CalMerge {
    //The list of ical urls we want to merge
    pub name: String,
    pub url: String,
    pub calendars: Vec<IcsCal>,
}

pub mod ejdb2;

