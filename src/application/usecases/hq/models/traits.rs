use crate::{
    application::ports::{
        clients::container::HasModelStorage, repository::container::HasHotReload,
    },
    pkg::macros::trait_clients,
};

trait_clients!(HQModels, HasModelStorage, HasHotReload);
