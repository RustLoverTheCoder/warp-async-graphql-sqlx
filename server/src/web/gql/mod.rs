use async_graphql::{http::{playground_source, GraphQLPlaygroundConfig}};
use async_graphql::{
    extensions::{ApolloTracing, Logger},
    EmptyMutation,
};
use async_graphql::{EmptySubscription, Schema};

use queries::QueryRoot;
use warp::{http::Response, Filter};

use crate::config::configs::Configs;
use std::{convert::Infallible, sync::Arc};

pub mod queries;

/// 为了代码简洁, 定义 `ServiceSchema`
pub type ServiceSchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;

/// 定义返回
pub type GraphqlResult<T> = std::result::Result<T, async_graphql::Error>;

// graphql 入口
pub fn graphql(config: Arc<Configs>) -> impl Filter + Clone {
    let mut schema =
        Schema::build(QueryRoot::default(), EmptyMutation, EmptySubscription).extension(Logger);

    // 是否开启 ApolloTracing
    if config.graphql.tracing.unwrap_or(false) {
        schema = schema.extension(ApolloTracing);
    }

    async_graphql_warp::graphql(schema.finish()).and_then(
        |(schema, request): (ServiceSchema, async_graphql::Request)| async move {
            Ok::<_, Infallible>(async_graphql::Response::from(schema.execute(request).await))
        },
    )
}

// GraphQLPlayground 入口
pub fn graphiql(config: Arc<Configs>) -> impl Filter {
    let path = config.graphql.graphiql.path.clone();

    warp::path(path.clone()).and(warp::get()).map(move || {
        Response::builder()
            .header("content-type", "text/html")
            .body(playground_source(GraphQLPlaygroundConfig::new(&path)))
    })
}
