mod config;
mod metrics;
mod namespace;
mod oauth;
mod tables;
mod views;

pub use config::*;
pub(crate) use metrics::*;
pub use namespace::*;
pub use oauth::*;
pub use tables::*;
pub use views::*;

pub use iceberg_ext::catalog::{NamespaceIdent, TableIdent};

use crate::service::*;
pub use crate::types::*;
use axum::async_trait;
use axum::extract::{Form, Json, Path, Query, State};
pub use axum::routing::{get, post, put};
use http::HeaderMap;

pub trait V1Service<S: crate::service::State>
where
    Self: V1ConfigService<S>
        + V1NamespaceService<S>
        + V1OAuthService<S>
        + V1TablesService<S>
        + V1MetricsService<S>
        + V1ViewsService<S>
        + Send
        + Sync
        + Clone
        + 'static,
{
}

#[async_trait]
pub trait V1MetricsService<S: crate::service::State>
where
    Self: Send + Sync + Clone + 'static,
{
    /// List all table identifiers underneath a given namespace
    async fn report_metrics(
        parameters: TableParameters,
        request: serde_json::Value,
        state: ApiContext<S>,
        headers: HeaderMap,
    ) -> Result<ListTablesResponse>;
}

#[async_trait]
pub trait V1ViewsService<S: crate::service::State>
where
    Self: Send + Sync + Clone + 'static,
{
    /// List all views underneath a given namespace
    async fn list_views(
        parameters: NamespaceParameters,
        query: PaginationQuery,
        state: ApiContext<S>,
        headers: HeaderMap,
    ) -> Result<ListTablesResponse>;

    /// Create a view in the given namespace
    async fn create_view(
        parameters: NamespaceParameters,
        request: CreateViewRequest,
        state: ApiContext<S>,
        headers: HeaderMap,
    ) -> Result<LoadViewResult>;
}

#[async_trait]
pub trait V1TablesService<S: crate::service::State>
where
    Self: Send + Sync + Clone + 'static,
{
    /// List all table identifiers underneath a given namespace
    async fn list_tables(
        parameters: NamespaceParameters,
        query: PaginationQuery,
        state: ApiContext<S>,
        headers: HeaderMap,
    ) -> Result<ListTablesResponse>;

    /// Create a table in the given namespace
    async fn create_table(
        parameters: NamespaceParameters,
        request: CreateTableRequest,
        state: ApiContext<S>,
        headers: HeaderMap,
    ) -> Result<LoadTableResult>;

    /// Register a table in the given namespace using given metadata file location
    async fn register_table(
        parameters: NamespaceParameters,
        request: RegisterTableRequest,
        state: ApiContext<S>,
        headers: HeaderMap,
    ) -> Result<LoadTableResult>;

    /// Load a table from the catalog
    async fn load_table(
        parameters: TableParameters,
        state: ApiContext<S>,
        headers: HeaderMap,
    ) -> Result<LoadTableResult>;

    /// Commit updates to a table
    async fn commit_table(
        parameters: TableParameters,
        request: CommitTableRequest,
        state: ApiContext<S>,
        headers: HeaderMap,
    ) -> Result<CommitTableResponse>;

    /// Drop a table from the catalog
    async fn drop_table(
        parameters: TableParameters,
        state: ApiContext<S>,
        headers: HeaderMap,
    ) -> Result<()>;

    /// Check if a table exists
    async fn table_exists(
        parameters: TableParameters,
        state: ApiContext<S>,
        headers: HeaderMap,
    ) -> Result<()>;

    /// Rename a table
    async fn rename_table(
        prefix: Option<Prefix>,
        request: RenameTableRequest,
        state: ApiContext<S>,
        headers: HeaderMap,
    ) -> Result<()>;

    /// Commit updates to multiple tables in an atomic operation
    async fn commit_transaction(
        prefix: Option<Prefix>,
        request: CommitTransactionRequest,
        state: ApiContext<S>,
        headers: HeaderMap,
    ) -> Result<()>;
}

#[async_trait]
pub trait V1NamespaceService<S: crate::service::State>
where
    Self: Send + Sync + Clone + 'static,
{
    /// List all namespaces at a certain level, optionally starting from a given parent namespace.
    /// If table accounting.tax.paid.info exists, using 'SELECT NAMESPACE IN accounting'
    /// would translate into `GET /namespaces?parent=accounting` and must return a namespace,
    /// ["accounting", "tax"] only. Using 'SELECT NAMESPACE IN accounting.tax' would translate into `GET /namespaces?parent=accounting%1Ftax` and must return a namespace, ["accounting", "tax", "paid"]. If `parent` is not provided, all top-level namespaces should be listed.
    async fn list_namespaces(
        prefix: Option<Prefix>,
        query: ListNamespacesQuery,
        state: ApiContext<S>,
        headers: HeaderMap,
    ) -> Result<ListNamespacesResponse>;

    /// Create a namespace, with an optional set of properties.
    /// The server might also add properties, such as `last_modified_time` etc.
    async fn create_namespace(
        prefix: Option<Prefix>,
        request: CreateNamespaceRequest,
        state: ApiContext<S>,
        headers: HeaderMap,
    ) -> Result<CreateNamespaceResponse>;

    /// Return all stored metadata properties for a given namespace
    async fn load_namespace_metadata(
        parameters: NamespaceParameters,
        state: ApiContext<S>,
        headers: HeaderMap,
    ) -> Result<GetNamespaceResponse>;

    /// Check if a namespace exists
    async fn namespace_exists(
        parameters: NamespaceParameters,
        state: ApiContext<S>,
        headers: HeaderMap,
    ) -> Result<()>;

    /// Drop a namespace from the catalog. Namespace must be empty.
    async fn drop_namespace(
        parameters: NamespaceParameters,
        state: ApiContext<S>,
        headers: HeaderMap,
    ) -> Result<()>;

    /// Set or remove properties on a namespace
    async fn update_namespace_properties(
        parameters: NamespaceParameters,
        request: UpdateNamespacePropertiesRequest,
        state: ApiContext<S>,
        headers: HeaderMap,
    ) -> Result<UpdateNamespacePropertiesResponse>;
}
