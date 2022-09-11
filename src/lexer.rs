use std::io;
use std::io::Error;

pub fn lexString(input : &[u8], mut index : usize) -> Result<(&str, usize), &str> {
    let mut found_end = false;
    if(index >= input.len()){
        return Err("empty string")
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
            return Err("Expected end of quoted string")
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
        return Err("No string found")
    } else {
        return Ok((s,index))
    }
}


mod test {
    use crate::lexer::lexString;

    #[test]
    fn test_lexing(){
        #[derive(Debug)]
        struct TestCase {
            input : &'static str,
            index: usize,
            expectedResult: Result<(&'static str, usize), &'static str>
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
                expectedResult: Err("No string found")
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
}