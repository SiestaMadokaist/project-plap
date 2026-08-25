use crate::{
    displayable,
    pkg::auth::{
        authreq::{AuthReq, AuthSecret},
        errors::AuthError,
    },
};

pub struct Authorizer<D> {
    secret: AuthSecret,
    _p: std::marker::PhantomData<D>,
}

impl<D> Authorizer<D> {
    pub fn authenticate(&self, req: AuthReq) -> Result<String, AuthError> {
        Err(AuthError::TODO)
    }

    pub fn authorize(&self, jwt: &str) -> Result<D, AuthError> {
        Err(AuthError::TODO)
    }
}
