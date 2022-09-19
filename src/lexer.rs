use std::collections::HashMap;
use std::io;
use std::io::Error;

use std::str::FromStr;
use serde_json::{Value,Map};
use serde::Serialize;
use crate::Document;

#[derive(PartialEq,Debug)]
pub struct QueryComparison<'a> {
    pub key: Vec<&'a str>,
    pub value :&'a str,
    pub op: QueryOp
}

impl<'a> QueryComparison<'a> {
    pub fn matches_document(&self, document : &Document) -> bool {
        let keys = &self.key;
        let mut result= &Value::Null;
        for key in keys {
            if(result.is_null()) {
                if let Some(inner) =  document.get(*key) {
                    result = inner
                } else {
                    return  false
                }

            } else {
                if let Some(inner) = result.get(*key) {
                    result = inner
                } else {
                    return false
                }
            }
        }
        if(result.is_null()){
            false
        } else {
            let value = &self.value;
            match self.op {
                QueryOp::Equal => {
                    match result {
                        Value::Number(number) => {
                            if let Some(n) = number.as_f64() {
                                f64::from_str(*value).unwrap() == n
                            } else {
                                false
                            }
                        }
                        Value::String(s) => {
                            s == *value
                        }
                        // any other type of value should be false
                        _ => false
                    }
                }
                x => {
                    match result {
                        Value::Number(number) => {
                            if let Some(n) = number.as_f64() {
                                match x {
                                    QueryOp::Greater => {
                                        n > f64::from_str(*value).unwrap()
                                    },
                                    QueryOp::Less => {
                                        n < f64::from_str(*value).unwrap()
                                    },
                                    _ => false
                                }
                            } else {
                                false
                            }
                        },
                        Value::String(s) => {
                            match (f64::from_str(&s),f64::from_str(*value)) {
                                (Ok(a), Ok(b)) => {
                                    match x {
                                        QueryOp::Greater => {
                                            a > b
                                        },
                                        QueryOp::Less => {
                                           a < b
                                        },
                                        _ => false
                                    }
                                }
                                _ => false
                            }

                        },
                        // any other type of value should be false
                        _ => false
                    }
                }
            }

        }
    }
}

#[derive(Serialize)]
pub struct DocumentResult {
    pub id: String,
    pub body: Value
}

#[derive(PartialEq,Debug,Copy,Clone)]
pub enum QueryOp {
    Equal,
    Greater,
    Less
}

pub fn lexString(input : &[u8], mut index : usize) -> Result<(&str, usize), (&str,usize)> {
    let mut found_end = false;
    if(index >= input.len()){
        return Err(("empty string",index))
    }
    let mut s = "";
    if (input[index] == b'"'){
        // handling nested quotes
        index += 1;
        let start = index;

        while(index < input.len()){
            if(input[index] == b'"'){
                found_end = true;
                break
            }
            index +=1;
        }
        s = std::str::from_utf8(&input[start .. index]).unwrap();
        if(!found_end) {
            // if we've not found a quotation ending
            return Err(("Expected end of quoted string",index))
        }

        return Ok((s,index + 1))
    }

    // if unquoted, read as much letters/digits
    let start = index;
    while(index < input.len()){
        let c = input[index];
        if(!(c.is_ascii_alphanumeric() || c == b'.')){
            break
        }
        index+=1;
    }
    s = std::str::from_utf8(&input[start..index]).unwrap();
    if(s.is_empty()){
        return Err(("No string found",index))
    } else {
        return Ok((s,index))
    }
}

