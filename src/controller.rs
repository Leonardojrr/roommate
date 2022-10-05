use crate::room::Context;
use futures_util::future::join;
use std::sync::Arc;
use tokio::sync::{mpsc::Sender, Mutex, MutexGuard};

pub struct Controller<T> {
    ctx: Arc<Mutex<Context<T>>>,
    signal_sender: Sender<bool>,
}

impl<T> Controller<T> {
    pub fn new(ctx: Arc<Mutex<Context<T>>>, signal_sender: Sender<bool>) -> Self {
        Self { ctx, signal_sender }
    }

    pub async fn ctx(&self) -> MutexGuard<Context<T>> {
        let guard_future = self.ctx.lock();
        let signal_future = self.signal_sender.send(true);

        let (guard, _) = join(guard_future, signal_future).await;

        return guard;
    }
}
