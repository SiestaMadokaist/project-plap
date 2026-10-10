use std::sync::Arc;

#[derive(Default)]
#[allow(dead_code)]
pub struct WsRepos {}

impl WsRepos {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }

    #[allow(dead_code)]
    pub fn arc() -> Arc<Self> {
        Arc::new(Self::new())
    }
}
