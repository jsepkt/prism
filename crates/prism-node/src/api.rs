use crate::service::NodeService;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, post},
    Json, Router,
};
use prism_core::Transaction;
use prism_crypto::PublicKey;
use serde::{Deserialize, Serialize};
use tower_http::cors::{Any, CorsLayer};

const DASHBOARD_HTML: &str = include_str!("../../../apps/dashboard/index.html");
const MOBILE_HTML: &str = include_str!("../../../apps/mobile/index.html");

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    block_height: u64,
    latest_block_hash: String,
    validator: String,
    pending_mempool_txs: usize,
}

#[derive(Deserialize)]
struct FaucetRequest {
    pubkey: String,
    amount: u64,
}

pub fn create_router(service: NodeService) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/", get(dashboard_handler))
        .route("/mobile", get(mobile_handler))
        .route("/mobile/", get(mobile_handler))
        .route("/health", get(health_handler))
        .route("/api/v1/state", get(state_handler))
        .route("/api/v1/blocks/latest", get(latest_block_handler))
        .route("/api/v1/accounts/:pubkey", get(get_account_handler))
        .route("/api/v1/bounties", get(get_bounties_handler))
        .route("/api/v1/schemas", get(get_schemas_handler))
        .route("/api/v1/transactions", post(submit_transaction_handler))
        .route("/api/v1/dev/faucet", post(faucet_handler))
        .layer(cors)
        .with_state(service)
}

async fn dashboard_handler() -> impl IntoResponse {
    Html(DASHBOARD_HTML)
}

async fn mobile_handler() -> impl IntoResponse {
    Html(MOBILE_HTML)
}

async fn health_handler(State(svc): State<NodeService>) -> impl IntoResponse {
    let state = svc.state.read().await;
    let mempool = svc.mempool.read().await;
    let res = HealthResponse {
        status: "healthy".to_string(),
        block_height: state.block_height,
        latest_block_hash: state.latest_block_hash.to_hex(),
        validator: svc.validator_keypair.public_key().to_hex(),
        pending_mempool_txs: mempool.size(),
    };
    Json(res)
}

async fn state_handler(State(svc): State<NodeService>) -> impl IntoResponse {
    let state = svc.state.read().await;
    Json(serde_json::json!({
        "block_height": state.block_height,
        "latest_block_hash": state.latest_block_hash.to_hex(),
        "total_accounts": state.accounts.len(),
        "total_bounties": state.bounties.len(),
        "total_schemas": state.schemas.len(),
    }))
}

async fn latest_block_handler(State(svc): State<NodeService>) -> impl IntoResponse {
    let blocks = svc.blocks.read().await;
    if let Some(latest) = blocks.last() {
        (StatusCode::OK, Json(serde_json::to_value(latest).unwrap())).into_response()
    } else {
        (StatusCode::OK, Json(serde_json::json!({ "message": "Genesis only" }))).into_response()
    }
}

async fn get_account_handler(
    State(svc): State<NodeService>,
    Path(pubkey_hex): Path<String>,
) -> impl IntoResponse {
    let pubkey = match PublicKey::from_hex(&pubkey_hex) {
        Ok(pk) => pk,
        Err(_) => return (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": "Invalid pubkey hex" }))).into_response(),
    };
    let state = svc.state.read().await;
    let account = state.get_account(&pubkey);
    Json(serde_json::to_value(account).unwrap()).into_response()
}

async fn get_bounties_handler(State(svc): State<NodeService>) -> impl IntoResponse {
    let state = svc.state.read().await;
    let bounties: Vec<_> = state.bounties.values().cloned().collect();
    Json(serde_json::to_value(bounties).unwrap())
}

async fn get_schemas_handler(State(svc): State<NodeService>) -> impl IntoResponse {
    let state = svc.state.read().await;
    let schemas: Vec<_> = state.schemas.values().cloned().collect();
    Json(serde_json::to_value(schemas).unwrap())
}

async fn submit_transaction_handler(
    State(svc): State<NodeService>,
    Json(tx): Json<Transaction>,
) -> impl IntoResponse {
    match svc.submit_transaction(tx).await {
        Ok(hash) => (StatusCode::ACCEPTED, Json(serde_json::json!({ "tx_hash": hash.to_hex() }))),
        Err(err) => (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": err }))),
    }
}

async fn faucet_handler(
    State(svc): State<NodeService>,
    Json(req): Json<FaucetRequest>,
) -> impl IntoResponse {
    let pubkey = match PublicKey::from_hex(&req.pubkey) {
        Ok(pk) => pk,
        Err(_) => return (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": "Invalid pubkey" }))),
    };
    let mut state = svc.state.write().await;
    let acc = state.get_account_mut(&pubkey);
    acc.balance += req.amount;
    (StatusCode::OK, Json(serde_json::json!({
        "pubkey": req.pubkey,
        "new_balance": acc.balance,
    })))
}
