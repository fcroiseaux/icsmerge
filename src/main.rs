// #![deny(warnings)]

use actix_web::{App, get, HttpResponse, HttpServer, Responder, web};
use actix_web::client::Client;
use rand::Rng;

use icsutils::*;


async fn get_http_request(url :&str) -> String {
    let client = Client::default();
    match client.get(url).send().await {
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
    }
}

async fn ics_merge() -> impl Responder {
    let calendars = [
        ("InTech", "https://webmail.intech.lu/owa/calendar/25be2d37664e47899a9c952e5d652e98@Intech.lu/497fedb4dfb34173a0770b0879cbacbc17006863668209442216/calendar.ics"),
        ("Perso", "https://calendar.google.com/calendar/ical/fcroiseaux%40gmail.com/private-db183d5c924a98df943d4ed104fbb95f/basic.ics"),
        ("Tokeny", "https://calendar.google.com/calendar/u/1?cid=ZmFicmljZUB0b2tlbnkuY29t"),
        ("Lumena", "https://outlook.office365.com/owa/calendar/4e690d4c256b4fca9a11d2c03328a21c@lumena.tech/04e70dc6d07c4e6c8c01377ebdab5c6f9379776718195930947/calendar.ics")
    ];

    let mut resp = String::from(BEGIN_VCALENDAR);
    resp.push_str(NEW_LINE);

    for cal in &calendars {
        let ics_content = get_http_request(cal.1).await;
        println!("Calendar : {} fetched", cal.0);
        resp.push_str(&parse_calendar_content(cal.0, ics_content));
    }

    resp.push_str(END_VCALENDAR);
    HttpResponse::Ok().header("Content-Type", "text/calendar").body(resp)
}


struct AppState {
    cal_url: String
}

#[get("/")]
async fn index(url: web::Data<AppState>) -> String {
    let url_s = &url.cal_url;
    format!("Try to send request to /{}", url_s)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    let cal_s = format!("{:x}", rand::thread_rng().gen::<u64>()) + ".ics";
    let state_url = web::Data::new(AppState {
        cal_url: cal_s
    });

    HttpServer::new(move || {
        App::new()
            .app_data(state_url.clone())
            .service(index)
            .route(&state_url.cal_url, web::get().to(ics_merge))
    })
        .bind("0.0.0.0:8080")?
        .run()
        .await

}

