use async_trait::async_trait;
use tauri::Runtime;
use tauri_plugin_background_service::{BackgroundService, ServiceContext, ServiceError};

pub struct ReadModeService {
    is_running: bool,
}

impl ReadModeService {
    pub fn new() -> Self {
        Self { is_running: false }
    }
}

#[async_trait]
impl<R: Runtime> BackgroundService<R> for ReadModeService {
    async fn init(&mut self, _ctx: &ServiceContext<R>) -> Result<(), ServiceError> {
        Ok(())
    }

    async fn run(&mut self, ctx: &ServiceContext<R>) -> Result<(), ServiceError> {
        self.is_running = true;
        ctx.shutdown.cancelled().await;
        self.is_running = false;
        Ok(())
    }
}
