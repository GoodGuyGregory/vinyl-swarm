pub async fn status_handler() -> impl IntoResponse {
    let message_status: &str = "vinyl swarm running: 👽 ";

    let json_response = serde_json::json!({
        "status": "ok",
        "message": message_status
    });

    Json(json_response)
}