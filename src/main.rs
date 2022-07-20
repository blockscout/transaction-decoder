use actix_web::{error, web, App, HttpServer, Responder};
use derive_more::{Display, Error};
use sha3::{Digest, Keccak256};

use reqwest;

pub mod structs;
use crate::structs::{AbiMethod, Request, Response, Transaction};

type Result<T> = std::result::Result<T, MyError>;
#[derive(Debug, Display, Error)]
#[display(fmt = "my error: {}", name)]
struct MyError {
    name: &'static str,
}

impl From<reqwest::Error> for MyError {
    fn from(_: reqwest::Error) -> Self {
        MyError { name: "error" }
    }
}

impl From<serde_json::Error> for MyError {
    fn from(_: serde_json::Error) -> Self {
        MyError { name: "error" }
    }
}

impl From<actix_web::Error> for MyError {
    fn from(_: actix_web::Error) -> Self {
        MyError { name: "error" }
    }
}

impl error::ResponseError for MyError {}

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

async fn find_abi_method_by_txn_input(
    input: &String,
    methods: &Vec<AbiMethod>,
) -> Result<AbiMethod> {
    for method in methods {
        let hex = get_hex(&get_method_full_string(method)?)?;
        if input[2..10].eq(&hex[..8]) {
            return Ok(method.clone());
        }
    }
    Err(MyError {
        name: "method not found",
    })
}

fn get_hex(input: &String) -> Result<String> {
    let mut hasher = Keccak256::new();
    hasher.update(input.as_bytes());
    let res = hasher.finalize();
    Ok(format!("{:x}", res))
}

fn find_method_in_contract(contract: &String, method: &AbiMethod) -> Result<usize> {
    let start = contract.find(&format!("function {}", method.name));
    let start = match start {
        Some(v) => v,
        None => {
            return Err(MyError {
                name: "method not found",
            })
        }
    };
    let line_number = find_line_number_in_contract(contract, start);
    Ok(line_number)
}

fn find_line_number_in_contract(contract: &String, index: usize) -> usize {
    let mut n = 0;
    for (i, c) in contract.chars().enumerate() {
        if i >= index {
            break;
        }
        if c == '\n' {
            n += 1;
        }
    }
    n
}

async fn index(req: web::Json<Request>) -> Result<impl Responder> {
    let txn_input = get_txn_input(&req.txn).await?;
    let method = find_abi_method_by_txn_input(&txn_input, &req.abi).await?;
    let line_number = find_method_in_contract(&req.contract, &method)? as u32 + 1;
    let response = Response {
        method,
        line_number,
    };
    Ok(web::Json(response))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().route("/", web::post().to(index)))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
