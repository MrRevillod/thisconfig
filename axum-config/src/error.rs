use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};

use serde::Serialize;

#[derive(Serialize)]
pub struct ErrorResponse {
    code: u16,
    success: bool,
    message: String,
}

impl ErrorResponse {
    pub fn internal_server_error() -> Self {
        ErrorResponse {
            code: 500,
            success: false,
            message: "Internal Server Error".to_string(),
        }
    }
}

impl IntoResponse for ErrorResponse {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(self)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use crate::error::ErrorResponse;
    use axum::{http::StatusCode, response::IntoResponse};

    #[test]
    fn test_into_response_status() {
        let error = ErrorResponse::internal_server_error();
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
