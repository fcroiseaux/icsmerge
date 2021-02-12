// #![deny(warnings)]

use actix_web::client::Client;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use rand::Rng;

use icsutils::*;

async fn get_http_request(url: &str) -> String {
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

async fn merge_calendars(calendars: Vec<icsutils::db::IcsCal>) -> String {
    let mut resp = String::from(BEGIN_VCALENDAR);
    resp.push_str(NEW_LINE);

    for cal in &calendars {
        let ics_content = get_http_request(&cal.ics_url).await;
        println!("Calendar : {} fetched", &cal.name);
        resp.push_str(&parse_calendar_content(&cal.name, ics_content));
    }

    resp.push_str(END_VCALENDAR);
    resp
}

struct AppState {
    cal_url: String,
}

#[get("/")]
async fn index(url: web::Data<AppState>) -> String {
    let url_s = &url.cal_url;
    format!("Try to send request to /{}", url_s)
}

#[post("/createcal")]
async fn create_cal(cal: web::Json<icsutils::db::CalMerge>) -> String {
    let cal_struct = icsutils::db::CalMerge {
        name: cal.name.to_string(),
        url: cal.url.to_string(),
        calendars: cal.calendars.to_vec()
    };
    match icsutils::db::create_cal(cal_struct) {
        Ok(r) => r,
        Err(e) => format!("Error /{:?}", e),
    }
}

#[get("/init")]
async fn init() -> String {
    icsutils::db::init();
    "DONE DB initialized".to_string()
}

#[get("/readdb")]
async fn readdb() -> String {
    serde_json::to_string(&icsutils::db::get_cals_from_db()).unwrap()
}

#[get("/get_cal/{cal_url}")]
async fn get_cal(path: web::Path<(String,)>) -> String {
    let url = path.into_inner().0;
    serde_json::to_string(&icsutils::db::get_cals_from_url(url)).unwrap()
}

#[get("/{cal_url}.ics")]
async fn serve_ics(path: web::Path<(String,)>) -> impl Responder {
    let cal_url = path.into_inner().0;
    let cal_merge = icsutils::db::get_cals_from_url(cal_url + ".ics");
    match cal_merge.first() {
        Some(cal_m) => HttpResponse::Ok()
            .header("Content-Type", "text/calendar")
            .body(merge_calendars(cal_m.clone().calendars).await),
        None => HttpResponse::NotFound().body("No merge configuraion found"),
    }
}



#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let cal_s = format!("{:x}", rand::thread_rng().gen::<u64>()) + ".ics";
    let state_url = web::Data::new(AppState { cal_url: cal_s });

    HttpServer::new(move || {
        App::new()
            .app_data(state_url.clone())
            .service(index)
            .service(init)
            .service(readdb)
            .service(serve_ics)
            .service(create_cal)
            .service(get_cal)
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