pub fn get_path_values(document: &Map<String,Value>, prefix : String) -> Vec<String> {
    let mut pv : Vec<String>= Vec::new();
    fn go(doc: &Map<String,Value>, pref : &String, pv : &mut Vec<String>)  {
        for (key,value) in doc {
            match value {
                Value::Array(_) => {
                    // not supported
                    continue
                }
                Value::Object(o) => {
                    let prefix = if(pref.is_empty()){
                        String::from(key)
                    } else {
                        format!("{}.{}",&pref,&key)
                    };
                    go(o,&prefix,pv)
                }
                _ => {
                    let mut u_key = String::new();
                    if(!pref.is_empty()){
                        u_key = format!("{}.{}",&pref,&key);
                    }
                    else {
                        u_key = key.clone();
                    }

                    if let Some(s) = value.as_str() {
                        pv.push(format!("{}={}",&u_key, s));
                    } else {
                        pv.push(format!("{}={}",&u_key, value));
                    }
                }
            }
        }
    }
    go(&document, &prefix,&mut pv);
    pv

}
pub fn parseQuery<'a>(query : &'a[u8]) -> Result<Vec<QueryComparison<'a>>,  String> {

    // empty query check done in service layer
    let mut  i: usize = 0;
    let mut result = vec![];
    while(i < query.len()){
        // remove all whitespace
        loop {
            if(query[i].is_ascii_whitespace()){
                i +=1
            } else {
                break;
            }
        }

        match lexString(query,i) {
            Ok((key,nextIndex)) => {
                if(query[nextIndex] != b':'){
                    return Err(format!("Expected colon at index : {nextIndex}"))
                }
                i = nextIndex + 1;
                let mut op = QueryOp::Equal;
                match query[i] {
                    b'>' =>{
                        op = QueryOp::Greater;
                        i+=1;
                    }
                    b'<' => {
                        op = QueryOp::Less;
                        i+=1;
                    }
                   _ => (),
                };
                match lexString(query, i) {
                    Ok((value,nextIndex)) => {
                        i = nextIndex;
                        let comp = QueryComparison {
                            key : key.split(".").collect(),
                            value,
                            op
                        };
                       result.push(comp);
                    }
                    Err((e,at)) => {
                        println!("error : {}",e);
                        return Err(format!("Expected valid value for {}, got `{}` instead",&key,unsafe {std::str::from_utf8_unchecked(&query[at..])}))
                    }
                }
            },
            Err((e,at)) => {
                return Err(format!("Expected valid key, got `{:?}` instead",unsafe {std::str::from_utf8_unchecked(&query[at..])}))
            }
        }
    }
    Ok(result)
}


mod test {
    use serde::de::Unexpected::Str;
    use serde_json::{json, Value};
    use super::*;

    #[test]
    fn test_lexing(){
        #[derive(Debug)]
        struct TestCase {
            input : &'static str,
            index: usize,
            expectedResult: Result<(&'static str, usize), (&'static str, usize)>
        }

        let testCases = [
            TestCase{
               input : "a.b:c",
               index:  0,
               expectedResult: Ok(( "a.b", 3))
            },
            TestCase{
                input : "\"a b : . 2\":12`",
                index:  0,
                expectedResult: Ok(( "a b : . 2", 11))
            },
            TestCase{
                input : " a:2",
                index:  0,
                expectedResult: Err(("No string found",0))
            },
            TestCase{
                input : " a:2",
                index:  1,
                expectedResult: Ok(("a",2))
            }
        ];
        for testCase in testCases {
            let result = lexString(testCase.input.as_bytes(),testCase.index);
            assert_eq!(result, testCase.expectedResult)
        }
    }

    #[test]
    fn test_parsing(){
        struct TestCase<'a> {
            query : &'a [u8],
            expectedResult: Result<Vec<QueryComparison<'a>>,String>
        }
        let testCases = [
            TestCase {
                query: "a.b:1 c:>2".as_bytes(),
                expectedResult: Ok(vec![
                    QueryComparison {
                        key: vec!["a","b"],
                        value: "1",
                        op: QueryOp::Equal
                    },
                    QueryComparison {
                        key: vec!["c"],
                        value: "2",
                        op: QueryOp::Greater
                    },
                ])
            },
            TestCase {
                query: "a.c:@2".as_bytes(),
                expectedResult: Err(String::from("Expected valid value for a.c, got `@2` instead"))
            },
        ];


        for testCase in &testCases {
            assert_eq!(parseQuery(testCase.query), testCase.expectedResult)
        }
    }

    #[test]
    fn test_query_comparison_matcher(){
        let queryComparison = QueryComparison {
            key: vec!["a","b"],
            value: "2",
            op: QueryOp::Greater
        };

        let document: Value = json!( {
            "a": {
                "b": 3
            }
            }
        );

        let result = queryComparison.matches_document(document.as_object().unwrap());
        println!("{}",result);

    }

    #[test]
    fn test_get_path_values(){
        let jsonValue  = json!({
            "a": 1,
            "b" : {
                "c" : {
                    "d" : {
                        "e" : 1,
                        "f" : 3
                    }
                },
                "g":"2"
            }
        });

        let parsed = serde_json::from_value::<Map<String,Value>>(jsonValue).unwrap();

        let result = get_path_values(&parsed,String::new());

        let expectedResult: Vec<String> = vec!["a=1", "b.c.d.e=1", "b.c.d.f=3", "b.g=2"].iter().map(|x| String::from(*x)).collect();

        assert_eq!(expectedResult,result);
        let jsonValue  = json!({
            "a": 1,
            "b" : "2",
            "c": {
                "d" : "3"
            }
        });

        let parsed = serde_json::from_value::<Map<String,Value>>(jsonValue).unwrap();

        let result = get_path_values(&parsed,String::new());

        let expectedResult: Vec<String> = vec!["a=1", "b=2", "c.d=3"].iter().map(|x| String::from(*x)).collect();

        assert_eq!(expectedResult,result);
    }
}