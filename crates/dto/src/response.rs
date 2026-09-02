use domain::errors::DomainError;
use pkg::json_type;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::json;

use crate::{
    httpcode,
    response::Response::{No, Yes},
};

pub trait DTO: Serialize + DeserializeOwned + Clone {}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Failure {
    code: u16,
    error: DomainError,
}
impl Failure {
    pub fn new(error: DomainError) -> Self {
        let code = httpcode::code(&error);
        Self { code, error }
    }
}

impl DTO for Failure {}
json_type!(Failure);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Placeholder(pub u32);
json_type!(Placeholder);
impl DTO for Placeholder {}
impl DTO for serde_json::Value {}

pub trait ToResp {
    fn to_resp(&self) -> Response<serde_json::Value>;
    fn to_result(&self) -> Result<Response<serde_json::Value>, DomainError>;
}

// DTO is guaranteed to be deserializable so unwrap should be safe (?);
fn to_resp<D: DTO>(d: &D) -> Response<serde_json::Value> {
    let serialized = serde_json::to_value(d).map_err(|x| DomainError::Serialize(x.to_string()));
    let resp: Response<serde_json::Value> = match serialized {
        Ok(v) => Response::Yes(v),
        Err(e) => Response::No(Failure::new(e)),
    };
    resp
}
impl<D: DTO> ToResp for Result<D, DomainError> {
    fn to_resp(&self) -> Response<serde_json::Value> {
        match self {
            Ok(x) => to_resp(x),
            Err(x) => Response::No(Failure::new(x.clone())),
        }
    }

    fn to_result(&self) -> Result<Response<serde_json::Value>, DomainError> {
        Ok(self.to_resp())
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "status", content = "data")]
// #[serde(bound(deserialize = "D: serde::de::DeserializeOwned"))]
pub enum Response<D> {
    #[serde(rename = "success")]
    Yes(D),
    #[serde(rename = "failure")]
    No(Failure),
}

impl<D: DTO> Response<D> {
    fn internal_error() -> String {
        let message = json!({
            "status": "failure",
            "data": {
                "code": 503,
                "error": "UnhandledException: failed to deserialize response"
            }
        });
        serde_json::to_string(&message).expect("valid hardcoded json always success")
    }

    pub fn httpcode(&self) -> u16 {
        match self {
            Yes(_) => 200,
            No(x) => x.code,
        }
    }

    pub fn to_body(&self) -> String {
        let body: String = serde_json::to_string(self).unwrap_or_else(|_| Self::internal_error());
        body
    }

    pub fn get(&self) -> Result<D, DomainError> {
        match self {
            Yes(x) => Ok(x.clone()),
            No(x) => Err(x.error.clone()),
        }
    }
}

#[cfg(test)]
mod tests {

    use domain::errors::DomainError::{self, EmptyResponse};
    use serde::{Deserialize, Serialize};

    use crate::response::{Placeholder, ToResp, DTO};

    #[derive(Debug, Serialize, Deserialize, Clone)]
    pub struct Nested {
        a: u32,
        b: String,
        c: Box<Option<Nested>>,
    }
    impl Nested {
        pub fn new(a: u32, b: String, c: Option<Nested>) -> Self {
            Nested {
                a,
                b,
                c: Box::new(c),
            }
        }
    }
    impl DTO for Nested {}

    #[test]
    fn shape_nested_ok() {
        let c1 = Nested::new(10, "xxx".into(), None);
        let c2 = Nested::new(100, "yyy".into(), Some(c1));
        let resp = Ok(c2).to_resp();
        let s = serde_json::to_string(&resp).unwrap();
        let expected =
            r#"{"status":"success","data":{"a":100,"b":"yyy","c":{"a":10,"b":"xxx","c":null}}}"#;
        assert_eq!(s.len(), expected.len());
    }

    #[test]
    fn shape_simple_ok() {
        let value = Ok(Placeholder(200)).to_resp();
        let s = serde_json::to_string(&value).unwrap();
        // length check to bypass inconsistent deserialize-ordering
        assert_eq!(s.len(), r#"{"data":200,"status":"success"}"#.len());
    }

    #[test]
    fn shape_fail() {
        let err: Result<Placeholder, DomainError> = Err(EmptyResponse);
        let s = serde_json::to_string(&err.to_resp()).unwrap();
        // length check to bypass inconsistent deserialize-ordering
        assert_eq!(
            s.len(),
            r#"{"data":{"code":503,"error":"EmptyResponse"},"status":"failure"}"#.len()
        );
    }

    // #[test]
    // fn shape_failsafe() -> () {
    //     let mut c1 = Nested::new(10, "xxx".into(), None);
    //     // let rc = Rc::new(c1);
    //     // c1.c = Box::new(Some(rc.clone().));
    //     let resp = Ok(c1).to_resp();
    //     let body = resp.to_body();
    //     println!("{}", body);
    // }
}
