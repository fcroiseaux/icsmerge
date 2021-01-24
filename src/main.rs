// #![deny(warnings)]

use std::convert::Infallible;

use hyper::{Body, Method, Request, Response, Server, StatusCode};
use hyper::service::{make_service_fn, service_fn};

async fn ics_merge(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => {
            Ok(Response::new(Body::from("Try POSTing data to /echo")))
        }
        (&Method::GET, "/fxzo.ics") => {
            let calendars = [
                "https://webmail.intech.lu/owa/calendar/25be2d37664e47899a9c952e5d652e98@Intech.lu/497fedb4dfb34173a0770b0879cbacbc17006863668209442216/calendar.ics",
                "https://calendar.google.com/calendar/ical/fcroiseaux%40gmail.com/private-db183d5c924a98df943d4ed104fbb95f/basic.ics"
            ];

            let mut resp = String::from("BEGIN:VCALENDAR\n");

            for cal in &calendars {
                let r = match fetch_calendar_content(cal).await {
                    Ok(f) => f,
                    Err(_e) => String::new()
                };
                resp.push_str(&r);
            }
            resp.push_str("END:VCALENDAR");

            let response = Response::builder()
                .status(200)
                .header("content-type", "text/calendar")
                .body(Body::from(resp))
                .unwrap();

            Ok(response)
        }
        _ => {
            let mut not_found = Response::default();
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}

async fn fetch_calendar_content(url: &str) -> Result<String, Box<dyn std::error::Error>> {
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


#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    pretty_env_logger::init();

    // For every connection, we must make a `Service` to handle all
    // incoming HTTP requests on said connection.
    let make_svc = make_service_fn(|_conn| {
        // This is the `Service` that will handle the connection.
        // `service_fn` is a helper to convert a function that
        // returns a Response into a `Service`.
        async { Ok::<_, Infallible>(service_fn(ics_merge)) }
    });

    let addr = ([0, 0, 0, 0], 8080).into();

    let server = Server::bind(&addr).serve(make_svc);

    println!("Listening on http://{}", addr);

    server.await?;

    Ok(())
}

