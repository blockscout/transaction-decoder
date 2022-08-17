use actix_web::{test, web, App};
use ethabi::Contract;
use std::{fs, str::FromStr};
use transaction_decoder::{decode, DisplayBytes, Request};

pub fn read_test_case(num: u32, txn: &str, network: String) -> (Request, String) {
    let abi = fs::read_to_string(format!("tests/test_cases/contract_{}/abi.json", num)).unwrap();
    let abi: Contract = serde_json::from_str(&abi).unwrap();
    let txn = DisplayBytes::from_str(txn).expect("Invalid transaction hash");
    let ans = fs::read_to_string(format!("tests/test_cases/contract_{}/ans.json", num)).unwrap();

    (
        Request {
            abi,
            tx_hash: txn,
            network,
        },
        ans.replace(" ", "").replace("\n", ""),
    )
}

pub async fn start_test(data: &Request) -> actix_web::dev::ServiceResponse {
    let app = test::init_service(App::new().route("/", web::post().to(decode))).await;
    let req = test::TestRequest::post()
        .uri("/")
        .set_json(&data)
        .to_request();
    test::call_service(&app, req).await
}
