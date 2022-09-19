use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::fmt::format;
use std::fs::{DirEntry, read};
use std::io;
use std::io::ErrorKind;
use std::option::Iter;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use actix_web::{HttpRequest, HttpResponse, Responder, web, get, post};
use actix_web::dev::ResourcePath;
use actix_web::error;
use actix_web::web::{Data, Json};
use serde_json::{Number, Value};
use crate::db::DB;
use serde_json::json;
use serde::{Deserialize, Serialize};
use crate::lexer::{DocumentResult, get_path_values, parseQuery, QueryComparison, QueryOp};
use crate::db::Document;
use serde_json::Map;
use walkdir::{Error, WalkDir};
use crate::service::{DocumentResponse, QueryParams, Service};

pub async fn greet(req: HttpRequest) -> HttpResponse {
    HttpResponse::Ok().finish()
}

#[post("/docs")]
pub async fn addDoc(req: HttpRequest, document: web::Json<Document>, db: web::Data<DB>) -> impl Responder {
    match Service::addDoc(document,db){
        Ok(response) => {
            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            if e.kind() == ErrorKind::InvalidData {
                HttpResponse::BadRequest().finish()
            } else { HttpResponse::BadRequest().finish() }
        }
    }
}

#[get("/docs/{id}")]
pub async fn getById(req: HttpRequest, id: web::Path<String>, db: web::Data<DB>) -> impl Responder {
    match Service::getDocumentById(&id.into_inner(),db) {
        Ok(doc) =>  HttpResponse::Ok().json(doc),
        Err(e) => {
            println!("{:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
#[get("/docs")]
pub async fn getDoc(req: HttpRequest, query: web::Query<QueryParams>, db: web::Data<DB>) -> impl Responder {
   match  Service::getDocuments(query,db) {
       Ok(Ok(documentResponse)) => HttpResponse::Ok().json(documentResponse),
       Ok(Err(e)) =>  HttpResponse::BadRequest().body(e),
       Err(e) => {
           println!("{:?}", e);
           HttpResponse::InternalServerError().finish()
       }
   }
}
