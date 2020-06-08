use actix_web::{
    error, web, App, Error as HttpError, HttpRequest, HttpResponse, HttpServer, Responder,
    Result as HttpResult,
};
use bytes::Bytes;
use sled::{Config, Db, Result};
use std::sync::Arc;

struct AppData {
    db: Db,
}

// Just for reference
/*
fn basic(db: Db) -> Result<()> {
    let k = b"k".to_vec();
    let v1 = b"v1".to_vec();
    let v2 = b"v2".to_vec();

    // set and get
    db.insert(k.clone(), v1.clone())?;
    assert_eq!(db.get(&k).unwrap().unwrap(), (v1.clone()));

    // compare and swap
    match db.compare_and_swap(k.clone(), Some(&v1.clone()), Some(v2.clone()))? {
        Ok(()) => println!("it worked!"),
        Err(sled::CompareAndSwapError {
            current: cur,
            proposed: _,
        }) => println!("the actual current value is {:?}", cur),
    }

    // scan forward
    let mut iter = db.range(k.as_slice()..);
    let (k1, v1) = iter.next().unwrap().unwrap();
    assert_eq!(v1, v2.clone());
    assert_eq!(k1, k.clone());
    assert_eq!(iter.next(), None);

    // deletion
    // db.remove(&k)?;

    Ok(())
}
*/

fn init_db() -> Result<Db> {
    let config: Config = Config::default()
        .path("./data".to_owned())
        .cache_capacity(u64::MAX);

    config.open()
}

#[actix_web::put("/{key}")]
async fn create_or_update_value(
    key: web::Path<String>,
    data: web::Data<Arc<AppData>>,
    body: Bytes,
) -> impl Responder {
    // Get a reference to our "DB"
    let db: Db = data.db.clone();
    // From Bytes to bytes, duh
    let value: &[u8] = body.as_ref();
    // Simply put stuff into DB, this could be anything and there is no need to deserialize it
    db.insert(key.clone(), value).unwrap();
    // Send back the body for confirmation
    HttpResponse::Ok().body(body)
}

#[actix_web::get("/{key}")]
async fn read_value(key: web::Path<String>, data: web::Data<Arc<AppData>>) -> impl Responder {
    let db: Db = data.db.clone();
    // Key from String to bytes
    let key_vec = key.clone().into_bytes();
    // Get result
    let result = db.get(key_vec).unwrap();

    // If has result
    if let Some(value) = result {
        // Get that thing as bytes vec (HttpResponse wants ownership of the var)
        // This will allocate some memory but at least it will avoid serialization and deserialization
        // Also why should I deserialize stuff from my DB? It must be clean (should be?)
        let response = value.to_vec();
        // Send it back as that thing
        HttpResponse::Ok().body(response)
    } else {
        // Not found
        HttpResponse::NotFound().body(format!("Key {:?} not found", key))
    }
}

#[actix_web::delete("/{key}")]
async fn delete_value(key: web::Path<String>, data: web::Data<Arc<AppData>>) -> impl Responder {
    let db: Db = data.db.clone();
    // Key from String to bytes
    let key_vec = key.clone().into_bytes();
    // Get result
    let result = db.remove(key_vec).unwrap();
    // If has result
    if let Some(value) = result {
        // Get that thing as bytes vec (HttpResponse wants ownership of the var)
        let response = value.to_vec();
        // Send it back as that thing
        HttpResponse::Ok().body(response)
    } else {
        // Not found
        HttpResponse::NotFound().body(format!("Key {:?} not found", key))
    }
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    // basic(db.clone()).unwrap();
    // println!("{:?}", db);

    let data: Arc<AppData> = Arc::new(AppData {
        db: init_db().unwrap(),
    });

    // TODO: I should only accept JSON or else users will be able to send any kind of shit
    HttpServer::new(move || {
        App::new()
            .data(data.clone())
            .service(create_or_update_value)
            .service(read_value)
            .service(delete_value)
    })
    .bind("127.0.0.1:8088")?
    .run()
    .await
}
