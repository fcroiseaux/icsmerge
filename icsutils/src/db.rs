//! The DB module is used to isolate the databse and to ease database switching.
//!
//! Two DB sub-modules are provided: one for sled and the second for ejdb
//! You must reference the module you want to use in icsmerge crate :
//! ```
//! use icsutils::db::ejdb2::*;
//! ```
//! or
//! ```
//! use icsutils::db::ejdb2::*;
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
    /// The list of calendars that must be merged
    pub calendars: Vec<IcsCal>,
}

pub mod ejdb2;
pub mod sled2;
