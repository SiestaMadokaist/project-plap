use std::rc::Rc;

#[derive(Default)]
pub struct WsRepos {}

impl WsRepos {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn rc() -> Rc<Self> {
        Rc::new(Self::new())
    }
}
