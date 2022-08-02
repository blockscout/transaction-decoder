use actix_web::{error, http::StatusCode, web, Responder};
use anyhow::anyhow;
use ethabi::{Contract, Function};

use crate::bytes::Bytes;
use thiserror::Error;

use crate::{Request, Response, Transaction};

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("not found")]
    NotFound,

    #[error("Error while getting tx info: {0}")]
    GetTxInfo(String),

    #[error("{0}")]
    Other(#[from] anyhow::Error),
}

impl error::ResponseError for Error {
    fn status_code(&self) -> StatusCode {
        match *self {
            Error::NotFound => StatusCode::BAD_REQUEST,
            Error::GetTxInfo(_) => StatusCode::BAD_REQUEST,
            Error::Other(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

async fn get_txn_input(txn_hash: &Bytes, network: &String) -> Result<Bytes> {
    let res = reqwest::get(format!(
        "https://blockscout.com/{}/api?module=transaction&action=gettxinfo&txhash={}",
        network, txn_hash
    ))
    .await
    .map_err(|err| anyhow!(err))?;

    if res.status() == StatusCode::NOT_FOUND {
        return Err(Error::NotFound);
    }

    let res: Transaction = res.json().await.map_err(|err| anyhow!(err))?;

    if res.status != "1" {
        return Err(Error::GetTxInfo(res.message));
    }

    res.result
        .map(|result| result.input)
        .ok_or_else(|| Error::Other(anyhow!("Missing input block")))
}

async fn find_abi_method_by_txn_input(input: &Bytes, abi: &Contract) -> Result<Function> {
    if input.0.len() < 4 {
        return Err(Error::NotFound);
    }

    for function in &abi.functions {
        let hex = function.1[0].short_signature();
        if &input.0[0..4] == hex.as_slice() {
            return Ok(function.1[0].clone());
        }
    }

    Err(Error::NotFound)
}

pub async fn index(req: web::Json<Request>) -> Result<impl Responder> {
    let txn_input = get_txn_input(&req.tx_hash, &req.network).await?;
    let method = find_abi_method_by_txn_input(&txn_input, &req.abi).await?;
    let response = Response { method };

    Ok(web::Json(response))
}
