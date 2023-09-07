use log::error;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use crate::model::worker::message::WorkerMessage;

pub struct ShutdownHandler<'a> {
    pub tx: &'a mpsc::Sender<WorkerMessage>,
    thread_sigint: Option<JoinHandle<()>>,
    thread_timer: Option<JoinHandle<()>>,
}

impl<'a> ShutdownHandler<'a> {
    pub fn new(tx: &mpsc::Sender<WorkerMessage>) -> ShutdownHandler {
        ShutdownHandler {
            tx,
            thread_sigint: None,
            thread_timer: None,
        }
    }


    pub fn start(&mut self) {
        let tx = self.tx.clone();
        let thread = tokio::spawn(async move {
            match tokio::signal::ctrl_c().await {
                Ok(()) => {
                    tx.send(WorkerMessage::exit_sigint()).await.unwrap();
                }
                Err(_) => {
                    error!("Unable to create SIGINT handler.");
                }
            }
        });
        self.thread_sigint = Some(thread);
    }

    pub fn start_shutdown_timer(&mut self, max_runtime_secs: u64) {
        let tx = self.tx.clone();
        let thread = tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_secs(max_runtime_secs)).await;
            let res = tx.send(WorkerMessage::exit_time()).await;
            if res.is_err() {
                error!("Unable to send shutdown message.");
            }
        });
        self.thread_timer = Some(thread);
    }

    pub fn abort(&mut self) {
        if let Some(thread) = &self.thread_sigint {
            thread.abort();
        }
        if let Some(thread) = &self.thread_timer {
            thread.abort();
        }
    }
}

