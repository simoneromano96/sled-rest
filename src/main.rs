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
    // Simply put stuff into DB
    data.db.insert(key.clone(), body.as_ref()).unwrap();
    // Send back the body for confirmation
    HttpResponse::Ok().body(body)
}

#[actix_web::get("/{key}")]
async fn read_value(key: web::Path<String>, data: web::Data<Arc<AppData>>) -> impl Responder {
    // Get result
    let result = data.db.get(key.as_bytes()).unwrap();
    // If has result
    if let Some(value) = result {
        // Send back the result
        HttpResponse::Ok().body(value.to_vec())
    } else {
        // Not found
        HttpResponse::NotFound().body(format!("Key {:?} not found", key))
    }
}

#[actix_web::delete("/{key}")]
async fn delete_value(key: web::Path<String>, data: web::Data<Arc<AppData>>) -> impl Responder {
    // Get result
    let result = data.db.remove(key.as_bytes()).unwrap();
    // If has result
    if let Some(value) = result {
        // Send back the result
        HttpResponse::Ok().body(value.to_vec())
    } else {
        // Not found
        HttpResponse::NotFound().body(format!("Key {:?} not found", key))
    }
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    // Create AppData with the DB
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
