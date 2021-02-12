use crate::db::*;
use rand::Rng;

pub fn init_db() {
    let db = open_db();
    db.clear();
}

fn open_db() -> sled::Db {
    sled::open("mergeics_sled_db").unwrap()
}

pub fn get_cals_from_db() -> Vec<CalMerge> {
    Vec::new()
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
