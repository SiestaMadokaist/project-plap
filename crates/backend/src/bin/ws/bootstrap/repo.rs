use std::rc::Rc;

#[derive(Default)]
#[allow(dead_code)]
pub struct WsRepos {}

impl WsRepos {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }

    #[allow(dead_code)]
    pub fn rc() -> Rc<Self> {
        Rc::new(Self::new())
    }
}
