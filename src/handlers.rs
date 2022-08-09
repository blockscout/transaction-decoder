use actix_web::{error, http::StatusCode, web, Responder};
use anyhow::anyhow;
use ethabi::{Contract, Function, Token};

use crate::DisplayBytes;
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

async fn get_txn_input(txn_hash: &DisplayBytes, network: &String) -> Result<DisplayBytes> {
    let res = reqwest::get(format!(
        "https://blockscout.com/{}/api?module=transaction&action=gettxinfo&txhash={}",
        network, txn_hash
    ))
    .await
    .map_err(|err| anyhow!(err))?;

    if res.status() == StatusCode::NOT_FOUND {
        return Err(Error::GetTxInfo("Wrong network".to_string()));
    }

    let res: Transaction = res.json().await.map_err(|err| anyhow!(err))?;

    if res.status != "1" {
        return Err(Error::GetTxInfo(res.message));
    }

    res.result
        .map(|result| result.input)
        .ok_or_else(|| Error::Other(anyhow!("Missing input block")))
}

async fn find_abi_method_by_txn_input(
    input: &bytes::Bytes,
    abi: &Contract,
) -> Result<(Option<Function>, Option<Vec<Token>>)> {
    if input.len() < 4 {
        if abi.fallback {
            return Ok((None, None));
        }
        return Err(Error::NotFound);
    }
    for function in &abi.functions {
        for f in function.1 {
            let hex = f.short_signature();
            if &input[0..4] == hex.as_slice() {
                let decoded_inputs = f.decode_input(&input[4..]).ok();
                return Ok((Some(f.clone()), decoded_inputs));
            }
        }
    }

    if abi.fallback {
        return Ok((None, None));
    }

    Err(Error::NotFound)
}

pub async fn decode(req: web::Json<Request>) -> Result<impl Responder> {
    let txn_input = get_txn_input(&req.tx_hash, &req.network).await?;
    let (method, decoded_inputs) = find_abi_method_by_txn_input(&txn_input.0, &req.abi).await?;
    let response = Response {
        method,
        decoded_inputs,
    };
    Ok(web::Json(response))
}
