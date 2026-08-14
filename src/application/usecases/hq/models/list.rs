use std::rc::Rc;

pub struct List<C> {
    clients: Rc<C>,
}
