use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use crate::model::error::RsrqError;
use crate::model::worker::message::WorkerMessage;

pub type RsrqResult<T> = Result<T, RsrqError>;
pub type OptUsizeFuture = JoinHandle<Result<Option<usize>, RsrqError>>;

pub type WorkerMsgSend = mpsc::Sender<WorkerMessage>;
pub type WorkerMsgRec = mpsc::Receiver<WorkerMessage>;

pub type JobFuture = JoinHandle<Result<(), RsrqError>>;

