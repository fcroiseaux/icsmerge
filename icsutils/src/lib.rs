use std::io::BufReader;

use stringreader::StringReader;
use crate::db::IcsCal;

pub mod db;

/// ical kerwords
pub const BEGIN_VTIMEZONE: &str = "BEGIN:VTIMEZONE";
pub const BEGIN_VCALENDAR: &str = "BEGIN:VCALENDAR";
pub const END_VCALENDAR: &str = "END:VCALENDAR";
pub const LOCATION: &str = "LOCATION:";
pub const SUMMARY: &str = "SUMMARY:";
pub const DESCRIPTION: &str = "DESCRIPTION:";
pub const METHOD: &str = "METHOD:";
pub const PRODID: &str = "PRODID:";
pub const VERSION: &str = "VERSION:";
pub const X_WR_CALNAME: &str = "X-WR-CALNAME:";
pub const CALSCALE: &str = "CALSCALE:";

/// List of keywords that must be ignored when the calendar is private
const PRIV_ICAL_KEYWORDS: [&str; 2] = [
    DESCRIPTION,
    PRODID
];

/// List of keywords that must be ignored in order to merge calendars
const PUB_ICAL_KEYWORDS: [&str; 7] = [
    BEGIN_VCALENDAR,
    END_VCALENDAR,
    METHOD,
    PRODID,
    VERSION,
    X_WR_CALNAME,
    CALSCALE,
];

pub const NEW_LINE: &str = "\n";

/// parse an ical stream
/// calendar: IcsCal the detailed information about the calendar
/// ical_str: Sttring the content of the file
pub fn parse_calendar_content(calendar: IcsCal, ical_str: String) -> String {
    fn not_filtered_keywords(cal_line: &str, keywords: &Vec<&str>) -> bool {
        keywords
            .iter()
            .find(|&&l| cal_line.starts_with(l))
            .is_none()
    }

    let reader = ical::LineReader::new(BufReader::new(StringReader::new(&ical_str)));
    let mut keywords = PUB_ICAL_KEYWORDS.to_vec();
    if calendar.is_private  {
        keywords.extend_from_slice(&PRIV_ICAL_KEYWORDS.to_vec());
    };
    let r =
        reader.filter_map(
            |l| match not_filtered_keywords(l.as_str(), &keywords) {
                false => None,
                true => {
                    let ll = l.as_str();
                    if calendar.is_private && ll.starts_with(SUMMARY) {
                        Some(String::from(&(SUMMARY.to_owned() + &calendar.name + NEW_LINE)))
                    } else {
                        Some(ll.to_string() + NEW_LINE)
                    }
                }
            },
        );

    r.collect::<String>()
}
