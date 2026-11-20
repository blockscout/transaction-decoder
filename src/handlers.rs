use actix_web::{http::StatusCode, web, Responder};
use anyhow::anyhow;
use ethabi::{Contract, Function};

use crate::DisplayBytes;

use crate::{Error, Request, Response, ResponseMethod, Transaction};

type Result<T> = std::result::Result<T, Error>;

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
) -> Result<Option<Function>> {
    if input.len() < 4 {
        if abi.fallback {
            return Ok(None);
        }
        return Err(Error::NotFound);
    }
    for function in &abi.functions {
        for f in function.1 {
            let hex = f.short_signature();
            if &input[0..4] == hex.as_slice() {
                return Ok(Some(f.clone()));
            }
        }
    }

    if abi.fallback {
        return Ok(None);
    }

    Err(Error::NotFound)
}

pub async fn decode(req: web::Json<Request>) -> Result<impl Responder> {
    let txn_input = get_txn_input(&req.tx_hash, &req.network).await?;
    let method = find_abi_method_by_txn_input(&txn_input.0, &req.abi).await?;
    let response = Response {
        method: method
            .map(|x| ResponseMethod::new(x, &txn_input.0[4..]))
            .transpose()
            .map_err(anyhow::Error::msg)?,
    };
    Ok(web::Json(response))
}
