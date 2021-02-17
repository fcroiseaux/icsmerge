//! The DB module is used to isolate the databse and to ease database switching.
//!
//! One DB sub-module is provided for sled. If you want to use another db,
//! e.g. ejdb, create a file ejdb2.rs and reimplements the functions as is sled2.rs
//! after just replace :
//! ```
//! pub mod sled2;
//! use icsutils::db::sled2 as db;
//! ```
//! by
//! ```
//! pub mod ejdb2;
//! use icsutils::db::ejdb2 as db;
//! ```
use serde::{Deserialize, Serialize};

/// Represents the information related to one merged calendar
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct IcsCal {
    /// The name of the source calendar
    pub name: String,
    /// If true, the detail of events will be hiden
    pub is_private: bool,
    /// The url of the source calendar, provided by th calendar provider
    pub ics_url: String,
}

/// Describes the merge configuration
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct MergeConf {
    /// The name of the merged calendar
    pub name: String,
    /// The endpoint of the merged calendar, must end with .ics
    pub url: String,
    /// bcrypt encrypted password
    pub password: String,
    /// The list of calendars that must be merged
    pub calendars: Vec<IcsCal>,
}

pub mod sled2;
use crate::db::sled2 as db;


pub fn init_db() -> Result<String, String> {
    db::init_db()
}

pub fn get_cals_from_db() -> String {
    db::get_cals_from_db()
}

pub fn get_cal_from_url(url: &str) -> Option<String> {
    db::get_cal_from_url(url)
}

pub fn delete_calmerge(url: &String) -> Option<String> {
    db::delete_calmerge(url)
}

pub fn insert_cal(url: String, doc: String) -> Result<String, String> {
    db::insert_cal(url, doc)
}
