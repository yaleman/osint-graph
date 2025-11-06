use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Redirect,
};
use serde::Deserialize;
use tower_sessions::Session;
use tracing::*;

use crate::{entity::user, SharedState};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};

// Query params for OAuth callback
#[derive(Debug, Deserialize)]
pub struct OAuthCallbackQuery {
    code: String,
    state: String,
}

// ========== Auth Handlers ==========

#[instrument(level = "info", skip_all)]
pub(crate) async fn auth_login(
    State(state): State<SharedState>,
) -> Result<Redirect, (StatusCode, String)> {
    let reader = state.read().await;
    let oauth_client = reader.oauth_client.as_ref().ok_or((
        StatusCode::SERVICE_UNAVAILABLE,
        "OAuth not configured".to_string(),
    ))?;

    let (auth_url, _state) = oauth_client.generate_auth_url().await.map_err(|e| {
        error!("Failed to generate auth URL: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to start login".to_string(),
        )
    })?;

    Ok(Redirect::to(&auth_url))
}

#[instrument(level = "info", skip_all)]
pub(crate) async fn auth_callback(
    State(state): State<SharedState>,
    Query(query): Query<OAuthCallbackQuery>,
    session: Session,
) -> Result<Redirect, (StatusCode, String)> {
    debug!(
        "Auth callback received - code: {}, state: {}",
        &query.code, &query.state
    );
    let reader = state.read().await;
    let oauth_client = reader.oauth_client.as_ref().ok_or((
        StatusCode::SERVICE_UNAVAILABLE,
        "OAuth not configured".to_string(),
    ))?;

    // Exchange code for tokens
    let (email, subject) = oauth_client
        .exchange_code(&query.code, &query.state)
        .await
        .map_err(|e| {
            error!(error=?e, "Failed to exchange OAuth2 code with IDP!");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Authentication failed".to_string(),
            )
        })?;

    debug!(
        "OAuth2 Code exchange successful - email: {}, subject: {}",
        &email, &subject
    );

    // Get or create user in database

    let user = match user::Entity::find()
        .filter(user::Column::Subject.eq(subject.clone()))
        .one(&reader.conn)
        .await
        .map_err(|e| {
            error!("Failed to query user: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error".to_string(),
            )
        })? {
        Some(u) => u,
        None => {
            let new_user = user::ActiveModel {
                subject: Set(subject.clone()),
                email: Set(email.clone()),
                ..Default::default()
            };
            new_user.insert(&reader.conn).await.map_err(|e| {
                error!("Failed to create user: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Check the logs!".to_string(),
                )
            })?
        }
    };

    trace!("trying to create store user sesssion");
    // Store user subject in session
    session
        .insert("user_subject", user.subject.clone())
        .await
        .map_err(|e| {
            error!("Failed to store session: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to save session".to_string(),
            )
        })?;

    // Save the session to ensure it's persisted
    session.save().await.map_err(|e| {
        error!("Failed to save session: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to persist session".to_string(),
        )
    })?;

    info!("Successfully authenticated user: {}", user.subject);
    trace!("successfully stored user session, redirecting");
    Ok(Redirect::to("/"))
}

#[instrument(level = "info", skip_all)]
pub(crate) async fn auth_logout(session: Session) -> Result<Redirect, (StatusCode, String)> {
    session
        .remove::<String>("user_subject")
        .await
        .map_err(|e| {
            error!("Failed to clear session: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to logout".to_string(),
            )
        })?;

    Ok(Redirect::to("/"))
}
