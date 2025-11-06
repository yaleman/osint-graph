use std::sync::{Arc, Once};

use crate::entity::{node, project};
use crate::project::ProjectExport;
use crate::{build_app, AppState};
use axum_test::*;
use osint_graph_shared::node::NodeType;
use osint_graph_shared::StringVec;
use tokio::sync::RwLock;
use tracing::debug;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use uuid::Uuid;

static INIT: Once = Once::new();

async fn setup_test_server() -> TestServer {
    INIT.call_once(|| {
        tracing_subscriber::registry()
            .with(tracing_subscriber::EnvFilter::new(
                "osint_graph_backend=debug,tower_http=debug,debug",
            ))
            .with(tracing_subscriber::fmt::layer())
            .init();
    });
    let appstate = AppState::test().await;
    let dbpool: sqlx::Pool<sqlx::Sqlite> = appstate.conn.get_sqlite_connection_pool().clone();
    let shared_state = Arc::new(RwLock::new(appstate));
    let app = build_app(&shared_state, dbpool, false).await;

    let config = TestServerConfig {
        // Preserve cookies across requests
        // for the session cookie to work.
        save_cookies: true,

        expect_success_by_default: true,
        restrict_requests_with_http_schema: false,
        default_content_type: None,
        default_scheme: Some("http".into()),
        ..Default::default()
    };

    TestServer::new_with_config(app, config).unwrap()
}

#[tokio::test]
async fn test_failing_setup_server() {
    // I sure hope this path isn't writeable!
    crate::storage::start_db(Some(
        &format!("/asdfasdf{}/asd{}fsadfdf", Uuid::new_v4(), Uuid::new_v4()).into(),
    ))
    .await
    .expect_err("Should fail to open DB");
}

#[tokio::test]
async fn test_api_project_node_save_load() {
    let server = setup_test_server().await;

    let node_id = Uuid::new_v4();
    let project_id = Uuid::new_v4();

    let project = project::Model {
        id: project_id,
        name: "foobar".to_string(),
        user: Uuid::new_v4(),
        creationdate: chrono::Utc::now(),
        last_updated: None,
        description: None,
        tags: StringVec::default(),
    };

    // create the project
    let res = server.post("/api/v1/project").json(&project).await;
    res.assert_status_ok();

    let res = server.get(&format!("/api/v1/project/{}", project_id)).await;
    res.assert_status_ok();

    let res = server
        .post("/api/v1/node")
        .json(&node::Model {
            project_id,
            id: node_id,
            node_type: NodeType::Person,
            display: "Test Person".to_string(),
            value: "foo".to_string(),
            updated: chrono::Utc::now(),
            ..Default::default()
        })
        .await;
    assert_eq!(res.status_code(), 200);

    let res = server
        .get(&format!("/api/v1/node/{}", node_id))
        .expect_success()
        .await;
    assert_eq!(res.status_code(), 200);

    let res = server
        .get(&format!("/api/v1/node/{}", Uuid::new_v4()))
        .expect_failure()
        .await;
    assert_eq!(res.status_code(), 404);

    // looking for something that shouldn't exist
    let res = server
        .get(&format!("/api/v1/project/{}", node_id))
        .expect_failure()
        .await;
    assert_eq!(res.status_code(), 404);

    // looking for something that shouldn't exist
    let res = server.get("/api/v1/projects").expect_success().await;
    assert_eq!(res.status_code(), 200);
    assert!(!res.json::<Vec<project::Model>>().is_empty());

    // looking for something that shouldn't exist
    let res = server
        .get(&format!("/api/v1/project/{}", node_id))
        .expect_failure()
        .await;
    assert_eq!(res.status_code(), 404);
}

