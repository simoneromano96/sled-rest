use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use bytes::Bytes;
use sled::{Config, Db, Result};
use std::sync::Arc;

struct AppData {
    db: Db,
}

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

fn init_db() -> Result<Db> {
    let config: Config = Config::default()
        .path("./data".to_owned())
        .cache_capacity(u64::MAX);

    config.open()
}

#[actix_web::put("/{key}")]
async fn create(
    key: web::Path<String>,
    data: web::Data<Arc<AppData>>,
    body: Bytes,
) -> impl Responder {
    let db: Db = data.db.clone();
    let value: &[u8] = body.as_ref();
    // let v1 = b"v1".to_vec();

    db.insert(key.clone(), value).unwrap();
    HttpResponse::Ok().body(body)
}

#[actix_web::get("/{key}")]
async fn get_key(key: web::Path<String>, data: web::Data<Arc<AppData>>) -> impl Responder {
    let db: Db = data.db.clone();
    let key_vec = key.clone().into_bytes();
    let result = db.get(key_vec).unwrap();

    if let Some(value) = result {
        // let deserialised: String = std::str::from_utf8(&*value).unwrap().to_string();
        HttpResponse::Ok().body(format!("{:?}", std::str::from_utf8(&*value).unwrap()))
    } else {
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

    HttpServer::new(move || {
        App::new()
            .data(data.clone())
            .service(get_key)
            .service(create)
    })
    .bind("127.0.0.1:8088")?
    .run()
    .await
}
