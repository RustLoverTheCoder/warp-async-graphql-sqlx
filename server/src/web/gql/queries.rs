use async_graphql::*;

use crate::web::gql::GraphqlResult;

/// 定义查询根节点
#[derive(MergedObject, Default)]
pub struct QueryRoot(PingQuery);

/// ping Query
#[derive(Default)]
pub struct PingQuery;

#[Object]
impl PingQuery {
    async fn ping(&self) -> GraphqlResult<String> {
        Ok("pong".to_string())
    }
}