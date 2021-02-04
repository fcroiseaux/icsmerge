use std::io::BufReader;

use stringreader::StringReader;

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

const ICAL_KEYWORDS: [&str; 9] = [
    BEGIN_VCALENDAR,
    END_VCALENDAR,
    METHOD,
    LOCATION,
    DESCRIPTION,
    PRODID,
    VERSION,
    X_WR_CALNAME,
    CALSCALE,
];

pub const NEW_LINE: &str = "\n";

pub fn parse_calendar_content(calendar: &str, resp: String) -> String {
    fn not_filtered_keywords(cal_line: &str, keywords: Vec<&str>) -> bool {
        keywords
            .iter()
            .find(|&&l| cal_line.starts_with(l))
            .is_none()
    }

    let reader = ical::LineReader::new(BufReader::new(StringReader::new(&resp)));
    let r =
        reader.filter_map(
            |l| match not_filtered_keywords(l.as_str(), ICAL_KEYWORDS.to_vec()) {
                false => None,
                true => {
                    let ll = l.as_str();
                    if ll.starts_with(SUMMARY) {
                        Some(String::from(&(SUMMARY.to_owned() + calendar + NEW_LINE)))
                    } else {
                        Some(ll.to_string() + NEW_LINE)
                    }
                }
            },
        );

    r.collect::<String>()
}
