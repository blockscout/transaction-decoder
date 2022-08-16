use actix_web::{http::StatusCode, web, Responder};
use anyhow::anyhow;
use ethabi::{ethereum_types::H256, Contract, Event, Function, Log, RawLog};
use std::collections::HashMap;

use crate::DisplayBytes;

use crate::{
    AbiResponse, Error, EventRequest, EventResponse, Request, Response, ResponseMethod,
    Transaction, TxLog,
};

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
            .map_err(|err| anyhow!(err))?,
    };
    Ok(web::Json(response))
}

async fn get_tx_logs(txn_hash: &DisplayBytes, network: &String) -> Result<Vec<TxLog>> {
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
    match res.result {
        Some(result) => Ok(result.logs),
        None => Err(Error::GetTxInfo("Jops".to_string())),
    }
}

async fn get_contract_abi(address: &DisplayBytes) -> Result<Contract> {
    let res = reqwest::get(format!(
        "https://blockscout.com/eth/mainnet/api?module=contract&action=getabi&address={}",
        address
    ))
    .await
    .map_err(|err| anyhow!(err))?;

    if res.status() == StatusCode::NOT_FOUND {
        return Err(Error::GetTxInfo("Wrong network".to_string()));
    }

    let res: AbiResponse = res.json().await.map_err(|err| anyhow!(err))?;

    if res.status == "0" {
        return Err(Error::GetTxInfo(res.message));
    }

    let c: Contract = serde_json::from_str(
        &res.result
            .ok_or_else(|| Error::GetTxInfo("Abi parsing failed".to_string()))?,
    )
    .map_err(|err| anyhow!(err))?;

    Ok(c)
}

async fn get_log_event(log: &TxLog, abi_map: &mut HashMap<Vec<u8>, Contract>) -> Result<Event> {
    abi_map
        .entry(log.address.to_vec())
        .or_insert(get_contract_abi(&log.address).await?);

    let abi = abi_map
        .get(&log.address.to_vec())
        .ok_or_else(|| Error::GetTxInfo("Failed to get abi".to_string()))?;

    for event in abi.events() {
        if log.topics[0]
            .clone()
            .ok_or_else(|| Error::GetTxInfo("Failed to decode anonymous event".to_string()))?
            .0
            == event.signature().as_bytes()
        {
            return Ok(event.clone());
        }
    }

    Err(Error::GetTxInfo("Failed to find event".to_string()))
}

async fn procces_events(logs: &Vec<TxLog>) -> Result<Vec<(Event, Log, String)>> {
    let mut abi_map: HashMap<Vec<u8>, Contract> = HashMap::new();

    let mut proccesed_events: Vec<(Event, Log, String)> = Vec::new();

    for log in logs {
        let event = match get_log_event(log, &mut abi_map).await {
            Ok(v) => v,
            Err(_) => continue,
        };
        let parced_log = event
            .parse_log(RawLog {
                data: log.data.to_vec(),
                topics: log
                    .topics
                    .iter()
                    .filter(|x| **x != None)
                    .map(|x| H256::from_slice(&x.clone().unwrap().to_vec()))
                    .collect(),
            })
            .map_err(|err| anyhow!(err))?;

        proccesed_events.push((event, parced_log, log.index.to_string()));
    }

    Ok(proccesed_events)
}

pub async fn decode_events(req: web::Json<EventRequest>) -> Result<impl Responder> {
    let logs = get_tx_logs(&req.tx_hash, &req.network).await?;
    let proccesed_events = procces_events(&logs).await?;
    let response = EventResponse::new(proccesed_events);
    Ok(web::Json(response))
}
