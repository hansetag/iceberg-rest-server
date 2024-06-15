use crate::api::management::ApiServer;
use crate::api::{ApiContext, Result};
use crate::request_metadata::RequestMetadata;
use crate::service::storage::{StorageCredential, StorageProfile};
use crate::service::{auth::AuthZHandler, secrets::SecretStore, Catalog, State, Transaction};
use crate::{request_metadata, ProjectIdent, WarehouseIdent};
use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ToSchema)]
#[serde(rename_all = "kebab-case")]
pub struct CreateWarehouseRequest {
    /// Name of the warehouse to create. Must be unique
    /// within a project.
    pub warehouse_name: String,
    /// Project ID in which to create the warehouse.
    pub project_id: uuid::Uuid,
    /// Storage profile to use for the warehouse.
    pub storage_profile: StorageProfile,
    /// Optional storage credential to use for the warehouse.
    pub storage_credential: Option<StorageCredential>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ToSchema)]
#[serde(rename_all = "kebab-case")]
pub struct CreateWarehouseResponse {
    /// ID of the created warehouse.
    pub warehouse_id: uuid::Uuid,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ToSchema)]
#[serde(rename_all = "kebab-case")]
pub struct UpdateWarehouseStorageRequest {
    /// Storage profile to use for the warehouse.
    pub storage_profile: Option<StorageProfile>,
    /// Optional storage credential to use for the warehouse.
    /// The existing credential is not re-used. If no credential is
    /// provided, we assume that this storage does not require credentials.
    pub storage_credential: Option<StorageCredential>,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "kebab-case")]
pub struct ListWarehouseRequest {
    /// Optional filter to include inactive warehouses.
    #[serde(default)]
    pub include_inactive: Option<bool>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ToSchema)]
#[serde(rename_all = "kebab-case")]
pub struct ListProjectResponse {
    /// List of project IDs.
    pub project_ids: Vec<uuid::Uuid>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ToSchema)]
#[serde(rename_all = "kebab-case")]
pub struct WarehouseResponse {
    /// ID of the warehouse.
    pub id: uuid::Uuid,
    /// Name of the warehouse.
    pub name: String,
    /// Project ID in which the warehouse is created.
    pub project_id: uuid::Uuid,
    /// Storage profile used for the warehouse.
    pub storage_profile: StorageProfile,
    /// Whether the warehouse is active.
    pub status: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ToSchema)]
#[serde(rename_all = "kebab-case")]
pub struct ListWarehouseResponse {
    /// List of warehouses in the project.
    pub warehouses: Vec<WarehouseResponse>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ToSchema)]
#[serde(rename_all = "kebab-case")]
pub struct UpdateWarehouseCredentialRequest {
    /// New storage credential to use for the warehouse.
    /// If not specified, the existing credential is removed.
    pub new_storage_credential: Option<StorageCredential>,
}

impl axum::response::IntoResponse for CreateWarehouseResponse {
    fn into_response(self) -> axum::http::Response<axum::body::Body> {
        axum::Json(self).into_response()
    }
}

impl<C: Catalog, A: AuthZHandler, S: SecretStore> Service<C, A, S> for ApiServer<C, A, S> {}

#[async_trait::async_trait]
#[allow(clippy::module_name_repetitions)]
pub trait Service<C: Catalog, A: AuthZHandler, S: SecretStore> {
    async fn create_warehouse(
        request: CreateWarehouseRequest,
        context: ApiContext<State<A, C, S>>,
        request_metadata: RequestMetadata,
    ) -> Result<CreateWarehouseResponse> {
        let CreateWarehouseRequest {
            warehouse_name,
            project_id,
            mut storage_profile,
            storage_credential,
        } = request;
        let project_ident = ProjectIdent::from(project_id.clone());

        // ------------------- AuthZ -------------------
        A::check_create_warehouse(&request_metadata, &project_ident, context.v1_state.auth).await?;

        // ------------------- Business Logic -------------------
        storage_profile
            .validate(storage_credential.as_ref())
            .await?;

        let mut transaction = C::Transaction::begin_write(context.v1_state.catalog).await?;
        let secret_id = if let Some(storage_credential) = storage_credential {
            Some(S::create_secret(storage_credential, context.v1_state.secrets).await?)
        } else {
            None
        };

        let warehouse_id = C::create_warehouse_profile(
            warehouse_name,
            project_id.into(),
            storage_profile,
            secret_id,
            transaction.transaction(),
        )
        .await?;

        transaction.commit().await?;

        Ok(CreateWarehouseResponse {
            warehouse_id: warehouse_id.into_uuid(),
        })
    }

    async fn list_projects(
        context: ApiContext<State<A, C, S>>,
        request_metadata: RequestMetadata,
    ) -> Result<ListProjectResponse> {
        // ------------------- AuthZ -------------------
        let projects = A::check_list_projects(&request_metadata, context.v1_state.auth).await?;

        // ------------------- Business Logic -------------------
        if let Some(projects) = projects {
            return Ok(ListProjectResponse {
                project_ids: projects.into_iter().map(|id| id.into_uuid()).collect(),
            });
        }

        let projects = C::list_projects(context.v1_state.catalog).await?;
        Ok(ListProjectResponse {
            project_ids: projects.into_iter().map(|id| id.into_uuid()).collect(),
        })
    }

