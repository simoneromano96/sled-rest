use actix_web::{
    error, web, App, Error as HttpError, HttpRequest, HttpResponse, HttpServer, Responder,
    Result as HttpResult,
};
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use serde_json;
use sled::{Config, Db, Result};
use std::sync::Arc;

struct AppData {
    db: Db,
}

#[derive(Serialize, Deserialize)]
struct Collection {
    key: String,
}

fn init_db() -> Result<Db> {
    let config: Config = Config::default()
        .path("./data".to_owned())
        .cache_capacity(u64::MAX);

    config.open()
}

#[actix_web::post("/collections")]
async fn create_collection(
    json: web::Json<Collection>,
    data: web::Data<Arc<AppData>>,
) -> impl Responder {
    let collections = data.db.get("collections").unwrap();
    if let Some(value) = collections {
        HttpResponse::Ok().body("something")
    } else {
        let collection = [json.into_inner()];
        let serialized: String = serde_json::to_string(&collection).unwrap();
        data.db.insert("collections", serialized.as_bytes()).unwrap();
        HttpResponse::Ok().body(serialized)
    }
}

#[actix_web::get("/collections")]
async fn get_collections(data: web::Data<Arc<AppData>>) -> impl Responder {
    // Get result
    let result = data.db.get("collections").unwrap();
    if let Some(value) = result {
        // Send back the result
        HttpResponse::Ok().body(value.to_vec())
    } else {
        // Not found
        HttpResponse::NotFound().body("No collections in db")
    }
}

#[actix_web::put("/{key}")]
async fn create_or_update_value(
    key: web::Path<String>,
    data: web::Data<Arc<AppData>>,
    body: Bytes,
) -> impl Responder {
    // Simply put stuff into DB
    data.db.insert(key.as_bytes(), body.as_ref()).unwrap();
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
