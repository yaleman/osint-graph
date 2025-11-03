use std::sync::{Arc, Once};

use crate::{build_app, AppState};
use axum_test::*;
use osint_graph_shared::node::Node;
use osint_graph_shared::project::Project;
use tokio::sync::RwLock;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use uuid::Uuid;

static INIT: Once = Once::new();

async fn setup_test_server() -> TestServer {
    INIT.call_once(|| {
        tracing_subscriber::registry()
            .with(tracing_subscriber::EnvFilter::new(
                "osint_graph_backend=debug,tower_http=debug",
            ))
            .with(tracing_subscriber::fmt::layer())
            .init();
    });

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
    crate::storage::start_db(
        Some(format!(
            "/asdfasdf{}/asd{}fsadfdf",
            Uuid::new_v4(),
            Uuid::new_v4()
        )),
        None,
    )
    .await
    .expect_err("Should fail to open DB");
}

#[tokio::test]
async fn test_api_project_node_save_load() {
    let server = setup_test_server().await;

    let node_id = Uuid::new_v4();
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

    let res = server.get(&format!("/api/v1/project/{}", project_id)).await;
    res.assert_status_ok();

    let res = server
        .post("/api/v1/node")
        .json(&Node {
            project_id,
            id: node_id,
            node_type: "person".to_string(),
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
    assert!(!res.json::<Vec<Project>>().is_empty());

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
    let project1 = Project {
        id: project_id,
        name: "Test Project 1".to_string(),
        user: Uuid::new_v4(),
        creationdate: chrono::Utc::now(),
        last_updated: None,
        nodes: Default::default(),
    };

    // Create second project
    let project2 = Project {
        id: other_project_id,
        name: "Test Project 2".to_string(),
        user: Uuid::new_v4(),
        creationdate: chrono::Utc::now(),
        last_updated: None,
        nodes: Default::default(),
    };

    // Create both projects
    server
        .post("/api/v1/project")
        .json(&project1)
        .await
        .assert_status_ok();
    server
        .post("/api/v1/project")
        .json(&project2)
        .await
        .assert_status_ok();

    // Test getting nodes from empty project
    let res = server
        .get(&format!("/api/v1/project/{}/nodes", project_id))
        .await;
    res.assert_status_ok();
    let nodes: Vec<Node> = res.json();
    assert!(nodes.is_empty());

    // Create nodes for first project
    let node1 = Node {
        project_id,
        id: node_id1,
        node_type: "person".to_string(),
        display: "John Doe".to_string(),
        value: "john@example.com".to_string(),
        updated: chrono::Utc::now(),
        notes: Some("First person".to_string()),
        pos_x: Some(100),
        pos_y: Some(200),
    };

    let node2 = Node {
        project_id,
        id: node_id2,
        node_type: "domain".to_string(),
        display: "example.com".to_string(),
        value: "example.com".to_string(),
        updated: chrono::Utc::now(),
        notes: Some("Domain node".to_string()),
        pos_x: Some(300),
        pos_y: Some(400),
    };

    // Create node for second project
    let other_node = Node {
        project_id: other_project_id,
        id: other_node_id,
        node_type: "ip".to_string(),
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
    let nodes: Vec<Node> = res.json();
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
    let nodes: Vec<Node> = res.json();
    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0].id, other_node_id);

    // Test getting nodes for non-existent project
    let res = server
        .get(&format!("/api/v1/project/{}/nodes", Uuid::new_v4()))
        .await;
    res.assert_status_ok();
    let nodes: Vec<Node> = res.json();
    assert!(nodes.is_empty());
}

#[tokio::test]
async fn test_api_projects_crud() {
    let server = setup_test_server().await;

    // Test getting all projects (should include default project)
    let res = server.get("/api/v1/projects").await;
    res.assert_status_ok();
    let initial_projects: Vec<Project> = res.json();
    let initial_count = initial_projects.len();

    // Create a new project
    let project_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let project = Project {
        id: project_id,
        name: "CRUD Test Project".to_string(),
        user: user_id,
        creationdate: chrono::Utc::now(),
        last_updated: None,
        nodes: Default::default(),
    };

    // Test project creation
    let res = server.post("/api/v1/project").json(&project).await;
    res.assert_status_ok();

    // Test getting all projects (should have one more)
    let res = server.get("/api/v1/projects").await;
    res.assert_status_ok();
    let projects: Vec<Project> = res.json();
    assert_eq!(projects.len(), initial_count + 1);

    // Test getting specific project
    let res = server.get(&format!("/api/v1/project/{}", project_id)).await;
    res.assert_status_ok();
    let retrieved_project: Project = res.json();
    assert_eq!(retrieved_project.id, project_id);
    assert_eq!(retrieved_project.name, "CRUD Test Project");
    assert_eq!(retrieved_project.user, user_id);

    // Test getting non-existent project
    let res = server
        .get(&format!("/api/v1/project/{}", Uuid::new_v4()))
        .expect_failure()
        .await;
    assert_eq!(res.status_code(), 404);
}

#[tokio::test]
async fn test_api_nodes_crud() {
    let server = setup_test_server().await;

    // Create a project first
    let project_id = Uuid::new_v4();
    let project = Project {
        id: project_id,
        name: "Node CRUD Test".to_string(),
        user: Uuid::new_v4(),
        creationdate: chrono::Utc::now(),
        last_updated: None,
        nodes: Default::default(),
    };
    server
        .post("/api/v1/project")
        .json(&project)
        .await
        .assert_status_ok();

    // Test node creation
    let node_id = Uuid::new_v4();
    let node = Node {
        project_id,
        id: node_id,
        node_type: "email".to_string(),
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
    let retrieved_node: Node = res.json();
    assert_eq!(retrieved_node.id, node_id);
    assert_eq!(retrieved_node.project_id, project_id);
    assert_eq!(retrieved_node.node_type, "email");
    assert_eq!(retrieved_node.display, "test@example.com");
    assert_eq!(retrieved_node.value, "test@example.com");
    assert_eq!(retrieved_node.notes, Some("Test email node".to_string()));
    assert_eq!(retrieved_node.pos_x, Some(150));
    assert_eq!(retrieved_node.pos_y, Some(250));

    // Test updating node (using same endpoint)
    let updated_node = Node {
        project_id,
        id: node_id,
        node_type: "email".to_string(),
        display: "updated@example.com".to_string(),
        value: "updated@example.com".to_string(),
        updated: chrono::Utc::now(),
        notes: Some("Updated test email node".to_string()),
        pos_x: Some(300),
        pos_y: Some(400),
    };

    let res = server.post("/api/v1/node").json(&updated_node).await;
    res.assert_status_ok();

    // Verify the update
    let res = server.get(&format!("/api/v1/node/{}", node_id)).await;
    res.assert_status_ok();
    let retrieved_node: Node = res.json();
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
    let node = Node {
        project_id: non_existent_project_id,
        id: node_id,
        node_type: "person".to_string(),
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
