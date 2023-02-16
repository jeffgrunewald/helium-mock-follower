use anyhow::{Error, Result};
use clap::Parser;
use futures_util::TryFutureExt;
use helium_mock_follower::{
    follower_service::FollowerService,
    gateway_oracle::GatewayOracle,
    height_oracle::{self, HeightOracle},
    settings::Settings,
};
use helium_proto::services::follower::follower_server::FollowerServer;
use std::{path::PathBuf, time::Duration};
use tokio::signal;
use tonic::transport;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Debug, clap::Parser)]
#[clap(version = env!("CARGO_PKG_VERSION"))]
#[clap(about = "Helium Blockchain Node Mock GRPC Server")]
pub struct Cli {
    #[clap(short = 'c')]
    config: Option<PathBuf>,

    #[clap(subcommand)]
    cmd: Cmd,
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        let settings = Settings::new(self.config)?;
        self.cmd.run(settings).await
    }
}

#[derive(Debug, clap::Subcommand)]
pub enum Cmd {
    Server(Daemon),
}

impl Cmd {
    pub async fn run(&self, settings: Settings) -> Result<()> {
        match self {
            Self::Server(cmd) => cmd.run(&settings).await,
        }
    }
}

#[derive(Debug, clap::Args)]
pub struct Daemon;

impl Daemon {
    pub async fn run(&self, settings: &Settings) -> Result<()> {
        tracing_subscriber::registry()
            .with(tracing_subscriber::EnvFilter::new(&settings.log))
            .with(tracing_subscriber::fmt::layer())
            .init();

        let (shutdown_trigger, shutdown_listener) = triggered::trigger();
        tokio::spawn(async move {
            let _ = signal::ctrl_c().await;
            shutdown_trigger.trigger()
        });

        let listen_addr = settings.listen_addr()?;
        let (height_req, height_res) = height_oracle::block_channel(20);

        let gateway_oracle = GatewayOracle::new(settings.gateways.as_ref()).await?;
        let mut height_oracle = HeightOracle::new(settings.height);
        let follower_server =
            FollowerService::new(height_req, gateway_oracle, shutdown_listener.clone());
        let grpc_server = transport::Server::builder()
            .http2_keepalive_interval(Some(Duration::from_secs(250)))
            .http2_keepalive_timeout(Some(Duration::from_secs(60)))
            .add_service(FollowerServer::new(follower_server))
            .serve_with_shutdown(listen_addr, shutdown_listener.clone())
            .map_err(Error::from);

        tokio::try_join!(
            height_oracle.run(height_res, &shutdown_listener),
            grpc_server,
        )
        .map(|_| ())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    cli.run().await
}
