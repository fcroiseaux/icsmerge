use crate::db::*;
use rand::Rng;

pub fn init_db() -> Result<String, String> {
    let db = open_db();
    match db.clear() {
        Ok(_)=> Ok("DB initialized".to_string()),
        Err(e) => Err(e.to_string())
    }
}

fn open_db() -> sled::Db {
    sled::open("mergeics_sled_db").unwrap()
}

pub fn get_cals_from_db() -> Vec<CalMerge> {
    let db = open_db();
    db.iter()
        .values()
        .map(|ivec| {
            let cal_doc = String::from_utf8(ivec.unwrap().to_vec()).unwrap();
            serde_json::from_str(&cal_doc).unwrap()
        })
        .collect()
}

pub fn get_cals_from_url(url: String) -> Vec<CalMerge> {
    let db = open_db();
    match db.get(url).unwrap() {
        Some(ivec) => {
            let cal_doc = String::from_utf8(ivec.to_vec()).unwrap();
            let cal_m: CalMerge = serde_json::from_str(&cal_doc).unwrap();
            vec![cal_m]
        }
        None => vec![],
    }
}

pub fn delete_calmerge(url: &String) -> Option<String> {
    let db = open_db();
    match db.remove(url).unwrap() {
        Some(_) => Some(url.to_string()),
        None => None,
    }
}

pub fn insert_cal(mut cal: CalMerge) -> Result<String, String> {
    let db = open_db();
    if cal.url == "" {
        cal.url = format!("{:x}", rand::thread_rng().gen::<u64>()) + ".ics";
    }

    let doc: String = serde_json::to_string(&cal).unwrap();
    match db.insert(cal.url.clone(), doc.as_bytes()) {
        Ok(_r) => Ok(format!("/{}", cal.url)),
        Err(e) => Err(format!("/{:?}", e)),
    }
}
