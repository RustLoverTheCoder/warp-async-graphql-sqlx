use server::config::configs::{Configs, LogConfig};
use tokio::time::Instant;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 启动计时器
    let instant = Instant::now();

    // 初始化配置
    let configs = Configs::init_config()?;

    // 初始日志
    LogConfig::init(&configs.log)?;

    log::info!("🎉Started Application in {:.3?}", instant.elapsed());

    Ok(())
}