use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::engine::WgpuApp;

#[async_trait]
pub trait WgpuAppModule {
    fn new() -> Self
    where
        Self: Sized;

    async fn probe(&mut self, app: Arc<Mutex<WgpuApp>>) -> anyhow::Result<()>;

    fn update(&mut self, queue: &wgpu::Queue, dt: std::time::Duration) -> anyhow::Result<()> {
        Ok(())
    }

    fn render(&self) -> anyhow::Result<()> {
        Ok(())
    }
}
