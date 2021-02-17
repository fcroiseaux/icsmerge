// #![deny(warnings)]
//! # Merging multiple ics calendars into one with privacy options
//! This project is aimed to allow sharing multiple calendar coming from different sources in one single ics file.
//! I wrote this program because I use multiple calendars and often need to share my agendas with friends and colleagues
//! without sharing all details on all calendars. I also want to be able to make distinction between calendars.
use clap::clap_app;
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

struct AppPassword {
    root_password: String,
}

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
async fn init(root: web::Data<AppPassword>, info: web::Query<AuthRequest>) -> impl Responder {
    fn init_the_db() -> HttpResponse {
        match db::init_db() {
            Ok(msg) => HttpResponse::Ok().body(msg),
            Err(err) => {
                error!("Unable to initialize the DB");
                error!("{:?}", err);
                HttpResponse::BadRequest().body(err)
            }
        }
    }
    check_root_password_and_apply(root, info, init_the_db)
}

/// Returns all configuration structures stored in the db
/// Not used for privacy reasons but kept as an example of how to list all the db
async fn list_db(root: web::Data<AppPassword>, info: web::Query<AuthRequest>) -> impl Responder {
    fn list_ok() -> HttpResponse {
        let cals: String = db::get_cals_from_db();
        HttpResponse::Ok().body(cals)
    }
    check_root_password_and_apply(root, info, list_ok)
}

fn check_root_password_and_apply(
    root: web::Data<AppPassword>,
    info: web::Query<AuthRequest>,
    f: fn() -> HttpResponse,
) -> impl Responder {
    match &info.password {
        Some(pwd) => {
            if pwd == &root.root_password {
                f()
            } else {
                HttpResponse::BadRequest().body("Wrong Root password")
            }
        }
        None => {
            HttpResponse::BadRequest().body("Root password must be provided to list all the db")
        }
    }
}

fn check_password_and_apply(
    path: web::Path<(String,)>,
    info: web::Query<AuthRequest>,
    f: fn(String, String) -> HttpResponse,
) -> impl Responder {
    match &info.password {
        Some(pwd) => {
            let cal_url = path.into_inner().0;
            let cal_doc = db::get_cal_from_url(&cal_url);
            match cal_doc {
                Some(cal) => {
                    let cal_m: MergeConf = serde_json::from_str(&cal).unwrap();
                    match verify(pwd, &cal_m.password) {
                        Ok(valid) => match valid {
                            true => f(cal_url, cal),
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

/// Returns a specific configuration struct
async fn get_cal(path: web::Path<(String,)>, info: web::Query<AuthRequest>) -> impl Responder {
    fn ok_json(_url: String, cal: String) -> HttpResponse {
        HttpResponse::Ok().body(&cal)
    }
    check_password_and_apply(path, info, ok_json)
}

/// Delete a specific configuration struct
async fn delete_cal(path: web::Path<(String,)>, info: web::Query<AuthRequest>) -> impl Responder {
    fn del_cal(url: String, _cal: String) -> HttpResponse {
        match db::delete_calmerge(&url) {
            Some(url) => HttpResponse::Ok().body(format!("Calendar {} deleted", url)),
            None => HttpResponse::NotFound().body("No merge config struct found"),
        }
    }
    check_password_and_apply(path, info, del_cal)
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
    let app = clap_app!(icsmerge =>
        (version: "0.5")
        (author: "Fabrice Croiseaux. <fabrice@hackvest.com>")
        (about: "Web server that allow merging multiple .ics calenars into one with privacy options.")
        (@arg ROOT_PASSWORD: -p --root_password +required +takes_value "Set the root password")
    );

    let matches = app.clone().get_matches();
    let password = matches.value_of("ROOT_PASSWORD");

    if password.is_some() {
        println!("{} web server started", app.render_long_version());
    }

    let root_pwd = password.unwrap();

    let state_pwd = web::Data::new(AppPassword {
        root_password: root_pwd.to_string(),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(state_pwd.clone())
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
