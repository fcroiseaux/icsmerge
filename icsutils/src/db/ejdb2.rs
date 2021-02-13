
use ejdb::{bson, query, Database, DatabaseOpenMode};
use rand::Rng;
use crate::db::*;

pub fn init_db() -> Result<String, String> {
    let db = open_db();
    match db.drop_collection("calendars", true) {
        Ok(_) => Ok("DB initialized".to_string()),
        Err(err) => Err(err.to_string()),
    }
}

fn open_db() -> Database {
    Database::open_with_mode(
        "calendars.db",
        DatabaseOpenMode::default() | DatabaseOpenMode::CREATE,
    )
        .unwrap()
}

pub fn delete_calmerge(_url: &String) -> Option<String> {
    None
}

pub fn get_cals_from_db() -> Vec<MergeConf> {
    get_cals_from_query(query::Q.empty())
}

pub fn get_cals_from_url(url: String) -> Vec<MergeConf> {
    get_cals_from_query(query::Q.field("url").eq(url))
}

fn get_cals_from_query(q: query::Query) -> Vec<MergeConf> {
    let db = open_db();
    let cal_coll = db.collection("calendars").unwrap();
    let doc_list = cal_coll.query(q, query::QH.empty());
    let items = doc_list.find().unwrap();
    items
        .map(|doc| {
            let bson = bson::to_bson(&doc.unwrap()).unwrap();
            let c: MergeConf = bson::from_bson(bson).unwrap();
            c
        })
        .collect()
}

pub fn insert_cal(cal: MergeConf) -> Result<String, String> {
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
        Ok(_r) => Ok(format!("/{}", cal_url)),
        Err(e) => Err(format!("/{:?}", e)),
    }
}
