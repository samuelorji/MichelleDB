use std::collections::HashMap;
use actix_web::{HttpRequest, HttpResponse, Responder, web};
use serde_json::Value;

pub async fn greet(req: HttpRequest) -> HttpResponse {
    HttpResponse::Ok().finish()
}

pub async fn addDoc(req: HttpRequest, document: web::Json<HashMap<String,Value>>) -> impl Responder {
    web::Json(document)
}