    async fn list_warehouses(
        project_id: ProjectIdent,
        request: ListWarehouseRequest,
        context: ApiContext<State<A, C, S>>,
        request_metadata: RequestMetadata,
    ) -> Result<ListWarehouseResponse> {
        // ------------------- AuthZ -------------------
        let warehouses = A::check_list_warehouse_in_project(
            &request_metadata,
            &project_id,
            context.v1_state.auth,
        )
        .await?;

        // ------------------- Business Logic -------------------
        let warehouses = C::list_warehouses(
            &project_id.into(),
            request.include_inactive.unwrap_or(false),
            warehouses.as_ref(),
            context.v1_state.catalog,
        )
        .await?;

        Ok(ListWarehouseResponse {
            warehouses: warehouses
                .into_iter()
                .map(|warehouse| warehouse.into())
                .collect(),
        })
    }

    async fn get_warehouse(
        warehouse_id: WarehouseIdent,
        context: ApiContext<State<A, C, S>>,
        request_metadata: RequestMetadata,
    ) -> Result<WarehouseResponse> {
        // ------------------- AuthZ -------------------
        A::check_get_warehouse(&request_metadata, &warehouse_id, context.v1_state.auth).await?;

        // ------------------- Business Logic -------------------

        let warehouses =
            C::get_warehouse_metadata(&warehouse_id.into(), context.v1_state.catalog).await?;

        Ok(warehouses.into())
    }

    async fn delete_warehouse(
        warehouse_id: WarehouseIdent,
        context: ApiContext<State<A, C, S>>,
        request_metadata: RequestMetadata,
    ) -> Result<()> {
        // ------------------- AuthZ -------------------
        A::check_delete_warehouse(&request_metadata, &warehouse_id, context.v1_state.auth).await?;

        // ------------------- Business Logic -------------------
        let mut transaction = C::Transaction::begin_write(context.v1_state.catalog).await?;

        C::delete_warehouse(&warehouse_id.into(), transaction.transaction()).await?;

        transaction.commit().await?;

        Ok(())
    }
    async fn rename_warehouse(
        warehouse_id: WarehouseIdent,
        context: ApiContext<State<A, C, S>>,
        request_metadata: RequestMetadata,
    ) -> Result<http::StatusCode> {
        // ------------------- AuthZ -------------------
        A::check_rename_warehouse(&request_metadata, &warehouse_id, context.v1_state.auth).await?;

        // ------------------- Business Logic -------------------
        let mut transaction = C::Transaction::begin_write(context.v1_state.catalog).await?;

        C::rename_warehouse(&warehouse_id.into(), transaction.transaction()).await?;

        transaction.commit().await?;

        Ok(http::StatusCode::OK)
    }

    async fn deactivate_warehouse(
        warehouse_id: WarehouseIdent,
        context: ApiContext<State<A, C, S>>,
        _request_metadata: RequestMetadata,
    ) -> Result<()> {
        todo!()
    }

    async fn activate_warehouse(
        warehouse_id: WarehouseIdent,
        context: ApiContext<State<A, C, S>>,
        _request_metadata: RequestMetadata,
    ) -> Result<()> {
        todo!()
    }

    async fn update_storage(
        warehouse_id: WarehouseIdent,
        request: UpdateWarehouseStorageRequest,
        context: ApiContext<State<A, C, S>>,
        _request_metadata: RequestMetadata,
    ) -> Result<()> {
        todo!()
    }

    async fn update_credential(
        warehouse_id: WarehouseIdent,
        request: UpdateWarehouseCredentialRequest,
        context: ApiContext<State<A, C, S>>,
        _request_metadata: RequestMetadata,
    ) -> Result<()> {
        todo!()
    }
}

impl axum::response::IntoResponse for ListProjectResponse {
    fn into_response(self) -> axum::http::Response<axum::body::Body> {
        axum::Json(self).into_response()
    }
}

impl axum::response::IntoResponse for ListWarehouseResponse {
    fn into_response(self) -> axum::http::Response<axum::body::Body> {
        axum::Json(self).into_response()
    }
}

impl axum::response::IntoResponse for WarehouseResponse {
    fn into_response(self) -> axum::http::Response<axum::body::Body> {
        axum::Json(self).into_response()
    }
}

impl From<crate::service::WarehouseResponse> for WarehouseResponse {
    fn from(warehouse: crate::service::WarehouseResponse) -> Self {
        Self {
            id: warehouse.id.into_uuid(),
            name: warehouse.name,
            project_id: warehouse.project_id.into_uuid(),
            storage_profile: warehouse.storage_profile,
            status: warehouse.status.to_string(),
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_de_create_warehouse_request() {
        let request = serde_json::json!({
            "warehouse-name": "test_warehouse",
            "project-id": "f47ac10b-58cc-4372-a567-0e02b2c3d479",
            "storage-profile": {
                "type": "s3",
                "bucket": "test",
                "region": "dummy",
                "path-style-access": true,
                "endpoint": "http://localhost:9000",
            },
            "storage-credential": {
                "type": "s3",
                "credential-type": "access-key",
                "aws-access-key-id": "test-access-key-id",
                "aws-secret-access-key": "test-secret-access-key",
            },
        });

        let request: super::CreateWarehouseRequest = serde_json::from_value(request).unwrap();
        assert_eq!(request.warehouse_name, "test_warehouse");
        assert_eq!(
            request.project_id,
            uuid::Uuid::parse_str("f47ac10b-58cc-4372-a567-0e02b2c3d479").unwrap()
        );
        let s3_profile = request.storage_profile.try_into_s3(409).unwrap();
        assert_eq!(s3_profile.bucket, "test");
        assert_eq!(s3_profile.region, "dummy");
        assert_eq!(s3_profile.path_style_access, Some(true));
    }
}
