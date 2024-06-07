use std::marker::PhantomData;

use axum::{extract::State as AxumState, routing::post, Json, Router};
use iceberg_rest_service::ApiContext;

use crate::service::event_publisher::EventPublisher;
use crate::service::{auth::AuthZHandler, secrets::SecretStore, Catalog, State};

#[derive(Clone, Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct ApiServer<C: Catalog, A: AuthZHandler, S: SecretStore, P: EventPublisher> {
    auth_handler: PhantomData<A>,
    config_server: PhantomData<C>,
    secret_store: PhantomData<S>,
    publisher: PhantomData<P>,
}

pub mod v1 {
    use super::{
        post, ApiContext, AuthZHandler, AxumState, Catalog, Json, Router, SecretStore, State,
    };
    use axum::Extension;
    use iceberg_rest_service::RequestMetadata;
    pub mod warehouse;
    use crate::service::event_publisher::EventPublisher;
    use warehouse::WarehouseService;

    impl<C: Catalog, A: AuthZHandler, S: SecretStore, P: EventPublisher> super::ApiServer<C, A, S, P> {
        pub fn v1_router() -> Router<ApiContext<State<A, C, S, P>>> {
            Router::new()
            // List Namespaces
            .route(
                "/warehouse",
                // List Namespaces
                post(
                    |AxumState(api_context): AxumState<ApiContext<State<A, C, S, P>>>,
                     Extension(metadata): Extension<RequestMetadata>,
                     Json(request): Json<warehouse::CreateWarehouseRequest>| {
                        Self::create_warehouse(request, api_context, metadata)
                    },
                ),
            )
        }
    }
}
