// #![deny(warnings)]
//! # Merging multiple ics calendars into one with privacy options
//! This project is aimed to allow sharing multiple calendar coming from different sources in one single ics file.
//! I wrote this program because I use multiple calendars and often need to share my agendas with friends and colleagues
//! without sharing all details on all calendars. I also want to be able to make distinction between calendars.
use log::{error, info};

use actix_files::Files;
use actix_files::NamedFile;

use actix_web::client::Client;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};

use icsutils::db;

use icsutils::db::MergeConf;
use icsutils::*;

use bcrypt::{hash, verify};

use rand::Rng;
use serde::Deserialize;

/// Helper to get each calendar .ics file using actix-web client
async fn get_http_request(url: &str) -> String {
    let client = Client::default();
    match client.get(url).send().await {
        Ok(mut resp) => match resp.body().limit(102400000).await {
            Ok(r) => String::from_utf8(r.to_vec()).unwrap_or_default(),
            Err(e) => {
                error!("{:?}", e);
                String::new()
            }
        },
        Err(e1) => {
            error!("{:?}", e1);
            String::new()
        }
    }
}

/// ### Add a new configuration struct
/// Each configuration is identified by its url.
/// The format of the config structure is:
/// ```
/// {
///   "name": "Calendarr Name",
///   "url": "calendar_url.ics",
///   "calendars": [
///     {
///       "name": "Cal_1_Name",
///       "is_private": true,
///       "ics_url": "https://calendar.google.com/calendar/ical/adresse00gmail.com/private-tititatatoto/basic.ics"
///     },
///     {
///       "name": "Cal_2_Name",
///       "is_private": false,
///       "ics_url": "https://calendar.google.com/calendar/ical/adresse10gmail.com/private-tititatatoto/basic.ics"
///     },
///     {
///       "name": "Cal_3_Name",
///       "is_private": true,
///       "ics_url": "https://calendar.google.com/calendar/u/1?cid=xyzxyzxyzxyzxyzxyzxyz"
///     }
///   ]
/// }
/// ```
///
/// You can use the provided template to create your own file and use the following command line to add the configuration:
///
/// ```
/// curl -X POST -H "Content-Type: application/json" \
///     -d @calendars.json http://localhost:8080/api/create_cal
/// ```
///
async fn create_cal(cal: web::Json<icsutils::db::MergeConf>) -> impl Responder {
    let mut cal_struct = icsutils::db::MergeConf {
        name: cal.name.to_string(),
        url: cal.url.to_string(),
        password: String::new(),
        calendars: cal.calendars.to_vec(),
    };
    if cal_struct.url == "" {
        cal_struct.url = format!("{:x}", rand::thread_rng().gen::<u64>()) + ".ics";
    }
    cal_struct.password = match hash(cal.password.to_string(), 4) {
        Ok(h) => h,
        Err(_) => panic!(),
    };
    let doc: String = serde_json::to_string(&cal_struct).unwrap();
    match db::insert_cal(cal.url.to_string(), doc) {
        Ok(r) => HttpResponse::Created().body(r),
        Err(err) => {
            error!("{:?}", err);
            HttpResponse::BadRequest().body(format!("Error /{:?}", err))
        }
    }
}

/// Reinitialise the DB
async fn init() -> impl Responder {
    match db::init_db() {
        Ok(msg) => HttpResponse::Ok().body(msg),
        Err(err) => {
            error!("Unable to initialize the DB");
            error!("{:?}", err);
            HttpResponse::BadRequest().body(err)
        }
    }
}

/// Returns all configuration structures stored in the db
async fn list_db() -> impl Responder {
    HttpResponse::Ok().json(serde_json::to_value(&db::get_cals_from_db()).unwrap())
}

