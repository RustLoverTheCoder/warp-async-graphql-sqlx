use server::Application;
use tokio::time::Instant;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 启动计时器
    let instant = Instant::now();

    let application = Application::build().await?;

    log::info!("🎉Started Application in {:.3?}", instant.elapsed());

    application.await;

    Ok(())
}
