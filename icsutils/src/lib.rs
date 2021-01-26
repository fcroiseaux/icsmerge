#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

pub async fn fetch_calendar_content(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let resp = reqwest::get(url)
        .await?
        .text()
        .await?;

    let mut in_content = false;
    let mut content = String::new();
    for ics_line in resp.lines() {
        let line = String::from(ics_line);
        if !in_content {
            if line.starts_with("BEGIN:VTIMEZONE") {
                in_content = true;
                content.push_str(&line);
                content.push_str(&"\n");
            }
        } else if !line.starts_with("END:VCALENDAR") {
            content.push_str(&line);
            content.push_str(&"\n");
        }
    }
    return Ok(content);
}
