mod support;

use astrum_api::client::ClientAddress;
use axum::Router;
use axum::body::Body;
use axum::extract::{ConnectInfo, Extension};
use axum::http::{Request, StatusCode};
use axum::routing::get;
use http_body_util::BodyExt;
use serde_json::{Value, json};
use std::net::SocketAddr;
use support::Fixture;
use tower::ServiceExt;

async fn probe(Extension(ClientAddress(address)): Extension<ClientAddress>) -> axum::Json<Value> {
    axum::Json(json!({
        "extension": address.map(|address| address.to_string()),
        "accessor": astrum_api::auth::client_address(),
    }))
}

async fn seen(
    fixture: &Fixture,
    peer: Option<&str>,
    forwarded: &[&str],
    real_ip: Option<&str>,
) -> Value {
    let mut request = Request::builder().uri("/probe");
    for value in forwarded {
        request = request.header("x-forwarded-for", *value);
    }
    if let Some(address) = real_ip {
        request = request
            .header("x-real-ip", address)
            .header("forwarded", format!("for={address}"));
    }
    let mut request = request.body(Body::empty()).unwrap();
    if let Some(peer) = peer {
        let peer: std::net::IpAddr = peer.parse().unwrap();
        request
            .extensions_mut()
            .insert(ConnectInfo(SocketAddr::new(peer, 40000)));
    }
    let app = astrum_api::router_with(
        fixture.state.clone(),
        Router::new().route("/probe", get(probe)),
    );
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

#[tokio::test]
async fn the_outer_layer_derives_one_address_for_the_extension_and_the_accessor() {
    let fixture = Fixture::new().await;
    fixture.trust(&["10.0.0.0/8"]);

    let direct = seen(
        &fixture,
        Some("203.0.113.7"),
        &["198.51.100.1"],
        Some("198.51.100.2"),
    )
    .await;
    assert_eq!(
        direct,
        json!({"extension": "203.0.113.7", "accessor": "203.0.113.7"})
    );

    let proxied = seen(
        &fixture,
        Some("10.0.0.1"),
        &["198.51.100.1, 203.0.113.5", "10.2.2.2"],
        Some("198.51.100.2"),
    )
    .await;
    assert_eq!(
        proxied,
        json!({"extension": "203.0.113.5", "accessor": "203.0.113.5"})
    );

    let unknown = seen(&fixture, None, &["198.51.100.1"], None).await;
    assert_eq!(unknown, json!({"extension": null, "accessor": null}));
}
