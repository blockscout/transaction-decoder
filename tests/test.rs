use actix_web::{test, web, App};
use ethabi::Contract;
use std::{fs, str::FromStr};
use transaction_decoder::{index, Bytes, Request};

fn read_test_case(num: u32, txn: &str, network: String) -> (Request, String) {
    let abi = fs::read_to_string(format!("tests/test_cases/contract_{}/abi.json", num)).unwrap();
    let abi: Contract = serde_json::from_str(&abi).unwrap();
    let txn = Bytes::from_str(txn).expect("Invalid transaction hash");
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

async fn start_test(data: &Request) -> actix_web::dev::ServiceResponse {
    let app = test::init_service(App::new().route("/", web::post().to(index))).await;
    let req = test::TestRequest::post()
        .uri("/")
        .set_json(&data)
        .to_request();
    test::call_service(&app, req).await
}

#[actix_web::test]
async fn simple_empty_request_test() {
    let app = test::init_service(App::new().route("/", web::post().to(index))).await;
    let req = test::TestRequest::post().uri("/").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_client_error());
}

#[actix_web::test]
async fn wrong_transaction_test() {
    let (data, _) = read_test_case(
        1,
        "0x8e9624a11380ca4eeed2d16c2d4bd63d595b344988bc5864a1210ad95f4da0f0",
        "eth/mainnet".to_string(),
    );

    let resp = start_test(&data).await;

    assert!(resp.status().is_client_error());
}

#[actix_web::test]
async fn test_1() {
    let (data, ans) = read_test_case(
        1,
        "0x7b7e9c40f73ec6aa0b14ef61b485d7d41a9b2e70befed0b03face3bf3412c57e",
        "eth/mainnet".to_string(),
    );

    let resp = start_test(&data).await;

    assert!(resp.status().is_success());

    let body = test::read_body(resp).await;

    assert_eq!(body, ans);
}

#[actix_web::test]
async fn test_2() {
    let (data, ans) = read_test_case(
        2,
        "0x7c58d31f4a66afbd36e63c84795b1a5ce584d7f3c76710d16e0bb96319f95368",
        "eth/mainnet".to_string(),
    );

    let resp = start_test(&data).await;

    assert!(resp.status().is_success());

    let body = test::read_body(resp).await;

    assert_eq!(body, ans);
}

#[actix_web::test]
async fn test_3() {
    let (data, ans) = read_test_case(
        3,
        "0x35ba6e645cf20e91ac96e7ffc882df16b63a7454ce879d9146924284dc32c847",
        "eth/mainnet".to_string(),
    );

    let resp = start_test(&data).await;

    assert!(resp.status().is_success());

    let body = test::read_body(resp).await;

    assert_eq!(body, ans);
}

#[actix_web::test]
async fn non_existent_network_test() {
    let (data, _) = read_test_case(
        2,
        "0x7c58d31f4a66afbd36e63c84795b1a5ce584d7f3c76710d16e0bb96319f95368",
        "etcssss/mainnet".to_string(),
    );

    let resp = start_test(&data).await;

    assert!(resp.status().is_client_error());
}

#[actix_web::test]
async fn wrong_network_test() {
    let (data, _) = read_test_case(
        2,
        "0x7c58d31f4a66afbd36e63c84795b1a5ce584d7f3c76710d16e0bb96319f95368",
        "etc/mainnet".to_string(),
    );

    let resp = start_test(&data).await;

    assert!(resp.status().is_client_error());
}
