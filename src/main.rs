// #![deny(warnings)]

use std::convert::Infallible;
use hyper::{Body, Client, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};

async fn ics_merge(_: Request<Body>) -> Result<Response<Body>, Infallible> {
    let calendars = [
        "https://webmail.intech.lu/owa/calendar/25be2d37664e47899a9c952e5d652e98@Intech.lu/497fedb4dfb34173a0770b0879cbacbc17006863668209442216/calendar.ics",
        "https://calendar.google.com/calendar/ical/fcroiseaux%40gmail.com/private-db183d5c924a98df943d4ed104fbb95f/basic.ics"
    ];

    let resp = match fetch_calendar_content(calendars[0]).await {
        Ok(f) => f,
        Err(E) => String::from("Error")
    };

    Ok(Response::new(Body::from(resp)))
}

async fn fetch_calendar_content(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let resp = reqwest::get(url)
        .await?
        .text()
        .await?;
    let mut in_content = false;
    let mut content = String::from("");
    for lineR in resp.lines() {
        let line = String::from(lineR);
        if (!in_content) {
            if (line.starts_with("BEGIN:VTIMEZONE")) {
                in_content = true;
                content.push_str(&line)
            }
        } else if (!line.starts_with("END:VCALENDAR")) {
            content.push_str(&line);
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

    let addr = ([127, 0, 0, 1], 8080).into();

    let server = Server::bind(&addr).serve(make_svc);

    println!("Listening on http://{}", addr);

    server.await?;

    Ok(())
}

