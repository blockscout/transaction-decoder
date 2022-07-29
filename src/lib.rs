use actix_web::{
    error,
    http::{header::ContentType, StatusCode},
    web, HttpResponse, Responder,
};
use derive_more::{Display, Error};
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

    let res = res.text().await?;
    let res: Transaction = serde_json::from_str(&res)?;

    Ok(res.result.input)
}

fn get_method_full_string(method: &AbiMethod) -> Result<String> {
    let v = method.inputs.clone();
    let args = v
        .into_iter()
        .map(|x| x.arg_type)
        .collect::<Vec<_>>()
        .join(",");
    Ok(format!("{}({})", method.name, args))
}

async fn find_abi_method_by_txn_input(input: &str, methods: &Vec<AbiMethod>) -> Result<AbiMethod> {
    if input.len() >= 10 {
        for method in methods {
            let hex = get_hex(&get_method_full_string(method)?)?;
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
    let start = contract
        .find(&format!("function {}", method.name))
        .ok_or(Error::NotFound)?;
    let line_number = find_line_number_in_contract(contract, start);
    Ok(line_number)
}

fn find_line_number_in_contract(contract: &str, index: usize) -> u32 {
    (contract[..index].chars().filter(|x| *x == '\n').count() + 1) as u32
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
