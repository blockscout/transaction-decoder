use actix_web::{http::StatusCode, web, Responder};
use anyhow::anyhow;
use ethabi::{Contract, Event, Function, RawLog};
use std::collections::HashMap;

use crate::DisplayBytes;

use crate::{
    AbiResponse, DecodedEvent, Error, EventRequest, EventResponse, Request, Response,
    ResponseMethod, Transaction, TxLog,
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

    let res: Transaction = res.json().await.map_err(anyhow::Error::msg)?;

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

fn process_blockscout_response_status(status: StatusCode) -> Result<()> {
    if status == StatusCode::NOT_FOUND {
        return Err(Error::GetTxInfo("Wrong network".to_string()));
    }

    if status != StatusCode::OK {
        return Err(Error::Other(anyhow!(
            "call to blockscout failed with code {}",
            status
        )));
    }
    Ok(())
}

async fn get_tx_logs(txn_hash: &DisplayBytes, network: &String) -> Result<Vec<TxLog>> {
    let res = reqwest::get(format!(
        "https://blockscout.com/{}/api?module=transaction&action=gettxinfo&txhash={}",
        network, txn_hash
    ))
    .await
    .map_err(|err| anyhow!(err))?;

    process_blockscout_response_status(res.status())?;

    let res: Transaction = res.json().await.map_err(anyhow::Error::msg)?;

    if res.status != "1" {
        return Err(Error::GetTxInfo(res.message));
    }

    res.result
        .map(|result| result.logs)
        .ok_or_else(|| Error::Other(anyhow!("Missing input block")))
}

async fn get_contract_abi(network: &String, address: &DisplayBytes) -> Result<Contract> {
    let res = reqwest::get(format!(
        "https://blockscout.com/{}/api?module=contract&action=getabi&address={}",
        network, address
    ))
    .await
    .map_err(anyhow::Error::msg)?;

    process_blockscout_response_status(res.status())?;

    let res: AbiResponse = res.json().await.map_err(anyhow::Error::msg)?;

    if res.status == "0" {
        return Err(Error::GetTxInfo(res.message));
    }

    let c: Contract =
        serde_json::from_str(&res.result.ok_or_else(|| anyhow!("Abi parsing failed"))?)
            .map_err(anyhow::Error::msg)?;

    Ok(c)
}

async fn get_log_event(
    network: &String,
    log: &TxLog,
    abi_map: &mut HashMap<Vec<u8>, Contract>,
) -> Result<Event> {
    let abi = abi_map
        .entry(log.address.to_vec())
        .or_insert(get_contract_abi(network, &log.address).await?);

    let hash = log.topics[0]
        .as_ref()
        .ok_or_else(|| anyhow!("Failed to decode anonymous event"))?
        .0;

    abi.events()
        .into_iter()
        .find(|e| hash == e.signature().as_bytes())
        .cloned()
        .ok_or_else(|| Error::Other(anyhow!("Failed to find event")))
}

async fn process_events(network: &String, logs: &Vec<TxLog>) -> Result<Vec<Option<DecodedEvent>>> {
    let mut abi_map: HashMap<Vec<u8>, Contract> = HashMap::new();

    let mut processed_events: Vec<Option<DecodedEvent>> = Vec::new();

    for log in logs {
        let event = get_log_event(network, log, &mut abi_map).await.ok();

        let decoded_event = event.map(|e| -> Result<DecodedEvent> {
            let parsed_log = e
                .parse_log(RawLog {
                    data: log.data.to_vec(),
                    topics: log
                        .topics
                        .into_iter()
                        .filter(|x| *x != None)
                        .map(|x| x.unwrap())
                        .collect(),
                })
                .map_err(anyhow::Error::msg)?;
            Ok(DecodedEvent::new(e, parsed_log, log.index.as_str()))
        });

        processed_events.push(decoded_event.transpose()?);
    }

    Ok(processed_events)
}

pub async fn decode_events(req: web::Json<EventRequest>) -> Result<impl Responder> {
    let logs = get_tx_logs(&req.tx_hash, &req.network).await?;
    let proccesed_events = process_events(&req.network, &logs).await?;
    let response = EventResponse::new(proccesed_events);
    Ok(web::Json(response))
}
