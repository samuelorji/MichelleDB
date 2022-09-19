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

pub struct Service {}


#[derive(Serialize)]
pub struct DocumentResponse {
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
pub struct QueryParams {
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

impl Service {

    pub fn reIndex(db: &DB) -> Option<String> {
        let db = Arc::new(db);
        for documents in db.documents().into_iter()
            .filter(|e| { if let Ok(dir) = e {!dir.path().is_dir()} else {false} }) {
            let result = documents
                .map_err(|e| e.into_io_error().unwrap())
                .and_then(|filePath| {
                    std::fs::read(filePath.path()).and_then(|bytes| {
                        serde_json::from_str::<Document>(unsafe { std::str::from_utf8_unchecked(&bytes) }).map_err(|e| e.to_string()).and_then(|document| {
                            let file_name = filePath.path().file_name().and_then(|osStr| osStr.to_str()).unwrap();
                            Self::index(file_name, &document, &db)
                        }).map_err(|e|io::Error::from(ErrorKind::InvalidData) )
                    })
                });

            if let Err(e ) = result {
                return Some(format!("{:?}",e))
            }

        }
        None
    }

     fn index(id: &str, document: &Map<String, Value>, db: &DB) -> Result<(), String> {
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
    pub fn addDoc(document: web::Json<Document>, db: web::Data<DB>) -> io::Result<DocumentResponse> {
        let uuid = uuid::Uuid::new_v4();
        match serde_json::to_string(&document) {
            Ok(content) => {
                let uuidString = uuid.to_string();
                let db = db.into_inner();
                match Service::index(&uuidString, &document.0, &db) {
                    Ok(_) => {
                        match db.clone().write_document(&uuidString, content) {
                            Ok(_) => {
                                return Ok(DocumentResponse::from_HashMap(HashMap::from([(String::from("id"), Value::String(uuidString))])))
                            }
                             Err(e) => {
                                println!("{:?}", &e.to_string());
                                 return Err(io::Error::from(e))
                            }
                        }
                    }
                    Err(e) => {
                        println!("{}", &e);
                        return Err(io::Error::from(ErrorKind::Interrupted))
                    }
                }
            }
            Err(e) =>  {
                println!("{}", e.to_string());
                return Err(io::Error::from(ErrorKind::InvalidData))
            }
        }
    }

    pub fn getDocumentById(id : &str, db : web::Data<DB>) -> Result<Option<Document>,String> {
        db.getById(id)
    }

    pub fn getDocuments(query: web::Query<QueryParams>, db: web::Data<DB>) -> Result<Result<DocumentResponse,String>, String> {

        let mut documentsResults: Vec<DocumentResult> = Vec::new();
        let mut equalQueryCount = 0;
        let mut documentIdMatches: HashMap<String, u32> = HashMap::new();
        let skipIndex = query.skipIndex.unwrap_or(false);

        match parseQuery(query.q.as_bytes()) {
            Ok(queryComparisons) => {
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

                if (matchedIds.is_empty() && skipIndex) {
                    if let Err(e) = Service::do_scan(db.clone(), &mut documentsResults, &queryComparisons) {
                        println!("{:?}", &e);
                        return  Err(e);
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
            Err(e) => {
                return Ok(Err(e))
            }
        };
        let result = DocumentResponse::from_HashMap(
            HashMap::from([
                (String::from("count"), Value::Number(Number::from(documentsResults.len()))),
                (String::from("documents"), json!(documentsResults))
            ])
        );

        return Ok(Ok(result))

    }

    fn do_scan(db: Data<DB>, documentResultsVec: &mut Vec<DocumentResult>, queryComparisons: &Vec<QueryComparison>) -> Result<(), String> {
        println!("doing scan");
        for documents in db.documents().into_iter()
            .filter(|e| { if let Ok(dir) = e {!dir.path().is_dir()} else {false} }) {
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

}