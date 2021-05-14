use std::{convert::Infallible, future::Future, sync::Arc};

use async_graphql_warp::BadRequest;
use regex::Regex;
use security::crypto::CryptoService;
use sqlx::{Pool, Postgres};
use warp::{Filter, Rejection, hyper::StatusCode};

use crate::config::configs::{Configs, CryptoConfig, DatabaseConfig, LogConfig};

pub mod config;
pub mod security;
pub mod web;

lazy_static::lazy_static! {
    // 配置文件
    static ref CONFIGS: Arc<Configs> = Configs::init_config().unwrap();

    // 数据库
    static ref POOL: Pool<Postgres> = DatabaseConfig::init(&CONFIGS.database).unwrap();

    // 加密工具
    static ref CRYPTO: Arc<CryptoService> = CryptoConfig::get_crypto_server(&CONFIGS.crypto);

    // 正则
    static ref REGEXS: Regexs = {
        log::info!("初始化 '正则工具: [REGEXS]' ");
        Regexs {
            username: Regex::new(r"(@)").unwrap(),
            email: Regex::new(r"^[a-zA-Z0-9_-]{4,16}$").unwrap(),
        }
    };
}

pub struct Regexs {
    pub username: Regex,
    pub email: Regex,
}

/// http server application
pub struct Application;

impl Application {
    // 构建服务器
    pub async fn build() -> anyhow::Result<impl Future> {
        // 初始化配置
        lazy_static::initialize(&CONFIGS);

        // 初始日志
        LogConfig::init(&CONFIGS.log)?;

        // 正则
        lazy_static::initialize(&REGEXS);

        // 加密工具
        lazy_static::initialize(&CRYPTO);

        // 初始化数据库
        lazy_static::initialize(&POOL);
        // 验证数据库连接
        DatabaseConfig::check(&POOL).await;

        // graphql 入口
        let graphql = web::gql::graphql(CONFIGS.clone());

        // playground 入口
        let graphql_playground = web::gql::graphiql(CONFIGS.clone());

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

        let routes = graphql_playground.or(graphql).recover(recover);

        let serve = warp::serve(routes).run(([0, 0, 0, 0], 8000));

        Ok(serve)
    }
}
