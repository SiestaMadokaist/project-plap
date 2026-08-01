use crate::application::ports::repository::{
    translation::TranslationRepository, user::UserRepository,
};

pub trait RepositoryContainer {
    type User: UserRepository;
    type Translation: TranslationRepository;
    fn user(&self) -> &Self::User;
    fn translation(&self) -> &Self::Translation;
}

pub trait HasUser {
    type User: UserRepository;
    fn user(&self) -> &Self::User;
}

impl<RC: RepositoryContainer> HasUser for RC {
    type User = RC::User;
    fn user(&self) -> &Self::User {
        RepositoryContainer::user(self)
    }
}

pub trait HasTranslation {
    type Translation: TranslationRepository;
    fn translation(&self) -> &Self::Translation;
}

impl<RC: RepositoryContainer> HasTranslation for RC {
    type Translation = RC::Translation;
    fn translation(&self) -> &Self::Translation {
        RepositoryContainer::translation(self)
    }
}

pub trait AllRepos: HasTranslation + HasUser {}
