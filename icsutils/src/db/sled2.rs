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

pub fn get_cals_from_db() -> String {
    let db = open_db();
    db.iter()
        .values()
        .map(|ivec| String::from_utf8(ivec.unwrap().to_vec()).unwrap())
        .collect()
}

pub fn get_cal_from_url(url: &str) -> Option<String> {
    let db = open_db();
    match db.get(&url) {
        Ok(result) => result.map(| ivec | String::from_utf8(ivec.to_vec()).unwrap()),
        Err(_) => None
    }
}

pub fn delete_calmerge(url: &String) -> Option<String> {
    let db = open_db();
    match db.remove(url).unwrap() {
        Some(_) => Some(url.to_string()),
        None => None,
    }
}

pub fn insert_cal(url: String, doc: String) -> Result<String, String> {
    let db = open_db();
    match db.insert(url.clone(), doc.as_bytes()) {
        Ok(_r) => Ok(format!("/{}", url)),
        Err(e) => Err(format!("/{:?}", e)),
    }
}