/// Returns a specific configuration struct
async fn get_cal(path: web::Path<(String,)>, info: web::Query<AuthRequest>) -> impl Responder {
    match &info.password {
        Some(pwd) => {
            let cal_url = path.into_inner().0;
            let cal_doc = db::get_cal_from_url(&cal_url);
            match cal_doc {
                Some(cal) => {
                    let cal_m: MergeConf = serde_json::from_str(&cal).unwrap();
                    match verify(pwd, &cal_m.password) {
                        Ok(valid) => match valid {
                            true => HttpResponse::Ok().body(&cal),
                            false => HttpResponse::BadRequest().body("Wrong Password"),
                        },
                        Err(_) => HttpResponse::BadRequest().body("Password check Error"),
                    }
                }
                None => HttpResponse::NotFound().body("No merge config struct found"),
            }
        }
        None => {
            HttpResponse::BadRequest().body("A password must be provided to create a new structure")
        }
    }
}

/// Delete a specific configuration struct
async fn delete_cal(path: web::Path<(String,)>, info: web::Query<AuthRequest>) -> impl Responder {
    match &info.password {
        Some(pwd) => {
            let cal_url = path.into_inner().0;
            let cal_doc = db::get_cal_from_url(&cal_url);
            match cal_doc {
                Some(cal) => {
                    let cal_m: MergeConf = serde_json::from_str(&cal).unwrap();
                    match verify(pwd, &cal_m.password) {
                        Ok(valid) => match valid {
                            true => match db::delete_calmerge(&cal_url) {
                                Some(url) => {
                                    HttpResponse::Ok().body(format!("Calendar {} deleted", url))
                                }
                                None => {
                                    HttpResponse::NotFound().body("No merge config struct found")
                                }
                            },
                            false => HttpResponse::BadRequest().body("Wrong Password"),
                        },
                        Err(_) => HttpResponse::BadRequest().body("Password check Error"),
                    }
                }
                None => HttpResponse::NotFound().body("No merge config struct found"),
            }
        }
        None => {
            HttpResponse::BadRequest().body("A password must be provided to create a new structure")
        }
    }
}

/// Returns the merged ics file
async fn serve_ics(path: web::Path<(String,)>) -> impl Responder {
    let cal_url = path.into_inner().0;
    let cal_merge = db::get_cal_from_url(&(cal_url + ".ics"));
    match cal_merge {
        Some(cal_doc) => {
            let cal_m: MergeConf = serde_json::from_str(&cal_doc).unwrap();
            HttpResponse::Ok()
                .content_type("text/calendar")
                .body(merge_calendars(cal_m.clone()).await)
        }
        None => HttpResponse::NotFound().body("No merge config struct found"),
    }
}

/// Process all calendars and merge them in the same String.
async fn merge_calendars(merge_conf: MergeConf) -> String {
    let mut resp = String::from(BEGIN_VCALENDAR);
    resp.push_str(NEW_LINE);
    resp.push_str(X_WR_CALNAME);
    resp.push_str(&merge_conf.name);
    resp.push_str(NEW_LINE);

    for cal in &merge_conf.calendars {
        let ics_content = get_http_request(&cal.ics_url).await;
        info!("Calendar : {} fetched", &cal.name);
        resp.push_str(&parse_calendar_content(cal.clone(), ics_content));
    }

    resp.push_str(END_VCALENDAR);
    resp
}

/// Redirect / to /index.html
async fn index() -> NamedFile {
    NamedFile::open("website/index.html").unwrap()
}

#[derive(Deserialize)]
pub struct AuthRequest {
    password: Option<String>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(move || {
        App::new()
            .service(
                web::scope("/api")
                    .service(web::resource("/init_db").route(web::get().to(init)))
                    .service(web::resource("/list_db").route(web::get().to(list_db)))
                    .service(web::resource("/create_cal").route(web::post().to(create_cal)))
                    .service(
                        web::resource("/delete_cal/{cal_url}").route(web::get().to(delete_cal)),
                    )
                    .service(web::resource("/get_cal/{cal_url}").route(web::get().to(get_cal))),
            )
            .service(web::resource("/").route(web::get().to(index)))
            .service(web::resource("/{cal_url}.ics").route(web::get().to(serve_ics)))
            .service(Files::new("/", "website/").show_files_listing())
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
