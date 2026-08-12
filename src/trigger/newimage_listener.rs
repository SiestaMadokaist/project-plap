use std::{cell::Cell, path::Path, rc::Rc};

use notify::{Event, RecursiveMode, Watcher};
use tokio::sync::mpsc::{self, UnboundedReceiver};

use crate::{
    application::usecases::agent::{
        save_image::SaveImage,
        traits::{AgentClients, AgentRepos},
    },
    pkg::types::time::Timestamp,
};

pub struct NewImageListener<C: AgentClients, R: AgentRepos> {
    clients: Rc<C>,
    repos: Rc<R>,
    dir: String,
    last_active: Rc<Cell<Timestamp>>,
}

impl<C: AgentClients, R: AgentRepos> NewImageListener<C, R> {
    pub fn new(
        clients: Rc<C>,
        repos: Rc<R>,
        dir: String,
        last_active: Rc<Cell<Timestamp>>,
    ) -> Self {
        Self {
            clients,
            repos,
            dir,
            last_active,
        }
    }

    // filter if the change is something we care about.
    fn changed(&self, event: &Event) -> bool {
        event.kind.is_create()
    }

    /**
     */
    async fn handle_change(&self, event: &Event) -> anyhow::Result<()> {
        let extensions = vec![".png", ".jpg", ".jpeg"];
        let images = event
            .paths
            .iter()
            .filter(|x| extensions.iter().any(|e| x.ends_with(e)));
        let mut image_count = 0;
        for image_path in images {
            image_count += 1;
            let clients = self.clients.clone();
            let repos = self.repos.clone();
            let path = image_path.clone();
            let now = Timestamp::now();
            let saveimage = SaveImage::new(clients, repos, path, now);
            saveimage.exec().await?;
        }
        if image_count == 0 {
            return Ok(());
        }
        self.record_ok();
        Ok(())
    }

    fn record_ok(&self) -> () {
        let now = Timestamp::now();
        self.last_active.set(now);
    }

    async fn on_event(&self, event: &Event) -> anyhow::Result<()> {
        if !self.changed(event) {
            return Ok(());
        }
        self.handle_change(event).await
    }

    async fn init_watch(&self) -> anyhow::Result<UnboundedReceiver<notify::Result<Event>>> {
        // `notify::recommended_watcher` picks the OS-native backend — inotify on Linux,
        // no polling. The watcher must stay alive for as long as we want events, so it
        // lives in this stack frame for the duration of the loop below.
        let (tx, rx) = mpsc::unbounded_channel::<notify::Result<Event>>();

        let mut watcher = notify::recommended_watcher(move |res| {
            let _ = tx.send(res);
        })?;
        watcher.watch(Path::new(&self.dir), RecursiveMode::Recursive)?;
        Ok(rx)
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        let mut rx = self.init_watch().await?;
        while let Some(res) = rx.recv().await {
            let event = res?;
            self.on_event(&event).await?;
        }
        Ok(())
    }
}
