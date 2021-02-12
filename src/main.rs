// #![deny(warnings)]

use actix_web::client::Client;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use rand::Rng;
use serde::{Deserialize, Serialize};

use icsutils::*;

use ejdb::{bson, query, Database, DatabaseOpenMode};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct IcsCal {
    //The list of ical urls we want to merge
    name: String,
    ics_url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
struct CalMerge {
    //The list of ical urls we want to merge
    name: String,
    url: String,
    calendars: Vec<IcsCal>,
}

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

async fn ics_merge() -> impl Responder {
    // calendars is an Array of tuples in the form of [("cal1 name", "cal1_url"),("cal1 name", "cal1_url")]
    // next step is to replace with json file content
    let calendars: [(&str, &str); 0] = [];

    let mut resp = String::from(BEGIN_VCALENDAR);
    resp.push_str(NEW_LINE);

    for cal in &calendars {
        let ics_content = get_http_request(cal.1).await;
        println!("Calendar : {} fetched", cal.0);
        resp.push_str(&parse_calendar_content(cal.0, ics_content));
    }

    resp.push_str(END_VCALENDAR);
    HttpResponse::Ok()
        .header("Content-Type", "text/calendar")
        .body(resp)
}

async fn merge_calendars(calendars: Vec<IcsCal>) -> String {
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
async fn create_cal(cal: web::Json<CalMerge>) -> String {
    let db = open_db();
    let cal_coll = db.collection("calendars").unwrap();
    let cal_url = match cal.url.as_str() {
        "" => format!("{:x}", rand::thread_rng().gen::<u64>()) + ".ics",
        url_s => url_s.to_string(),
    };

    let mut d = bson::Document::new();
    d.insert("name", cal.name.to_string());
    d.insert("url", cal_url.to_string());
    d.insert("calendars", bson::to_bson(&cal.calendars.to_vec()).unwrap());
    match cal_coll.save(d) {
        Ok(_r) => format!("/{}", cal_url),
        Err(e) => format!("Error /{:?}", e),
    }
}

#[get("/init")]
async fn init() -> String {
    let db = open_db();
    db.drop_collection("calendars", true).unwrap();
    "DONE DB initialized".to_string()
}

#[get("/readdb")]
async fn readdb() -> String {
    serde_json::to_string(&get_cals_from_db()).unwrap()
}

#[get("/get_cal/{cal_url}")]
async fn get_cal(path: web::Path<(String,)>) -> String {
    let url = path.into_inner().0;
    serde_json::to_string(&get_cals_from_url(url)).unwrap()
}

#[get("/{cal_url}.ics")]
async fn serve_ics(path: web::Path<(String,)>) -> impl Responder {
    let cal_url = path.into_inner().0;
    let cal_merge = get_cals_from_url(cal_url + ".ics");
    match cal_merge.first() {
        Some(cal_m) => HttpResponse::Ok()
            .header("Content-Type", "text/calendar")
            .body(merge_calendars(cal_m.clone().calendars).await),
        None => HttpResponse::NotFound().body("No merge configuraion found"),
    }
}

fn open_db() -> Database {
    Database::open_with_mode(
        "calendars.db",
        DatabaseOpenMode::default() | DatabaseOpenMode::CREATE,
    )
    .unwrap()
}

fn get_cals_from_db() -> Vec<CalMerge> {
    get_cals_from_query(query::Q.empty())
}

fn get_cals_from_url(url: String) -> Vec<CalMerge> {
    get_cals_from_query(query::Q.field("url").eq(url))
}

fn get_cals_from_query(q: query::Query) -> Vec<CalMerge> {
    let db = open_db();
    let cal_coll = db.collection("calendars").unwrap();
    let doc_list = cal_coll.query(q, query::QH.empty());
    let items = doc_list.find().unwrap();
    items
        .map(|doc| {
            let bson = bson::to_bson(&doc.unwrap()).unwrap();
            let c: CalMerge = bson::from_bson(bson).unwrap();
            c
        })
        .collect()
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
            .route(&state_url.cal_url, web::get().to(ics_merge))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
