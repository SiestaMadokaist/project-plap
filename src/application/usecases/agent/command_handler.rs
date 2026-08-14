use std::rc::Rc;

use crate::{
    application::{
        ports::{
            clients::{self, storage::StorageClient},
            repository::{self, agent_command::AgentCommandRepository},
        },
        usecases::agent::inference::RunInference,
    },
    domain::{
        commands::{
            command::{
                Action::{Inference, Network},
                CommandDomain, Progression,
            },
            inference::InferenceArgs,
            network::{
                ModelDst,
                ModelSrc::{self, Civitai},
                NetworkAction::{Download, Upload},
                NetworkArgs,
            },
        },
        errors::DomainError,
        storage::StoragePath,
    },
    pkg::macros::{trait_clients, trait_repos},
};

trait_clients!(
    CommandHandlerClients,
    clients::container::HasDiffusion,
    clients::container::HasModelStorage
);
trait_repos!(CommandHandlerRepos, repository::container::HasAgentCommand);

pub struct CommandHandler<R: CommandHandlerRepos, C: CommandHandlerClients> {
    repo: Rc<R>,
    client: Rc<C>,
    params: CommandDomain,
}

impl<R: CommandHandlerRepos, C: CommandHandlerClients> CommandHandler<R, C> {
    pub fn new(repo: Rc<R>, client: Rc<C>, params: CommandDomain) -> Self {
        CommandHandler {
            repo,
            client,
            params,
        }
    }

    async fn record_progress(&self, progress: &Option<Progression>) -> anyhow::Result<()> {
        match progress {
            None => Ok(()),
            Some(p) => {
                let agent_repo = self.repo.agent_command();
                let id = &self.params.action_id;
                if let Err(x) = agent_repo.set_progress(id, p).await {
                    Err(x.into())
                } else {
                    Ok(())
                }
            }
        }
    }

    async fn handle_network(&self, arg: &NetworkArgs) -> anyhow::Result<()> {
        let result: anyhow::Result<()> = match &arg.src {
            ModelSrc::S3(s) => match &arg.dst {
                ModelDst::Local(d) => {
                    let storage = self.client.model_storage();
                    storage.download(&s.path, &d.path).await?;
                    if d.forward {
                        let fwd = d.path.to_str().unwrap_or_default();
                        if fwd == "" {
                            let msg = "local path must be defined first";
                            let err = DomainError::Prerequisite(msg.into());
                            Err(err.into())
                        } else {
                            storage.upload(&d.path, &StoragePath(fwd.into())).await?;
                            Ok(())
                        }
                    } else {
                        Ok(())
                    }
                }
                ModelDst::S3(_) => {
                    let msg = "transfer between s3 is not supported";
                    let err = DomainError::NotAllowed(msg.into());
                    Err(err.into())
                }
            },
            ModelSrc::Civitai(i) => match &arg.dst {
                ModelDst::S3(_) => {
                    let msg = "external to s3 must use local with forward = true";
                    let err = DomainError::NotAllowed(msg.into());
                    Err(err.into())
                }
                ModelDst::Local(_) => Err(DomainError::NotImplemented.into()),
            },
        };
        result
    }

    async fn handle_inference(&self, arg: &InferenceArgs) -> anyhow::Result<()> {
        let progress = self.params.progress.clone();
        let config = &arg.config;
        let client = self.client.clone();
        let mut inferer = RunInference::new(client, progress, config);
        let updated_progress = inferer.generate().await?;
        self.record_progress(&updated_progress).await?;
        Ok(())
    }

    pub async fn exec(&mut self) -> anyhow::Result<()> {
        let action = &self.params.action;
        match action {
            Inference(arg) => self.handle_inference(arg).await,
            Network(arg) => self.handle_network(arg).await,
            _ => Ok(()),
        }
    }
}
