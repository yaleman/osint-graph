use axum::Router;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    info(description = "OSINT Graph API Documentation", license(name = "MIT or Apache2", identifier="MIT Apache2.0"), title = "OSINT Graph", version = env!("CARGO_PKG_VERSION")),
    paths(
        crate::project::get_projects,
        crate::project::get_project,
        crate::project::post_project,
        crate::project::update_project,
        crate::project::delete_project,
        crate::project::export_project,
        crate::project::get_nodes_by_project,
        crate::project::get_node,
        crate::project::post_node,
        crate::project::update_node,
        crate::project::delete_node,
        crate::project::get_nodelinks_by_project,
        crate::project::post_nodelink,
        crate::project::delete_nodelink,
        crate::attachment::list_attachments,
        crate::attachment::upload_attachment,
        crate::attachment::view_attachment,
        crate::attachment::download_attachment,
        crate::attachment::update_attachment,
        crate::attachment::delete_attachment
    )
)]
pub struct ApiDoc;

pub(crate) fn api_route<T: Clone + Sync + Send + 'static>() -> Router<T> {
    let doc = ApiDoc::openapi();
    Router::new().merge(SwaggerUi::new("/api/v1/swagger-ui").url("/api/v1/openapi.json", doc))
}
