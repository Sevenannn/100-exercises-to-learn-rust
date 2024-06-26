use crate::data::*;
use crate::store::*;
use axum::{
    body::Body,
    debug_handler,
    extract::Json,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Extension, Router,
};
use std::sync::{Arc, RwLock};

// Create a ticket
#[debug_handler]
async fn create_ticket(
    ticket_store: Extension<SharedTicketStore>,
    Json(ticket_draft): Json<TicketDraft>,
) -> impl IntoResponse {
    let ticket_id = ticket_store.write().unwrap().add_ticket(ticket_draft);
    Response::builder()
        .status(StatusCode::CREATED)
        .body(Body::from(serde_json::to_string(&ticket_id).unwrap()))
        .unwrap()
}

// Retreive ticket details
#[debug_handler]
async fn retrieve_ticket(
    ticket_store: Extension<SharedTicketStore>,
    Json(id): Json<TicketId>,
) -> impl IntoResponse {
    let ticket = ticket_store
        .read()
        .unwrap()
        .get(id)
        .unwrap()
        .read()
        .unwrap()
        .clone();
    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(serde_json::to_string(&ticket).unwrap()))
        .unwrap()
}

// Update Ticket
async fn update_ticket(
    ticket_store: Extension<SharedTicketStore>,
    Json(ticket_patch): Json<TicketPatch>,
) -> impl IntoResponse {
    if let Some(ticket) = ticket_store.write().unwrap().get(ticket_patch.id) {
        let mut ticket = ticket.write().unwrap();
        if let Some(title) = ticket_patch.title {
            ticket.title = title;
        }
        if let Some(description) = ticket_patch.description {
            ticket.description = description;
        }
        if let Some(status) = ticket_patch.status {
            ticket.status = status;
        }
        return Response::builder()
            .status(StatusCode::OK)
            .body(Body::from(format!("Ticket {} updated", ticket_patch.id)))
            .unwrap();
    }
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::from(format!("Ticket {} not found", ticket_patch.id)))
        .unwrap()
}

// Router
pub fn app() -> Router {
    let ticket_store: Arc<RwLock<TicketStore>> = Arc::new(RwLock::new(TicketStore::new()));

    Router::new()
        .route("/", get(|| async { "Hello, Rust!" }))
        .route("/create-ticket", post(create_ticket))
        .route("/get-ticket", get(retrieve_ticket))
        .route("/update-ticket", post(update_ticket))
        .layer(Extension(ticket_store))
}
