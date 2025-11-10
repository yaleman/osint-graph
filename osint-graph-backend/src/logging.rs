//! Logging things
//!

use std::time::Duration;

use axum::{http::header::CONTENT_LENGTH, response::Response};
use tower_http::{
    classify::{ServerErrorsAsFailures, SharedClassifier},
    trace::{OnRequest, OnResponse, TraceLayer},
};
use tracing::{trace, Span};

#[derive(Copy, Clone)]
pub(crate) struct OsintSpanner {}

impl<B> tower_http::trace::MakeSpan<B> for OsintSpanner {
    fn make_span(&mut self, request: &axum::http::Request<B>) -> Span {
        let method = request.method().to_string();
        let uri = request.uri().to_string();
        tracing::info_span!(
            "request",
            method = %method,
            uri = %uri,
            status = tracing::field::Empty,
            latency_ms = tracing::field::Empty,
            bytes = tracing::field::Empty
        )
    }
}

impl<B> OnRequest<B> for OsintSpanner {
    fn on_request(&mut self, _request: &axum::http::Request<B>, _span: &Span) {
        trace!("request received");
    }
}

impl<B> OnResponse<B> for OsintSpanner {
    fn on_response(self, response: &Response<B>, latency: Duration, span: &Span) {
        span.record("status", response.status().as_u16());
        span.record("latency_ms", latency.as_millis() as u64);
        if let Some(content_length) = response
            .headers()
            .get(CONTENT_LENGTH)
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok())
        {
            span.record("bytes", content_length);
        }
        tracing::event!(tracing::Level::INFO, "response sent");
    }
}

pub(crate) fn logging_layer(
) -> TraceLayer<SharedClassifier<ServerErrorsAsFailures>, OsintSpanner, OsintSpanner, OsintSpanner>
{
    TraceLayer::new_for_http()
        .on_request(OsintSpanner {})
        .make_span_with(OsintSpanner {})
        .on_response(OsintSpanner {})
}
