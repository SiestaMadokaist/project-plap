use crate::application::ports::repository::user::UserRepository;

pub trait RepositoryContainer {
    fn user_repo(&self) -> impl UserRepository;
}
