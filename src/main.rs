// #![deny(warnings)]

use actix_web::client::Client;
use actix_files::Files;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};

use icsutils::db::sled2::*;
use icsutils::*;
use icsutils::db::MergeConf;

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

async fn merge_calendars(merge_conf: MergeConf) -> String {
    let mut resp = String::from(BEGIN_VCALENDAR);
    resp.push_str(NEW_LINE);
    resp.push_str(X_WR_CALNAME);
    resp.push_str(&merge_conf.name);
    resp.push_str(NEW_LINE);

    for cal in &merge_conf.calendars {
        let ics_content = get_http_request(&cal.ics_url).await;
        println!("Calendar : {} fetched", &cal.name);
        resp.push_str(&parse_calendar_content(cal.clone(), ics_content));
    }

    resp.push_str(END_VCALENDAR);
    resp
}



#[get("/")]
async fn index() -> String {
    format!("Doc tp come soon")
}

#[post("/create_cal")]
async fn create_cal(cal: web::Json<icsutils::db::MergeConf>) -> impl Responder {
    let cal_struct = icsutils::db::MergeConf {
        name: cal.name.to_string(),
        url: cal.url.to_string(),
        calendars: cal.calendars.to_vec(),
    };
    match insert_cal(cal_struct) {
        Ok(r) => HttpResponse::Created().body(r),
        Err(e) => HttpResponse::BadRequest().body(format!("Error /{:?}", e)),
    }
}

#[get("/init_db")]
async fn init() -> impl Responder {
    match init_db() {
        Ok(msg) => HttpResponse::Ok().body(msg),
        Err(err) => HttpResponse::BadRequest().body(err),
    }
}

#[get("/list_db")]
async fn readdb() -> String {
    serde_json::to_string(&get_cals_from_db()).unwrap()
}

#[get("/get_cal/{cal_url}")]
async fn get_cal(path: web::Path<(String,)>) -> impl Responder {
    let cal_url = path.into_inner().0;
    let cal_merge = get_cals_from_url(cal_url);
    match cal_merge.first() {
        Some(cal_m) => HttpResponse::Ok().body(serde_json::to_string(cal_m).unwrap()),
        None => HttpResponse::NotFound().body("No merge configuraion found"),
    }
}

#[get("/delete_cal/{cal_url}")]
async fn delete_cal(path: web::Path<(String,)>) -> impl Responder {
    let cal_url = path.into_inner().0;
    match delete_calmerge(&cal_url) {
        Some(url) => HttpResponse::Ok().body(format!("Calendar {} deleted", url)),
        None => HttpResponse::NotFound().body("No merge configuraion found"),
    }
}

#[get("/{cal_url}.ics")]
async fn serve_ics(path: web::Path<(String,)>) -> impl Responder {
    let cal_url = path.into_inner().0;
    let cal_merge = get_cals_from_url(cal_url + ".ics");
    match cal_merge.first() {
        Some(cal_m) => HttpResponse::Ok()
            .header("Content-Type", "text/calendar")
            .body(merge_calendars(cal_m.clone()).await),
        None => HttpResponse::NotFound().body("No merge configuraion found"),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(move || {
        App::new()
 //           .service(index)
            .service(init)
            .service(readdb)
            .service(serve_ics)
            .service(create_cal)
            .service(delete_cal)
            .service(get_cal)
            .service(Files::new("/", "website/").show_files_listing())

    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
