use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::fmt::format;
use std::fs::{DirEntry, read};
use std::io;
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

#[derive(Serialize)]
struct DocumentResponse {
    body: Value,
    status: &'static str,
}

impl DocumentResponse
{
    pub fn from_Map(result: Map<String, Value>) -> Self {
        Self {
            body: json!(result),
            status: "ok",
        }
    }

    pub fn from_HashMap(result: HashMap<String, Value>) -> Self {
        Self {
            body: json!(result),
            status: "ok",
        }
    }
}

#[derive(Debug, Deserialize)]
struct QueryParams {
    q: String,
    skipIndex: Option<bool>,
}

impl QueryParams {
    fn matchDocument(&self, document: &HashMap<String, Value>) -> Result<bool, String> {
        let queryComparisons = parseQuery(self.q.as_bytes())?;
        for queryComparison in &queryComparisons {}

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
            let db = db.into_inner();
            match index(&uuidString, &document.0, &db) {
                Ok(_) => {
                    match db.clone().write_document(&uuidString, content) {
                        Ok(_) => {
                            HttpResponse::Ok()
                                .json(DocumentResponse::from_HashMap(HashMap::from([(String::from("id"), Value::String(uuidString))])))
                        }
                        Err(e) => {
                            println!("{:?}", e);
                            HttpResponse::InternalServerError()
                                .finish()
                        }
                    }
                }
                Err(e) => {
                    println!("{}", e);
                    return HttpResponse::InternalServerError()
                        .finish();
                }
            }
        }
        Err(_) => HttpResponse::BadRequest()
            .finish()
    }
}

#[get("/docs/{id}")]
pub async fn getById(req: HttpRequest, id: web::Path<String>, db: web::Data<DB>) -> impl Responder {
    match db.getById(&id.into_inner()) {
        Ok(Some(document)) => HttpResponse::Ok().json(DocumentResponse::from_Map(document)),
        Ok(None) => HttpResponse::NotFound().finish(),
        Err(e) => {
            println!("{:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

fn do_scan(db: Data<DB>, documentResultsVec: &mut Vec<DocumentResult>, queryComparisons: &Vec<QueryComparison>) -> Result<(), String> {
    println!("doing a scan");
    for documents in db.documents().into_iter()
        .filter(|e| {
            e.is_ok_and(|dirEntry| !dirEntry.path().is_dir())
        }) {
        let result = documents
            .map_err(|e| e.into_io_error().unwrap())
            .and_then(|filePath| {
                std::fs::read(filePath.path()).and_then(|bytes| {
                    serde_json::from_str::<Document>(unsafe { std::str::from_utf8_unchecked(&bytes) }).and_then(|document| {
                        let is_match = queryComparisons.iter().all(|queryComparison| queryComparison.matches_document(&document));
                        if (is_match) {
                            let documentResult = DocumentResult {
                                id: String::from(filePath.path().file_name().unwrap().to_str().unwrap()),
                                body: json!(document),
                            };
                            documentResultsVec.push(documentResult)
                        }
                        Ok(())
                    })
                }.map_err(|e| e.into()))
            });

        if let Err(e) = result {
            return Err(format!("{:?}", e));
        }
    }
    Ok(())
}
#[get("/docs")]
pub async fn getDoc(req: HttpRequest, query: web::Query<QueryParams>, db: web::Data<DB>) -> impl Responder {
    let mut documentsResults: Vec<DocumentResult> = Vec::new();
    let mut equalQueryCount = 0;
    let mut documentIdMatches: HashMap<String, u32> = HashMap::new();

    match parseQuery(query.q.as_bytes()) {
        Ok(queryComparisons) => {
            if let Some(true) = &query.skipIndex {
                println!("skipping index");
                if let Err(e) = do_scan(db, &mut documentsResults, &queryComparisons) {
                    println!("{:?}", e);
                    return HttpResponse::InternalServerError().finish();
                }

            } else {

                // get elements that match index
                let mut matchedIds: Vec<String> = vec![];
                for comp in &queryComparisons {
                    match comp.op {
                        QueryOp::Equal => {
                            equalQueryCount += 1;
                            let index_key = format!("{}={}", comp.key.join("."), comp.value);
                            let matchedIds = match db.get_indexed_document(&index_key) {
                                Ok(idsCommaString) => {
                                    for documentId in idsCommaString.split(",") {
                                        // for each document Id we find for the equal filer, add to the map
                                        // and increment how many times we've found this id
                                        match documentIdMatches.get_mut(documentId) {
                                            None => {
                                                documentIdMatches.insert(documentId.to_string(), 1);
                                            }
                                            Some(id) => { *id += 1 }
                                        };
                                    }

                                    // now only take the ids that match all equality filters
                                    // i.e take ids were count == equalArguments
                                     matchedIds = documentIdMatches.iter()
                                        .filter(|(key, value)| **value == equalQueryCount)
                                        .map(|(k, v)| k.to_owned())
                                        .collect();

                                }
                                Err(e) => {
                                    println!("could not get indexed document because :{}",e)
                                }
                            };
                        }
                        _ => ()
                    }
                }

                println!("matched ids {:?}", &matchedIds);

                if (matchedIds.is_empty()) {
                    if let Err(e) = do_scan(db.clone(), &mut documentsResults, &queryComparisons) {
                        println!("{:?}", e);
                        return HttpResponse::InternalServerError().finish();
                    }
                } else {
                    let nonEqualQueryComparisons: Vec<&QueryComparison> = queryComparisons
                        .iter()
                        .filter(|queryComparison| {
                            queryComparison.op != QueryOp::Equal
                        }).collect();

                    for id in matchedIds {
                        if let Ok(Some(doc)) = db.getById(&id) {
                            if (nonEqualQueryComparisons.iter().all(|queryConparison| queryConparison.matches_document(&doc))) {
                                let documentResult = DocumentResult {
                                    id: id.to_string(),
                                    body: json!(doc),
                                };
                                documentsResults.push(documentResult)
                            }
                        }
                    }
                }
            }
        }
        Err(e) => {
            return HttpResponse::BadRequest()
                .body(e);
        }
    };
    let res = DocumentResponse::from_HashMap(
        HashMap::from([
            (String::from("count"), Value::Number(Number::from(documentsResults.len()))),
            (String::from("documents"), json!(documentsResults))
        ])
    );

    HttpResponse::Ok().json(res)
}

pub fn index(id: &str, document: &Map<String, Value>, db: &DB) -> Result<(), String> {
    let pathValues = get_path_values(document, String::new());
    for pv in &pathValues {
        match db.get_indexed_document(pv) {
            Ok(mut idsString) => {
                if (idsString.is_empty()) {
                    idsString = format!("{}", id);
                } else {
                    let ids = idsString.split(",");
                    let mut found = false;

                    for exisitingId in ids {
                        if exisitingId == id {
                            found = true
                        }
                    }

                    if (!found) {
                        idsString = format!("{},{}", idsString, id)
                    }
                }
                match db.write_to_index(pv, idsString) {
                    Ok(_) => {}
                    Err(e) => return Err(format!("{:?}", e))
                };
            }
            Err(e) => return Err(e)
        }
    }

    Ok(())
}
