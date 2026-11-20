mod test_case;

use actix_web::{test, web, App};
use transaction_decoder::decode;

use test_case::{read_test_case, start_test};

#[actix_web::test]
async fn simple_empty_request_test() {
    let app = test::init_service(App::new().route("/", web::post().to(decode))).await;
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
async fn simple_transfer_test() {
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
async fn random_contract_test() {
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
async fn fallback_test() {
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

#[actix_web::test]
async fn function_overload_test() {
    let (data, ans) = read_test_case(
        4,
        "0xca5d8ff91cba269df4b02852004ba2d2e4b2039342c484b4a78d87300a5c242b",
        "poa/sokol".to_string(),
    );

    let resp = start_test(&data).await;

    assert!(resp.status().is_success());

    let body = test::read_body(resp).await;

    assert_eq!(body, ans);
}

#[actix_web::test]
async fn bytes_bool_array_uint_test() {
    let (data, ans) = read_test_case(
        5,
        "0xb52b6348d27ca1a535a284bab143d8c233b7ff6033469e0b180cc0bdd2f7ccf7",
        "poa/sokol".to_string(),
    );

    let resp = start_test(&data).await;

    assert!(resp.status().is_success());

    let body = test::read_body(resp).await;

    assert_eq!(body, ans);
}

#[actix_web::test]
async fn struct_decode_test() {
    let (data, ans) = read_test_case(
        6,
        "0xa9237344b1f40d33a37eaeb1076ca781b359c2810a30fb663fea9951a7c3243d",
        "poa/sokol".to_string(),
    );

    let resp = start_test(&data).await;

    assert!(resp.status().is_success());

    let body = test::read_body(resp).await;

    assert_eq!(std::str::from_utf8(&body).unwrap().replace(" ", ""), ans);
}
