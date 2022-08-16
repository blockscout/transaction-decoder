use actix_web::{test, web, App};
use std::{fs, str::FromStr};
use transaction_decoder::{decode_events, DisplayBytes, EventRequest};

#[actix_web::test]
async fn simple_events_decoder_test() {
    let app = test::init_service(App::new().route("/", web::post().to(decode_events))).await;
    let req = test::TestRequest::post().uri("/").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_client_error());
}

#[actix_web::test]
async fn usdt_transfer_test() {
    let app = test::init_service(App::new().route("/", web::post().to(decode_events))).await;
    let ans = fs::read_to_string("tests/events_cases/ans_1.json".to_string())
        .unwrap()
        .replace(" ", "")
        .replace("\n", "");
    let data = EventRequest {
        tx_hash: DisplayBytes::from_str(
            "0x3cdd9fb20ca11e7925798a4e555b63f0484c497283fe1bf89cf3d34433b31f91",
        )
        .expect("Invalid transaction"),

        network: "eth/mainnet".to_string(),
    };

    let req = test::TestRequest::post()
        .uri("/")
        .set_json(&data)
        .to_request();
    let resp = test::call_service(&app, req).await;

    let body = test::read_body(resp).await;

    assert_eq!(body, ans);
}
