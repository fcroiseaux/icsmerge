#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

pub const BEGIN_VTIMEZONE: &str = "BEGIN:VTIMEZONE";
pub const BEGIN_VCALENDAR: &str = "BEGIN:VCALENDAR";
pub const END_VCALENDAR: &str = "END:VCALENDAR";
pub const NEW_LINE: &str = "\n";

pub fn fetch_calendar_content(resp: String) -> String {
    let mut in_content = false;
    let mut content = String::new();
    for ics_line in resp.lines() {
        let line = String::from(ics_line);
        if !in_content {
            if line.starts_with(BEGIN_VTIMEZONE) {
                in_content = true;
                content.push_str(&line);
                content.push_str(NEW_LINE);
            }
        } else if !line.starts_with(END_VCALENDAR) {
            process_content_line(&mut content, &line);
        }
    }
    return content;
}

fn process_content_line(content: &mut String, line: &str) {
    content.push_str(&line);
    content.push_str(NEW_LINE);
}
