use async_graphql::extensions::{ApolloTracing, Logger};
use async_graphql::{
    http::{playground_source, GraphQLPlaygroundConfig},
    Request,
};
use async_graphql::{EmptySubscription, Schema};

use mutations::MutationRoot;
use queries::QueryRoot;
use warp::{
    http::{Error, Response},
    Filter, Rejection,
};

use crate::{config::configs::Configs, State};
use std::{convert::Infallible, sync::Arc};

pub mod mutations;
pub mod queries;

/// 为了代码简洁, 定义 `ServiceSchema`
pub type ServiceSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

/// 定义返回
pub type GraphqlResult<T> = std::result::Result<T, async_graphql::Error>;

// graphql 入口
pub fn graphql(
    config: Arc<Configs>,
    state: Arc<State>,
) -> impl Filter<Extract = (async_graphql_warp::Response,), Error = Rejection> + Clone {
    let mut schema = Schema::build(
        QueryRoot::default(),
        MutationRoot::default(),
        EmptySubscription,
    )
    .data(state)
    .extension(Logger);

    // 是否开启 ApolloTracing
    if config.graphql.tracing.unwrap_or(false) {
        schema = schema.extension(ApolloTracing);
    }

    warp::path(config.graphql.path.clone())
        .and(async_graphql_warp::graphql(schema.finish()))
        .and_then(|(schema, request): (ServiceSchema, Request)| async move {
            Ok::<_, Infallible>(async_graphql_warp::Response::from(
                schema.execute(request).await,
            ))
        })
}

// GraphQLPlayground 入口
pub fn graphiql(
    config: Arc<Configs>,
) -> impl Filter<Extract = (Result<Response<String>, Error>,), Error = Rejection> + Clone {
    let path = config.graphql.graphiql.path.clone();

    log::info!(
        "🚀GraphQL UI: http://{}:{}/{}",
        config.server.host,
        config.server.port,
        &path
    );

    warp::path(path.clone()).and(warp::get()).map(move || {
        Response::builder()
            .header("content-type", "text/html")
            .body(playground_source(GraphQLPlaygroundConfig::new(
                &config.graphql.path,
            )))
    })
}