#[tokio::test]
async fn test_api_get_nodes_by_project() {
    let server = setup_test_server().await;

    let project_id = Uuid::new_v4();
    let node_id1 = Uuid::new_v4();
    let node_id2 = Uuid::new_v4();
    let other_project_id = Uuid::new_v4();
    let other_node_id = Uuid::new_v4();

    // Create first project
    let project1 = project::Model {
        id: project_id,
        name: "Test Project 1".to_string(),
        user: Uuid::new_v4(),
        creationdate: chrono::Utc::now(),
        last_updated: None,
        description: None,
        tags: StringVec::empty(),
    };

    // Create second project
    let project2 = project::Model {
        id: other_project_id,
        name: "Test Project 2".to_string(),
        user: Uuid::new_v4(),
        creationdate: chrono::Utc::now(),
        last_updated: None,
        description: None,
        tags: StringVec::empty(),
    };

    // Create both projects
    debug!("Creating project 1");
    server
        .post("/api/v1/project")
        .json(&project1)
        .await
        .assert_status_ok();
    debug!("Created project 1");

    server
        .post("/api/v1/project")
        .json(&project2)
        .await
        .assert_status_ok();
    debug!("Created project 2");

    // Test getting nodes from empty project
    let res = server
        .get(&format!("/api/v1/project/{}/nodes", project_id))
        .await;
    res.assert_status_ok();
    debug!("Fetched nodes for project 1");
    let nodes: Vec<node::Model> = res.json();
    assert!(nodes.is_empty());

    // Create nodes for first project
    let node1 = node::Model {
        project_id,
        id: node_id1,
        node_type: NodeType::Person,
        display: "John Doe".to_string(),
        value: "john@example.com".to_string(),
        updated: chrono::Utc::now(),
        notes: Some("First person".to_string()),
        pos_x: Some(100),
        pos_y: Some(200),
    };

    let node2 = node::Model {
        project_id,
        id: node_id2,
        node_type: NodeType::Domain,
        display: "example.com".to_string(),
        value: "example.com".to_string(),
        updated: chrono::Utc::now(),
        notes: Some("Domain node".to_string()),
        pos_x: Some(300),
        pos_y: Some(400),
    };

    // Create node for second project
    let other_node = node::Model {
        project_id: other_project_id,
        id: other_node_id,
        node_type: NodeType::Ip,
        display: "192.168.1.1".to_string(),
        value: "192.168.1.1".to_string(),
        updated: chrono::Utc::now(),
        notes: None,
        pos_x: Some(500),
        pos_y: Some(600),
    };

    // Add all nodes
    server
        .post("/api/v1/node")
        .json(&node1)
        .await
        .assert_status_ok();
    server
        .post("/api/v1/node")
        .json(&node2)
        .await
        .assert_status_ok();
    server
        .post("/api/v1/node")
        .json(&other_node)
        .await
        .assert_status_ok();

    // Test getting nodes for first project
    let res = server
        .get(&format!("/api/v1/project/{}/nodes", project_id))
        .await;
    res.assert_status_ok();
    let nodes: Vec<node::Model> = res.json();
    assert_eq!(nodes.len(), 2);

    // Verify we got the right nodes
    let node_ids: Vec<Uuid> = nodes.iter().map(|n| n.id).collect();
    assert!(node_ids.contains(&node_id1));
    assert!(node_ids.contains(&node_id2));
    assert!(!node_ids.contains(&other_node_id));

    // Test getting nodes for second project
    let res = server
        .get(&format!("/api/v1/project/{}/nodes", other_project_id))
        .await;
    res.assert_status_ok();
    let nodes: Vec<node::Model> = res.json();
    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0].id, other_node_id);

    // Test getting nodes for non-existent project
    let res = server
        .get(&format!("/api/v1/project/{}/nodes", Uuid::new_v4()))
        .await;
    res.assert_status_ok();
    let nodes: Vec<node::Model> = res.json();
    assert!(nodes.is_empty());
}

#[tokio::test]
async fn test_api_projects_crud() {
    let server = setup_test_server().await;

    // Test getting all projects (should include default project)
    let res = server.get("/api/v1/projects").await;
    res.assert_status_ok();
    let initial_projects: Vec<project::Model> = res.json();
    let initial_count = initial_projects.len();

    // Create a new project
    let project_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let project = project::Model {
        id: project_id,
        name: "CRUD Test Project".to_string(),
        user: user_id,
        creationdate: chrono::Utc::now(),
        last_updated: None,
        description: None,
        tags: StringVec::default(),
    };

    // Test project creation
    let res = server.post("/api/v1/project").json(&project).await;
    res.assert_status_ok();

    // Test getting all projects (should have one more)
    let res = server.get("/api/v1/projects").await;
    res.assert_status_ok();
    let projects: Vec<project::Model> = res.json();
    assert_eq!(projects.len(), initial_count + 1);

    // Test getting specific project
    let res = server.get(&format!("/api/v1/project/{}", project_id)).await;
    res.assert_status_ok();
    let retrieved_project: project::Model = res.json();
    assert_eq!(retrieved_project.id, project_id);
    assert_eq!(retrieved_project.name, "CRUD Test Project");
    assert_eq!(retrieved_project.user, user_id);

    // Test getting non-existent project
    let res = server
        .get(&format!("/api/v1/project/{}", Uuid::new_v4()))
        .expect_failure()
        .await;
    assert_eq!(res.status_code(), 404);

    let res = server
        .get(&format!("/api/v1/project/{}/export", retrieved_project.id))
        .expect_success()
        .await;

    let exported: ProjectExport = res.json();
    assert_eq!(exported.project.id, retrieved_project.id);
}

