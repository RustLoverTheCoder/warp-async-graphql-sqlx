use std::{convert::Infallible, future::Future, sync::Arc};

use async_graphql_warp::BadRequest;
use regex::Regex;
use security::crypto::CryptoService;
use sqlx::{PgPool, Pool, Postgres};
use warp::{hyper::StatusCode, Filter, Rejection};
use async_graphql::Context as GraphQLContext;
use crate::{config::configs::{Configs, CryptoConfig, DatabaseConfig, LogConfig}, web::gql::GraphqlResult};



pub mod config;
pub mod security;
pub mod web;
pub mod domain;
pub mod repository;
pub mod service;
pub mod common;

lazy_static::lazy_static! {
    // 正则
    static ref EMAIL_REGEX: Regex = Regex::new(r"(@)").unwrap();
    static ref USERNAME_REGEX: Regex = Regex::new(r"^[a-zA-Z0-9_-]{4,16}$").unwrap();
}

/// 全局的 state
pub struct State {
    // 数据库连接池
    pool: Arc<PgPool>,
    // 加密服务
    crypto: Arc<CryptoService>,
}

impl State {
    // 通过 GraphQLContext 获取 数据库连接池
    pub fn get_pool(ctx: &GraphQLContext<'_>) -> GraphqlResult<Arc<Pool<Postgres>>> {
        Ok(ctx.data::<Arc<State>>()?.pool.clone())
    }

    // 通过 GraphQLContext 获取 加密服务
    pub fn get_crypto_server(ctx: &GraphQLContext<'_>) -> GraphqlResult<Arc<CryptoService>> {
        Ok(ctx.data::<Arc<State>>()?.crypto.clone())
    }
}

/// http server application
pub struct Application;

impl Application {
    // 构建服务器
    pub async fn build() -> anyhow::Result<impl Future> {
         // 初始化静态常量
         lazy_static::initialize(&EMAIL_REGEX);
         lazy_static::initialize(&USERNAME_REGEX);
         log::info!("初始化 '静态常量' 完成");

         let configs = Configs::init_config()?;

         // 初始日志
         LogConfig::init(&configs.log)?;

         let pool = DatabaseConfig::init(&configs.database).await.unwrap();
         let crypto = CryptoConfig::get_crypto_server(&configs.crypto);
         let state = Arc::new(State { pool, crypto });

        // graphql 入口
        let graphql = web::gql::graphql(configs.clone(), state.clone());

        // playground 入口
        let playground = web::gql::graphiql(configs.clone());

        // 错误处理
        let recover = |err: Rejection| async move {
            if let Some(BadRequest(err)) = err.find() {
                return Ok::<_, Infallible>(warp::reply::with_status(
                    err.to_string(),
                    StatusCode::BAD_REQUEST,
                ));
            }

            Ok(warp::reply::with_status(
                "INTERNAL_SERVER_ERROR".to_string(),
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        };

        let routes = playground
            // graphql 入口
            .or(graphql)
            // 错误处理
            .recover(recover);

        let addr = configs.server.get_address();
        let serve = warp::serve(routes).run(addr);

        Ok(serve)
    }
}
