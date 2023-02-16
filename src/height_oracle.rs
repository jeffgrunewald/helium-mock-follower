use anyhow::Result;
use tokio::{
    sync::{mpsc, oneshot},
    time,
};

#[derive(Debug)]
pub struct RespTx(oneshot::Sender<Option<u64>>);
pub struct RespRx(oneshot::Receiver<Option<u64>>);

#[derive(Clone, Debug)]
pub struct BlockReq(mpsc::Sender<RespTx>);
pub struct BlockRes(mpsc::Receiver<RespTx>);

pub fn block_channel(size: usize) -> (BlockReq, BlockRes) {
    let (tx, rx) = mpsc::channel(size);
    (BlockReq(tx), BlockRes(rx))
}

impl BlockReq {
    pub async fn req(&self) -> Result<Option<u64>, oneshot::error::RecvError> {
        let (tx, rx) = result_channel();
        let _ = self.0.send(tx).await;
        rx.recv().await
    }
}

impl BlockRes {
    pub async fn recv(&mut self) -> Option<RespTx> {
        self.0.recv().await
    }
}

pub fn result_channel() -> (RespTx, RespRx) {
    let (tx, rx) = oneshot::channel();
    (RespTx(tx), RespRx(rx))
}

impl RespTx {
    pub fn send(self, msg: Option<u64>) {
        match self.0.send(msg) {
            Ok(()) => (),
            Err(err) => tracing::warn!("failed to return block height: {err:?}"),
        }
    }
}

impl RespRx {
    pub async fn recv(self) -> Result<Option<u64>, oneshot::error::RecvError> {
        self.0.await
    }
}

#[derive(Debug)]
pub struct HeightOracle {
    height: u64,
}

impl HeightOracle {
    pub fn new(starting_height: u64) -> Self {
        Self {
            height: starting_height,
        }
    }

    pub async fn run(
        &mut self,
        mut receiver: BlockRes,
        shutdown: &triggered::Listener,
    ) -> Result<()> {
        let mut timer = time::interval(std::time::Duration::from_secs(60));

        loop {
            if shutdown.is_triggered() {
                return Ok(());
            }

            tokio::select! {
                req = receiver.recv() => match req {
                    Some(resp_tx) => resp_tx.send(Some(self.height)),
                    None => {
                        tracing::warn!("height request channel closed");
                        return Ok(())
                    }
                },
                _ = timer.tick() => {
                    self.height += 1;
                },
                _ = shutdown.clone() => return Ok(()),
            }
        }
    }
}
