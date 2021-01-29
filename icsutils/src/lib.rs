#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

use std::io::BufReader;
use stringreader::StringReader;

pub const BEGIN_VTIMEZONE: &str = "BEGIN:VTIMEZONE";
pub const BEGIN_VCALENDAR: &str = "BEGIN:VCALENDAR";
pub const END_VCALENDAR: &str = "END:VCALENDAR";
pub const NEW_LINE: &str = "\n";

pub fn fetch_calendar_content(resp: String) -> String {
    let mut content = String::new();
    let sreader = StringReader::new(&resp);
    let buf = BufReader::new(sreader);

    let reader = ical::LineReader::new(buf);

    for line in reader {
        content.push_str(line.as_str());
        content.push_str(NEW_LINE);
    }

    return content
}

fn process_content_line(content: &mut String, line: &str) {
    content.push_str(&line);
    content.push_str(NEW_LINE);
}
