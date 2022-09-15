use std::collections::HashMap;
use std::path::{Path, PathBuf};
use actix_web::{HttpRequest, HttpResponse, Responder, web, get,post};
use actix_web::dev::ResourcePath;
use actix_web::error;
use actix_web::web::Json;
use serde_json::{Number, Value};
use crate::db::DB;
use serde_json::json;
use serde::{Deserialize, Serialize};
use crate::lexer::{DocumentResult, parseQuery, QueryComparison};
use crate::db::Document;

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
#[derive(Debug,Deserialize)]
struct QueryParams {
    q: String
}

impl QueryParams {
    fn matchDocument(&self, document : &HashMap<String, Value>) -> Result<bool,String> {
        let queryComparisons = parseQuery(self.q.as_bytes())?;
        for queryComparison in &queryComparisons {

        }

        Ok(true)

    }
}

pub async fn greet(req: HttpRequest) -> HttpResponse {
    HttpResponse::Ok().finish()
}


#[post("/docs")]
pub async fn addDoc(req: HttpRequest, document: web::Json<Document>, db: web::Data<DB>) -> impl Responder {
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
            match serde_json::from_str::<Document>(&result) {
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
#[get("/docs")]
pub async fn getDoc(req:HttpRequest, query : web::Query<QueryParams>,db : web::Data<DB>) -> impl Responder {

    let mut documentsResults : Vec<DocumentResult> = Vec::new();
    match parseQuery(query.q.as_bytes()) {
        Ok(queryComparisons) => {

            for documents in db.documents().into_iter()
                .filter(|e| {
                    if let Ok(entry) = e {
                        !Path::new(entry.path()).is_dir()
                    }  else {
                        false
                    }
                }) {

                match documents {
                    Ok(filePath) => {
                        match   std::fs::read(filePath.path()) {
                            Ok(bytes) => {
                                match serde_json::from_str::<Value>(unsafe { std::str::from_utf8_unchecked(&bytes)}) {
                                    Ok(document) => {
                                        let  is_match = queryComparisons.iter().all(|queryComparison| queryComparison.matches_document(&document));
                                        if(is_match) {
                                            let documentResult = DocumentResult {
                                                id: String::from(filePath.path().file_name().unwrap().to_str().unwrap()),
                                                body: document
                                            };
                                            documentsResults.push(documentResult)
                                        }
                                    }
                                    Err(e) => {
                                        println!("Serde failed: {:?}",e);
                                        return HttpResponse::InternalServerError().finish()
                                    }
                                }
                            }
                            Err(e) => {
                                println!("File read failed: {:?}",e);
                                return HttpResponse::InternalServerError().finish()
                            }
                        }
                    }
                    Err(e) => {
                        println!("can't open directory {:?}",e);
                       return HttpResponse::InternalServerError().finish()
                    }
                }
            }

        }
        Err(e) => {
           return HttpResponse::BadRequest()
                .body(e)
        }
    };
   let res =  DocumentResponse::<Value>::from_Map(
       HashMap::from([
           (String::from("count"), Value::Number(Number::from(documentsResults.len()))),
           (String::from("documents"), json!(documentsResults))
       ])
   );

    HttpResponse::Ok().json(res)
}