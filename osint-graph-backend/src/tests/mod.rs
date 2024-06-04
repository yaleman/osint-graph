use std::sync::Arc;

use crate::{build_app, AppState};
use axum_test::*;
use osint_graph_shared::node::Node;
use osint_graph_shared::project::Project;
use tokio::sync::RwLock;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use uuid::Uuid;

async fn setup_test_server() -> TestServer {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from("debug")
                .unwrap_or_else(|_| "osint_graph_backend=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let shared_state = Arc::new(RwLock::new(AppState::test().await));
    let app = build_app(&shared_state);

    let config = TestServerConfig::builder()
        // Preserve cookies across requests
        // for the session cookie to work.
        .save_cookies()
        .expect_success_by_default()
        .mock_transport()
        .build();

    TestServer::new_with_config(app, config).unwrap()
}

#[tokio::test]
async fn test_failing_setup_server() {
    // I sure hope this path isn't writeable!
    crate::storage::test_db(Some(format!(
        "/asdfasdf{}/asd{}fsadfdf",
        Uuid::new_v4(),
        Uuid::new_v4()
    )))
    .await
    .expect_err("Should fail to open DB");
}

#[tokio::test]
async fn test_api_project_node_save_load() {
    let server = setup_test_server().await;

    let id = Uuid::new_v4();
    let project_id = Uuid::new_v4();

    let project = Project {
        id: project_id,
        name: "foobar".to_string(),
        user: Uuid::new_v4(),
        creationdate: chrono::Utc::now(),
        last_updated: None,
        nodes: Default::default(),
    };

    // create the project
    let res = server.post("/api/v1/project").json(&project).await;
    res.assert_status_ok();

    let res = server
        .get(&format!("/api/v1/project/{}", project_id.to_string()))
        .await;
    res.assert_status_ok();

    let res = server
        .post("/api/v1/node")
        .json(&Node {
            project_id,
            id,
            value: "foo".to_string(),
            updated: chrono::Utc::now(),
            ..Default::default()
        })
        .await;
    assert_eq!(res.status_code(), 200);

    let res = server
        .get(&format!("/api/v1/node/{}", id.to_string()))
        .expect_success()
        .await;
    assert_eq!(res.status_code(), 200);

    let res = server
        .get(&format!("/api/v1/node/{}", Uuid::new_v4().to_string()))
        .expect_failure()
        .await;
    assert_eq!(res.status_code(), 404);

    // looking for something that shouldn't exist
    let res = server
        .get(&format!("/api/v1/project/{}", id.to_string()))
        .expect_failure()
        .await;
    assert_eq!(res.status_code(), 404);

    // looking for something that shouldn't exist
    let res = server.get("/api/v1/projects").expect_success().await;
    assert_eq!(res.status_code(), 200);
    assert!(!res.json::<Vec<Project>>().is_empty());

    // looking for something that shouldn't exist
    let res = server
        .get(&format!("/api/v1/project/{}", id.to_string()))
        .expect_failure()
        .await;
    assert_eq!(res.status_code(), 404);
}
