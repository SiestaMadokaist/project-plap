use std::rc::Rc;

#[derive(Default)]
pub struct WsClients {}

impl WsClients {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn rc() -> Rc<Self> {
        Rc::new(Self::new())
    }
}
