use anyhow::Result;
use async_trait::async_trait;
use sqlx::PgPool;

use crate::domain::users::{NewUser, Users};
use crate::repository::users::{ExtUsersRepository, UsersRepository};

pub struct UsersService;

#[async_trait]
pub trait ExtUsersService {
    /// 注册用户
    async fn user_register(new_user: &NewUser, password_hash: &str) -> Result<Users>;

    /// 根据用户名查询用户
    async fn find_by_username(username: &str) -> Result<Option<Users>>;

    /// 根据邮箱查询查询用户
    async fn find_by_email(email: &str) -> Result<Option<Users>>;

    /// 根据用户名查询用户2
    async fn find_by_username2(username: &str) -> Result<Users>;

    /// 检查用户是否存在
    async fn exists_by_username(username: &str) -> Result<bool>;

    /// 检查邮箱是否存在
    async fn exists_by_email(email: &str) -> Result<bool>;
}

#[async_trait]
impl ExtUsersService for UsersService {
    async fn user_register(new_user: &NewUser, password_hash: &str) -> Result<Users> {
        UsersRepository::create(new_user, password_hash).await
    }

    async fn find_by_username(username: &str) -> Result<Option<Users>> {
        UsersRepository::find_by_username(username).await
    }

    async fn find_by_email(email: &str) -> Result<Option<Users>> {
        UsersRepository::find_by_email(email).await
    }

    async fn find_by_username2(username: &str) -> Result<Users> {
        UsersRepository::find_by_username2(username).await
    }

    async fn exists_by_username(username: &str) -> Result<bool> {
        UsersRepository::exists_by_username(username).await
    }

    async fn exists_by_email(email: &str) -> Result<bool> {
        UsersRepository::exists_by_email(email).await
    }
}
