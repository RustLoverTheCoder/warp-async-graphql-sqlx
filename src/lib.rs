use std::{convert::Infallible, future::Future, sync::Arc};

use crate::config::configs::{Configs, CryptoConfig, DatabaseConfig, LogConfig};

use async_graphql_warp::BadRequest;
use regex::Regex;
use security::crypto::CryptoService;
use sqlx::{Pool, Postgres};
use warp::{hyper::StatusCode, Filter, Rejection};

pub mod common;
pub mod config;
pub mod domain;
pub mod repository;
pub mod security;
pub mod service;
pub mod web;

lazy_static::lazy_static! {
    // 配置文件
    static ref CONFIGS: Arc<Configs> = Configs::init_config().unwrap();

    // 数据库
    static ref POOL: Pool<Postgres> = DatabaseConfig::init(&CONFIGS.database).unwrap();

    // 加密工具
    static ref CRYPTO: Arc<CryptoService> = CryptoConfig::get_crypto_server(&CONFIGS.crypto);

    // 正则
    static ref EMAIL_REGEX: Regex = Regex::new(r"(@)").unwrap();
    static ref USERNAME_REGEX: Regex = Regex::new(r"^[a-zA-Z0-9_-]{4,16}$").unwrap();
}

/// http server application
pub struct Application;

impl Application {
    // 构建服务器
    pub async fn build() -> anyhow::Result<impl Future> {
        // 初始化
        lazy_static::initialize(&EMAIL_REGEX);
        lazy_static::initialize(&USERNAME_REGEX);
        log::info!("初始化 '静态常量' 完成");

        lazy_static::initialize(&CONFIGS);
        // 初始日志
        LogConfig::init(&CONFIGS.log).expect("日志初始化失败");
        lazy_static::initialize(&POOL);
        // 取下链接测试下
        POOL.acquire().await.expect("获取数据库连接失败");

        // graphql 入口
        let graphql = web::gql::graphql(CONFIGS.clone());

        // playground 入口
        let playground = web::gql::graphiql(CONFIGS.clone());

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

        let addr = CONFIGS.server.get_address();
        let serve = warp::serve(routes).run(addr);

        Ok(serve)
    }
}