#[tokio::test]
async fn test_api_nodes_crud() {
    let server = setup_test_server().await;

    // Create a project first
    let project_id = Uuid::new_v4();
    let project = project::Model {
        id: project_id,
        name: "Node CRUD Test".to_string(),
        user: Uuid::new_v4(),
        creationdate: chrono::Utc::now(),
        last_updated: None,
        description: None,
        tags: StringVec::default(),
    };
    server
        .post("/api/v1/project")
        .json(&project)
        .await
        .assert_status_ok();

    // Test node creation
    let node_id = Uuid::new_v4();
    let node = node::Model {
        project_id,
        id: node_id,
        node_type: NodeType::Email,
        display: "test@example.com".to_string(),
        value: "test@example.com".to_string(),
        updated: chrono::Utc::now(),
        notes: Some("Test email node".to_string()),
        pos_x: Some(150),
        pos_y: Some(250),
    };

    let res = server.post("/api/v1/node").json(&node).await;
    res.assert_status_ok();

    // Test getting specific node
    let res = server.get(&format!("/api/v1/node/{}", node_id)).await;
    res.assert_status_ok();
    let retrieved_node: node::Model = res.json();
    assert_eq!(retrieved_node.id, node_id);
    assert_eq!(retrieved_node.project_id, project_id);
    assert_eq!(retrieved_node.node_type, NodeType::Email);
    assert_eq!(retrieved_node.display, "test@example.com");
    assert_eq!(retrieved_node.value, "test@example.com");
    assert_eq!(retrieved_node.notes, Some("Test email node".to_string()));
    assert_eq!(retrieved_node.pos_x, Some(150));
    assert_eq!(retrieved_node.pos_y, Some(250));

    // Test updating node (using same endpoint)
    let updated_node = node::Model {
        project_id,
        id: node_id,
        node_type: NodeType::Email,
        display: "updated@example.com".to_string(),
        value: "updated@example.com".to_string(),
        updated: chrono::Utc::now(),
        notes: Some("Updated test email node".to_string()),
        pos_x: Some(300),
        pos_y: Some(400),
    };

    let res = server
        .put(&format!("/api/v1/node/{}", node_id))
        .json(&updated_node)
        .await;
    res.assert_status_ok();

    // Verify the update
    let res = server.get(&format!("/api/v1/node/{}", node_id)).await;
    res.assert_status_ok();
    let retrieved_node: node::Model = res.json();
    assert_eq!(retrieved_node.display, "updated@example.com");
    assert_eq!(retrieved_node.value, "updated@example.com");
    assert_eq!(
        retrieved_node.notes,
        Some("Updated test email node".to_string())
    );
    assert_eq!(retrieved_node.pos_x, Some(300));
    assert_eq!(retrieved_node.pos_y, Some(400));

    // Test getting non-existent node
    let res = server
        .get(&format!("/api/v1/node/{}", Uuid::new_v4()))
        .expect_failure()
        .await;
    assert_eq!(res.status_code(), 404);
}

#[tokio::test]
async fn test_api_node_foreign_key_constraint() {
    let server = setup_test_server().await;

    // Try to create a node with non-existent project_id
    let non_existent_project_id = Uuid::new_v4();
    let node_id = Uuid::new_v4();
    let node = node::Model {
        project_id: non_existent_project_id,
        id: node_id,
        node_type: NodeType::Person,
        display: "Test Person".to_string(),
        value: "test".to_string(),
        updated: chrono::Utc::now(),
        notes: None,
        pos_x: None,
        pos_y: None,
    };

    // This should fail due to project validation (project doesn't exist)
    let res = server
        .post("/api/v1/node")
        .json(&node)
        .expect_failure()
        .await;
    assert_eq!(res.status_code(), 404); // Project not found
}

#[tokio::test]
async fn test_api_update_project() {
    let server = setup_test_server().await;

    // Create a project first
    let project_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let project = project::Model {
        id: project_id,
        name: "Original Name".to_string(),
        user: user_id,
        creationdate: chrono::Utc::now(),
        last_updated: None,

        description: None,
        tags: StringVec::default(),
    };

    server
        .post("/api/v1/project")
        .json(&project)
        .await
        .assert_status_ok();

    // Update the project with new data
    let updated_project = project::Model {
        id: project_id,
        name: "Updated Name".to_string(),
        user: user_id,
        creationdate: chrono::Utc::now(),
        last_updated: None,
        description: Some("A test description".to_string()),
        tags: StringVec(vec!["tag1".to_string(), "tag2".to_string()]),
    };

    let res = server
        .put(&format!("/api/v1/project/{}", project_id))
        .json(&updated_project)
        .await;
    res.assert_status_ok();

    // Verify the update
    let res = server.get(&format!("/api/v1/project/{}", project_id)).await;
    res.assert_status_ok();
    let retrieved_project: project::Model = res.json();
    assert_eq!(retrieved_project.id, project_id);
    assert_eq!(retrieved_project.name, "Updated Name");
    assert_eq!(
        retrieved_project.description,
        Some("A test description".to_string())
    );
    assert_eq!(
        retrieved_project.tags.0,
        vec!["tag1".to_string(), "tag2".to_string()]
    );
    assert!(retrieved_project.last_updated.is_some());

    // Test updating non-existent project
    let should_not_update_this = Uuid::new_v4();
    debug!(
        "Trying to update non-existent project {}",
        should_not_update_this
    );
    let res = server
        .put(&format!("/api/v1/project/{}", should_not_update_this))
        .json(&updated_project)
        .expect_failure()
        .await;
    assert_eq!(res.status_code(), 404);
}

