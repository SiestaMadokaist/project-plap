use std::{
    cell::{Cell, OnceCell},
    path::{Path, PathBuf},
    rc::Rc,
};

use notify::{
    event::{AccessKind, AccessMode},
    Event, EventKind, INotifyWatcher, RecursiveMode, Watcher,
};
use tokio::sync::mpsc::{self, UnboundedReceiver};

use crate::{
    application::usecases::agent::{
        save_output::SaveOutput,
        traits::{AgentClients, AgentRepos},
    },
    pkg::types::time::Timestamp,
};

pub struct NewOutputListener<C: AgentClients, R: AgentRepos> {
    clients: Rc<C>,
    repos: Rc<R>,
    dir: String,
    last_active: Rc<Cell<Timestamp>>,
    watcher: OnceCell<INotifyWatcher>,
}

impl<C: AgentClients, R: AgentRepos> NewOutputListener<C, R> {
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
            watcher: OnceCell::new(),
        }
    }

    // filter if the change is something we care about.
    fn changed(&self, event: &Event) -> bool {
        matches!(
            event.kind,
            EventKind::Access(AccessKind::Close(AccessMode::Write))
        )
    }

    fn watchdir(&self) -> PathBuf {
        PathBuf::from(self.dir.clone())
    }

    /**
     */
    async fn handle_change(&self, event: &Event) -> anyhow::Result<()> {
        let extensions = vec![".png", ".jpg", ".jpeg", ".mp4", ".webp"];
        let files = event
            .paths
            .iter()
            .filter(|x| extensions.iter().any(|e| x.to_string_lossy().ends_with(e)));
        let mut output_count = 0;
        for output_path in files {
            tracing::info!("detected change for: {}", output_path.display());
            output_count += 1;
            let clients = self.clients.clone();
            let repos = self.repos.clone();
            let path = output_path.clone();
            let now = Timestamp::now();
            let saveoutput = SaveOutput::new(clients, repos, self.watchdir(), path, now);
            saveoutput.exec().await?;
        }
        if output_count == 0 {
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
        tracing::debug!("listening to changes in: {}", &self.dir);
        watcher.watch(Path::new(&self.dir), RecursiveMode::Recursive)?;
        self.watcher.set(watcher).expect("should succeed");
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
