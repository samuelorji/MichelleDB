use std::collections::HashMap;
use std::path::PathBuf;
use actix_web::{HttpRequest, HttpResponse, Responder, web, get,post};
use actix_web::web::Json;
use serde::de::Unexpected::Str;
use serde_json::Value;
use crate::db::DB;
use serde_json::json;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct DocumentResponse<V> {
    body: HashMap<String, V>,
    status: &'static str,
}

impl<V> DocumentResponse<V>
    where V: Serialize
{
    pub fn from_Map(result: HashMap<String, V>) -> Self {
        Self {
            body: result,
            status: "ok",
        }
    }
}

pub async fn greet(req: HttpRequest) -> HttpResponse {
    HttpResponse::Ok().finish()
}


#[post("/docs")]
pub async fn addDoc(req: HttpRequest, document: web::Json<HashMap<String, Value>>, db: web::Data<DB>) -> impl Responder {
    let uuid = uuid::Uuid::new_v4();
    match serde_json::to_string(&document) {
        Ok(content) => {
            let uuidString = uuid.to_string();
            std::fs::write(PathBuf::from(&db.dbDir).join(&uuidString), content);
            HttpResponse::Ok()
                .json(DocumentResponse::<String>::from_Map(HashMap::from([(String::from("id"), uuidString)])))
        }
        Err(_) => HttpResponse::BadRequest()
            .finish()
    }
}

#[get("/docs/{id}")]
pub async fn getById(req: HttpRequest, id: web::Path<String>, db: web::Data<DB>) -> impl Responder {
    match db.getById(id.into_inner()) {
        Some(result) => {
            match serde_json::from_str::<HashMap<String, Value>>(&result) {
                Ok(document) => {
                    HttpResponse::Ok()
                        .json(DocumentResponse::<Value>::from_Map(document))
                }
                Err(e) => {
                    println!("{:?}", e);
                    HttpResponse::InternalServerError()
                        .finish()
                }
            }
        }
        None => HttpResponse::NotFound()
            .finish()
    }
}