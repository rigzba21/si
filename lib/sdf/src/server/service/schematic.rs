use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::Json;
use axum::Router;
use dal::socket::{SocketError, SocketId};
use dal::{
    node::NodeId, schema::variant::SchemaVariantError, AttributeValueError, ComponentError,
    NodeError, NodeKind, NodeMenuError, NodePositionError, ReadTenancyError,
    SchemaError as DalSchemaError, SchematicError as DalSchematicError, SchematicKind,
    StandardModelError, TransactionsError,
};
use dal::{AttributeReadContext, WsEventError};
use thiserror::Error;

pub mod create_connection;
pub mod create_node;
pub mod get_node_add_menu;
pub mod get_node_template;
pub mod get_schematic;
pub mod list_schema_variants;
pub mod set_node_position;

#[derive(Debug, Error)]
pub enum SchematicError {
    #[error(transparent)]
    Nats(#[from] si_data::NatsError),
    #[error(transparent)]
    Pg(#[from] si_data::PgError),
    #[error(transparent)]
    PgPool(#[from] si_data::PgPoolError),
    #[error(transparent)]
    StandardModel(#[from] StandardModelError),
    #[error(transparent)]
    ContextTransaction(#[from] TransactionsError),
    #[error("schema error: {0}")]
    Schema(#[from] DalSchemaError),
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] AttributeValueError),
    #[error("attrubte value not found for context: {0:?}")]
    AttributeValueNotFoundForContext(AttributeReadContext),
    #[error("schema not found")]
    SchemaNotFound,
    #[error("component not found")]
    ComponentNotFound,
    #[error("node not found: {0}")]
    NodeNotFound(NodeId),
    #[error("schema variant not found")]
    SchemaVariantNotFound,
    #[error("node menu error: {0}")]
    NodeMenu(#[from] NodeMenuError),
    #[error("node error: {0}")]
    Node(#[from] NodeError),
    #[error("socket error: {0}")]
    Socket(#[from] SocketError),
    #[error("external provider not found for socket id: {0}")]
    ExternalProviderNotFoundForSocket(SocketId),
    #[error("internal provider not found for socket id: {0}")]
    InternalProviderNotFoundForSocket(SocketId),
    #[error("invalid request")]
    InvalidRequest,
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] SchemaVariantError),
    #[error("component error: {0}")]
    Component(#[from] ComponentError),
    #[error("node position error: {0}")]
    NodePosition(#[from] NodePositionError),
    #[error("dal schematic error: {0}")]
    SchematicError(#[from] DalSchematicError),
    #[error("read tenancy error: {0}")]
    ReadTenancy(#[from] ReadTenancyError),
    #[error("not authorized")]
    NotAuthorized,
    #[error("invalid system")]
    InvalidSystem,
    #[error("invalid schema kind ({0}) and parent node id pair ({1:?})")]
    InvalidSchematicKindParentNodeIdPair(SchematicKind, Option<NodeId>),
    #[error("parent node not found {0}")]
    ParentNodeNotFound(NodeId),
    #[error("invalid parent node kind {0:?}")]
    InvalidParentNode(NodeKind),
    #[error("ws event error: {0}")]
    WsEvent(#[from] WsEventError),
}

pub type SchematicResult<T> = std::result::Result<T, SchematicError>;

impl IntoResponse for SchematicError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            SchematicError::SchemaNotFound => (StatusCode::NOT_FOUND, self.to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        let body = Json(
            serde_json::json!({ "error": { "message": error_message, "code": 42, "statusCode": status.as_u16() } }),
        );

        (status, body).into_response()
    }
}

pub fn routes() -> Router {
    Router::new()
        .route("/get_schematic", get(get_schematic::get_schematic))
        .route(
            "/get_node_add_menu",
            post(get_node_add_menu::get_node_add_menu),
        )
        .route(
            "/get_node_template",
            get(get_node_template::get_node_template),
        )
        .route("/create_node", post(create_node::create_node))
        .route(
            "/set_node_position",
            post(set_node_position::set_node_position),
        )
        .route(
            "/create_connection",
            post(create_connection::create_connection),
        )
        .route(
            "/list_schema_variants",
            get(list_schema_variants::list_schema_variants),
        )
}