#[tokio::test]
async fn test_api_delete_project() {
    let server = setup_test_server().await;

    // Create a project
    let project_id = Uuid::new_v4();
    let project = project::Model {
        id: project_id,
        name: "Project to Delete".to_string(),
        user: Uuid::new_v4(),
        creationdate: chrono::Utc::now(),
        last_updated: None,
        description: Some("Will be deleted".to_string()),
        tags: StringVec(vec!["test".to_string()]),
    };
    debug!("Creating project to delete: {}", project_id);
    server
        .post("/api/v1/project")
        .json(&project)
        .await
        .assert_status_ok();

    // Create some nodes for the project
    let node_id1 = Uuid::new_v4();
    let node1 = node::Model {
        project_id,
        id: node_id1,
        node_type: NodeType::Person,
        display: "Test Person".to_string(),
        value: "test".to_string(),
        updated: chrono::Utc::now(),
        notes: None,
        pos_x: None,
        pos_y: None,
    };
    let node_id2 = Uuid::new_v4();
    let node2 = node::Model {
        project_id,
        id: node_id2,
        node_type: NodeType::Email,
        display: "test@example.com".to_string(),
        value: "test@example.com".to_string(),
        updated: chrono::Utc::now(),
        notes: None,
        pos_x: None,
        pos_y: None,
    };

    server
        .post("/api/v1/node")
        .json(&node1)
        .await
        .assert_status_ok();
    server
        .post("/api/v1/node")
        .json(&node2)
        .await
        .assert_status_ok();

    // Verify nodes exist
    server
        .get(&format!("/api/v1/node/{}", node_id1))
        .await
        .assert_status_ok();
    server
        .get(&format!("/api/v1/node/{}", node_id2))
        .await
        .assert_status_ok();

    // Delete the project
    let res = server
        .delete(&format!("/api/v1/project/{}", project_id))
        .await;
    res.assert_status_ok();

    // Verify project is deleted
    let res = server
        .get(&format!("/api/v1/project/{}", project_id))
        .expect_failure()
        .await;
    assert_eq!(res.status_code(), 404);

    // Verify cascade deletion - nodes should also be deleted
    let res = server
        .get(&format!("/api/v1/node/{}", node_id1))
        .expect_failure()
        .await;
    assert_eq!(res.status_code(), 404);

    let res = server
        .get(&format!("/api/v1/node/{}", node_id2))
        .expect_failure()
        .await;
    assert_eq!(res.status_code(), 404);
}

#[tokio::test]
async fn test_api_delete_project_not_found() {
    let server = setup_test_server().await;

    // Try to delete non-existent project
    let res = server
        .delete(&format!("/api/v1/project/{}", Uuid::new_v4()))
        .expect_failure()
        .await;
    assert_eq!(res.status_code(), 404);
}

#[tokio::test]
async fn test_api_delete_inbox_project_blocked() {
    let server = setup_test_server().await;

    // Try to delete the Inbox project (nil UUID)
    let res = server
        .delete(&format!("/api/v1/project/{}", Uuid::nil()))
        .expect_failure()
        .await;
    assert_eq!(res.status_code(), 400);

    // Verify error message
    let body = res.text();
    assert!(body.contains("Cannot delete project with nil UUID"));

    // Verify the Inbox project still exists
    let res = server
        .get(&format!("/api/v1/project/{}", Uuid::nil()))
        .await;
    res.assert_status_ok();
    let project: project::Model = res.json();
    assert_eq!(project.id, Uuid::nil());
    assert_eq!(project.name, "Inbox");
}

#[tokio::test]
async fn test_handle_error() {
    use super::*;
    use axum::response::IntoResponse;
    let err = tower::timeout::error::Elapsed::new();
    let res = handle_error(Box::new(err)).await.into_response();
    let expected = (StatusCode::REQUEST_TIMEOUT, "request timed out").into_response();

    assert_eq!(res.status(), expected.status());

    let err = tower::load_shed::error::Overloaded::new();
    let res = handle_error(Box::new(err)).await.into_response();
    let expected = (
        StatusCode::SERVICE_UNAVAILABLE,
        "service is overloaded, try again later",
    )
        .into_response();

    assert_eq!(res.status(), expected.status());
}
