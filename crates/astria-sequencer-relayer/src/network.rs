use astria_gossipnet::network::{
    Network,
    NetworkBuilder,
    Sha256Topic,
};
use eyre::Result;
use futures::StreamExt;
use tokio::{
    select,
    sync::mpsc::UnboundedReceiver,
};
use tracing::{
    debug,
    warn,
};

use crate::types::SequencerBlockData;

const BLOCKS_TOPIC: &str = "blocks";

pub struct GossipNetwork {
    network: Network,
    block_rx: UnboundedReceiver<SequencerBlockData>,
}

impl GossipNetwork {
    pub(crate) fn new(
        p2p_port: u16,
        block_rx: UnboundedReceiver<SequencerBlockData>,
    ) -> Result<Self> {
        let network = NetworkBuilder::new().port(p2p_port).build()?;
        Ok(Self {
            network,
            block_rx,
        })
    }

    /// Runs the gossip network.
    ///
    /// # Errors
    ///
    /// `GossipNetwork::run` never returns an error. The return type is
    /// only set to `eyre::Result` for convenient use in `SequencerRelayer`.
    pub(crate) async fn run(mut self) -> eyre::Result<()> {
        loop {
            select! {
                block = self.block_rx.recv() => {
                    if let Some(block) = block {
                        match self.publish(&block).await {
                            Ok(()) => debug!(block_hash = ?block.block_hash, "published block to network"),
                            Err(e) => warn!(?e, "failed to publish block to network"),
                        };
                    }
                },
                event = self.network.next() => {
                    if let Some(event) = event {
                        debug!(?event, "got event from network");
                    }
                },
            }
        }
        // Return Ok to make the types align (see the method's doc comment why this is necessary).
        // Allow unreachable code to quiet warnings
        #[allow(unreachable_code)]
        Ok(())
    }

    async fn publish(&mut self, block: &SequencerBlockData) -> Result<()> {
        self.network
            .publish(block.to_bytes()?, Sha256Topic::new(BLOCKS_TOPIC))
            .await?;
        Ok(())
    }
}
