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

#[derive(Serialize, Deserialize, Debug, Clone)]
struct CalMerge {
    //The list of ical urls we want to merge
    name: String,
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
    let cal_struct: CalMerge = CalMerge {
        name: cal.name.to_string(),
        calendars: cal.calendars.to_vec(),
    };
    let mut d = bson::Document::new();
    d.insert("name", cal_struct.name);
    d.insert("calendars", bson::to_bson(&cal_struct.calendars).unwrap());
    match cal_coll.save(d) {
        Ok(r) => format!("Saved /{:?}", r),
        Err(e) => format!("Error /{:?}", e),
    }
}

#[get("/init")]
async fn init() -> String {
    let db = open_db();
    db.drop_collection("calendars",true).unwrap();
    let cal_coll = db.collection("calendars").unwrap();
    cal_coll
        .save(bson! {
            "name" => "Fabrice",
            "calendars" => [{"name" => "InTech", "ics_url" => "12345"}, {"name" => "Lumena", "ics_url" => "567890"}]
        })
        .unwrap();
    "OK".to_string()
}

#[get("/readdb")]
async fn readdb() -> String {
    format!("{:?}", get_cals_from_db())
}

#[get("/{url_ics}.ics")]
async fn serve_ics(path: web::Path<(String,)>) -> String {
    format!("{:?}", path.into_inner().0)
}

fn open_db() -> Database {
    Database::open_with_mode(
        "calendars.db",
        DatabaseOpenMode::default() | DatabaseOpenMode::CREATE,
    ).unwrap()
}

fn get_cals_from_db() -> Vec<CalMerge> {
    let db = open_db();
    let cal_coll = db.collection("calendars").unwrap();
    let doc_list = cal_coll.query(query::Query::new(), query::QH.empty());
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
            .route(&state_url.cal_url, web::get().to(ics_merge))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
