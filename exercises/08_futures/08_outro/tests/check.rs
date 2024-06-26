use data::{Ticket, TicketDraft, TicketPatch};
use outro_08::*;
use store::TicketId;
use ticket_fields::test_helpers::*;

use axum::{
    body::Body,
    http::{self, Request, StatusCode},
};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use tokio::net::TcpListener;
use tower::{Service, ServiceExt};

/// Checklist

/// Test for spinning up the real server
#[tokio::test]
async fn the_real_deal() {
    let listener = TcpListener::bind("0.0.0.0:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, outro_08::handlers::app())
            .await
            .unwrap();
    });

    let client = hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new())
        .build_http();

    let response = client
        .request(
            Request::builder()
                .uri(format!("http://{addr}"))
                .header("Host", "localhost")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = response.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(&body[..], b"Hello, Rust!");
}

/// Test for sending request to server using multiple threads concurrently
#[tokio::test]
async fn multiple_request() {
    let mut app = outro_08::handlers::app().into_service();

    // Test create ticket
    let draft = TicketDraft {
        title: ticket_title(),
        description: ticket_description(),
    };

    let build_request = |draft: &TicketDraft| {
        Request::builder()
            .method(http::Method::POST)
            .uri("/create-ticket")
            .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
            .body(Body::from(serde_json::to_string(draft).unwrap()))
            .unwrap()
    };

    let request1 = build_request(&draft);
    let request2 = build_request(&draft);

    let first_handle = tokio::spawn(
        ServiceExt::<Request<Body>>::ready(&mut app)
            .await
            .unwrap()
            .call(request1),
    );
    let second_handle = tokio::spawn(
        ServiceExt::<Request<Body>>::ready(&mut app)
            .await
            .unwrap()
            .call(request2),
    );

    let (res1, res2) = tokio::join!(first_handle, second_handle);

    let response1 = res1.unwrap().unwrap();
    let response2 = res2.unwrap().unwrap();

    assert_eq!(response1.status(), StatusCode::CREATED);
    assert_eq!(response2.status(), StatusCode::CREATED);

    let body1 = response1.into_body().collect().await.unwrap().to_bytes();
    let body2 = response2.into_body().collect().await.unwrap().to_bytes();

    let id1: Value = serde_json::from_slice(&body1).unwrap();
    let id2: Value = serde_json::from_slice(&body2).unwrap();

    assert_eq!(id1, json!(TicketId(0)));
    assert_eq!(id2, json!(TicketId(1)));

    // Test get ticket
    let build_get_request = |ticket_id: TicketId| {
        Request::builder()
            .method(http::Method::GET)
            .uri("/get-ticket")
            .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
            .body(Body::from(serde_json::to_string(&ticket_id).unwrap()))
            .unwrap()
    };

    let request1 = build_get_request(TicketId(0));
    let request2 = build_get_request(TicketId(1));

    let first_handle = tokio::spawn(
        ServiceExt::<Request<Body>>::ready(&mut app)
            .await
            .unwrap()
            .call(request1),
    );
    let second_handle = tokio::spawn(
        ServiceExt::<Request<Body>>::ready(&mut app)
            .await
            .unwrap()
            .call(request2),
    );

    let (response1, response2) = tokio::join!(first_handle, second_handle);

    let response1 = response1.unwrap().unwrap();
    let response2 = response2.unwrap().unwrap();

    assert_eq!(response1.status(), StatusCode::OK);
    assert_eq!(response2.status(), StatusCode::OK);

    let body1 = response1.into_body().collect().await.unwrap().to_bytes();
    let body2 = response2.into_body().collect().await.unwrap().to_bytes();

    let ticket1: Value = serde_json::from_slice(&body1).unwrap();
    let ticket2: Value = serde_json::from_slice(&body2).unwrap();

    assert_eq!(
        ticket1,
        json!(Ticket {
            id: TicketId(0),
            title: ticket_title(),
            description: ticket_description(),
            status: data::Status::ToDo
        })
    );
    assert_eq!(
        ticket2,
        json!(Ticket {
            id: TicketId(1),
            title: ticket_title(),
            description: ticket_description(),
            status: data::Status::ToDo
        })
    );

    // Test update ticket
    let build_update_request = |ticket_patch: TicketPatch| {
        Request::builder()
            .method(http::Method::POST)
            .uri("/update-ticket")
            .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
            .body(Body::from(serde_json::to_string(&ticket_patch).unwrap()))
            .unwrap()
    };

    let patch1 = TicketPatch {
        id: TicketId(0),
        title: Some(ticket_title()),
        description: Some(ticket_description()),
        status: Some(data::Status::Done),
    };

    let patch2 = TicketPatch {
        id: TicketId(10),
        title: Some(ticket_title()),
        description: Some(ticket_description()),
        status: Some(data::Status::Done),
    };

    let request1 = build_update_request(patch1);
    let request2 = build_update_request(patch2);

    let first_handle = tokio::spawn(
        ServiceExt::<Request<Body>>::ready(&mut app)
            .await
            .unwrap()
            .call(request1),
    );
    let second_handle = tokio::spawn(
        ServiceExt::<Request<Body>>::ready(&mut app)
            .await
            .unwrap()
            .call(request2),
    );

    let (response1, response2) = tokio::join!(first_handle, second_handle);

    let response1 = response1.unwrap().unwrap();
    let response2 = response2.unwrap().unwrap();

    assert_eq!(response1.status(), StatusCode::OK);
    assert_eq!(response2.status(), StatusCode::NOT_FOUND);

    let body1 = response1.into_body().collect().await.unwrap().to_bytes();
    let body2 = response2.into_body().collect().await.unwrap().to_bytes();

    assert_eq!(&body1[..], b"Ticket 0 updated");
    assert_eq!(&body2[..], b"Ticket 10 not found");
}

// Reference for writing tests
// https://github.com/tokio-rs/axum/blob/main/examples/testing/src/main.rs
