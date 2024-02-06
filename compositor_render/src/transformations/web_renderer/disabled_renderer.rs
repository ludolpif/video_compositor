use std::sync::Arc;

use crate::{
    state::{RegisterCtx, RenderCtx},
    wgpu::texture::NodeTexture,
    Resolution,
};

use super::WebRendererSpec;

#[derive(Debug)]
pub struct WebRenderer {
    spec: WebRendererSpec,
}

impl WebRenderer {
    pub fn new(_ctx: &RegisterCtx, _spec: WebRendererSpec) -> Result<Self, CreateWebRendererError> {
        return Err(CreateWebRendererError::WebRenderingNotAvailable);
    }

    pub fn render(
        &self,
        _ctx: &RenderCtx,
        _sources: &[&NodeTexture],
        _buffers: &[Arc<wgpu::Buffer>],
        _target: &mut NodeTexture,
    ) -> Result<(), RenderWebsiteError> {
        Ok(())
    }

    pub fn resolution(&self) -> Resolution {
        self.spec.resolution
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CreateWebRendererError {
    #[error("Web rendering feature is not available")]
    WebRenderingNotAvailable,
}

#[derive(Debug, thiserror::Error)]
pub enum RenderWebsiteError {}
