use actix_web::{
    error,
    http::{header::ContentType, StatusCode},
    web, HttpResponse, Responder,
};
use derive_more::{Display, Error};
use regex::Regex;
use sha3::{Digest, Keccak256};

pub mod structs;
use crate::structs::{AbiMethod, Request, Response, Transaction};

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Display, Error)]
pub enum Error {
    #[display(fmt = "internal error")]
    InternalError,

    #[display(fmt = "bad request")]
    BadClientData,

    #[display(fmt = "not found")]
    NotFound,
}

impl From<reqwest::Error> for Error {
    fn from(_: reqwest::Error) -> Self {
        Error::NotFound
    }
}

impl From<serde_json::Error> for Error {
    fn from(_: serde_json::Error) -> Self {
        Error::BadClientData
    }
}

impl From<actix_web::Error> for Error {
    fn from(_: actix_web::Error) -> Self {
        Error::InternalError
    }
}

impl error::ResponseError for Error {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::html())
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            Error::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            Error::BadClientData => StatusCode::BAD_REQUEST,
            Error::NotFound => StatusCode::NOT_FOUND,
        }
    }
}

async fn get_txn_input(txn_hash: &String) -> Result<String> {
    let res = reqwest::get(format!(
        "https://blockscout.com/eth/mainnet/api?module=transaction&action=gettxinfo&txhash={}",
        txn_hash
    ))
    .await?;

    let res: Transaction = res.json().await?;

    Ok(res.result.input)
}

async fn find_abi_method_by_txn_input(input: &str, methods: &Vec<AbiMethod>) -> Result<AbiMethod> {
    if input.len() >= 10 {
        for method in methods {
            let hex = get_hex(&method.function_signature())?;
            if input[2..10].eq(&hex[..8]) {
                return Ok(method.clone());
            }
        }
    }
    Err(Error::NotFound)
}

fn get_hex(input: &String) -> Result<String> {
    let mut hasher = Keccak256::new();
    hasher.update(input.as_bytes());
    let res = hasher.finalize();
    Ok(format!("{:x}", res))
}

fn find_method_in_contract(contract: &str, method: &AbiMethod) -> Result<u32> {
    let signature = method.function();
    let mut start = 0;
    let re = Regex::new(r"(calldata|memory|storage| |\n)").unwrap();

    loop {
        start = contract[start..]
            .find(&format!("function {}", method.name))
            .ok_or(Error::BadClientData)?;
        let end = contract[start..].find(')').ok_or(Error::NotFound)? + start;

        if re.replace_all(&contract[(start + "function ".len())..end + 1], "") == signature {
            break;
        }
        start = end;
    }

    let line_number = (contract[..start].chars().filter(|x| *x == '\n').count() + 1) as u32;
    Ok(line_number)
}

pub async fn index(req: web::Json<Request>) -> Result<impl Responder> {
    let txn_input = get_txn_input(&req.txn).await?;
    let method = find_abi_method_by_txn_input(&txn_input, &req.abi).await?;
    let line_number = find_method_in_contract(&req.contract, &method)?;
    let response = Response {
        method,
        line_number,
    };
    Ok(web::Json(response))
}
