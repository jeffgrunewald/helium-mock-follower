use crate::{gateway_oracle::GatewayOracle, height_oracle::BlockReq, GrpcResult, GrpcStreamResult};
use helium_proto::services::follower::{
    self, follower_error::Type, follower_gateway_resp_v1::Result as GatewayResult, FollowerError,
    FollowerGatewayReqV1, FollowerGatewayRespV1, FollowerGatewayStreamReqV1,
    FollowerGatewayStreamRespV1, FollowerSubnetworkLastRewardHeightReqV1,
    FollowerSubnetworkLastRewardHeightRespV1, FollowerTxnStreamReqV1, FollowerTxnStreamRespV1,
    GatewayNotFound,
};
use rand::Rng;
use tonic::{Request, Response, Status};

pub struct FollowerService {
    gateway_oracle: GatewayOracle,
    height_oracle: BlockReq,
    shutdown: triggered::Listener,
}

impl FollowerService {
    pub fn new(
        height_oracle: BlockReq,
        gateway_oracle: GatewayOracle,
        shutdown: triggered::Listener,
    ) -> Self {
        Self {
            gateway_oracle,
            height_oracle,
            shutdown,
        }
    }
}

#[tonic::async_trait]
impl follower::follower_server::Follower for FollowerService {
    type txn_streamStream = GrpcStreamResult<FollowerTxnStreamRespV1>;
    async fn txn_stream(
        &self,
        _request: Request<FollowerTxnStreamReqV1>,
    ) -> GrpcResult<Self::txn_streamStream> {
        unimplemented!();
    }

    async fn find_gateway(
        &self,
        request: Request<FollowerGatewayReqV1>,
    ) -> GrpcResult<FollowerGatewayRespV1> {
        let address = request.into_inner().address;

        let height = self
            .height_oracle
            .req()
            .await
            .map_err(|_| Status::internal("failed to retrieve height"))?
            .ok_or_else(|| Status::internal("failed to retrieve height"))?;

        let response = match self.gateway_oracle.get(&address).await {
            Some(gateway_info) => Some(GatewayResult::Info(gateway_info)),
            None => Some(GatewayResult::Error(FollowerError {
                r#type: Some(Type::NotFound(GatewayNotFound { address })),
            })),
        };

        Ok(Response::new(FollowerGatewayRespV1 {
            height,
            result: response,
        }))
    }

    type active_gatewaysStream = GrpcStreamResult<FollowerGatewayStreamRespV1>;
    async fn active_gateways(
        &self,
        request: Request<FollowerGatewayStreamReqV1>,
    ) -> GrpcResult<Self::active_gatewaysStream> {
        let batch_size = request.into_inner().batch_size;

        let (tx, rx) = tokio::sync::mpsc::channel(20);
        let height = self
            .height_oracle
            .req()
            .await
            .map_err(|_| Status::internal("failed to retrieve height"))?
            .ok_or_else(|| Status::internal("failed to retrieve height"))?;
        let gateways = self.gateway_oracle.clone();
        let shutdown_listener = self.shutdown.clone();

        tokio::spawn(async move {
            let mut batch: Vec<FollowerGatewayRespV1> = Vec::with_capacity(batch_size as usize);

            for gw_info in gateways.values().await {
                if shutdown_listener.is_triggered() {
                    return Ok(());
                }

                if batch.len() == batch_size as usize {
                    if (tx.send(Ok(FollowerGatewayStreamRespV1 {
                        gateways: batch.clone(),
                    })))
                    .await
                    .is_err()
                    {
                        break;
                    }
                    batch.clear()
                };
                let info = FollowerGatewayRespV1 {
                    height,
                    result: Some(GatewayResult::Info(gw_info.clone())),
                };
                tracing::debug!("Pushing gateway into batch {:?}", gw_info.address);
                batch.push(info)
            }

            if !batch.is_empty() {
                tx.send(Ok(FollowerGatewayStreamRespV1 { gateways: batch }))
                    .await
            } else {
                Ok(())
            }
        });

        Ok(Response::new(GrpcStreamResult::new(rx)))
    }

    async fn subnetwork_last_reward_height(
        &self,
        _request: Request<FollowerSubnetworkLastRewardHeightReqV1>,
    ) -> GrpcResult<FollowerSubnetworkLastRewardHeightRespV1> {
        let height = self
            .height_oracle
            .req()
            .await
            .map_err(|_| Status::internal("failed to retrieve height"))?
            .ok_or_else(|| Status::internal("failed to retrieve height"))?;
        let mut rng = rand::thread_rng();
        let reward_diff = rng.gen_range(50..1000);
        let reward_height = height - reward_diff;
        Ok(Response::new(FollowerSubnetworkLastRewardHeightRespV1 {
            height,
            reward_height,
        }))
    }
}
