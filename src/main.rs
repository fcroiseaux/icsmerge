// #![deny(warnings)]

use actix_web::{App, get, HttpResponse, HttpServer, Responder};
use actix_web::client::Client;

use icsutils::*;

#[get("/fxzo.ics")]
async fn ics_merge() -> impl Responder {
    let calendars = [
        "https://webmail.intech.lu/owa/calendar/25be2d37664e47899a9c952e5d652e98@Intech.lu/497fedb4dfb34173a0770b0879cbacbc17006863668209442216/calendar.ics",
        "https://calendar.google.com/calendar/ical/fcroiseaux%40gmail.com/private-db183d5c924a98df943d4ed104fbb95f/basic.ics",
        "https://calendar.google.com/calendar/u/1?cid=ZmFicmljZUB0b2tlbnkuY29t",
        "https://outlook.office365.com/owa/calendar/4e690d4c256b4fca9a11d2c03328a21c@lumena.tech/04e70dc6d07c4e6c8c01377ebdab5c6f9379776718195930947/calendar.ics"
    ];

    let mut resp = String::new();

    let client = Client::default();
    for cal in &calendars {
        let ics_content: String = match client.get(*cal).send().await {
            Ok(mut resp) => match resp.body().limit(102400000).await {
                Ok(r) => String::from_utf8(r.to_vec()).unwrap_or_default(),
                Err(e) => {
                    println!("{}", e);
                    String::new()
                }
            },
            Err(e1) => {
                println!("{}", e1);
                String::new()
            }
        };
        println!("Calendar : {} fetched", &cal);
        resp.push_str(&fetch_calendar_content(ics_content));
    }
    HttpResponse::Ok().body(resp)
}

#[get("/")]
async fn home() -> impl Responder {
    HttpResponse::Ok().body("Try requesting to /fxzo.ics")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(home)
            .service(ics_merge)
    })
        .bind("0.0.0.0:8080")?
        .run()
        .await
}

