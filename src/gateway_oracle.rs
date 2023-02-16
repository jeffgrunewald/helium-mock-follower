use helium_crypto::PublicKeyBinary;
use helium_proto::{
    services::follower::GatewayInfo, BlockchainRegionParamV1, BlockchainRegionParamsV1,
    BlockchainRegionSpreadingV1, GatewayStakingMode, Region, RegionSpreading, TaggedSpreading,
};
use std::{collections::HashMap, fs::File, str::FromStr, sync::Arc};
use tokio::sync::RwLock;

pub struct GatewayOracle(pub Arc<RwLock<HashMap<Vec<u8>, GatewayInfo>>>);

#[derive(Debug, thiserror::Error)]
pub enum GatewayError {
    #[error("Failed to load gateways from file")]
    LoadError(#[from] std::io::Error),
    #[error("Failed to deserialize gateway from csv")]
    Deserialize(#[from] csv::Error),
    #[error("Failed to map text to proto field")]
    Proto(#[from] prost::DecodeError),
}

impl GatewayOracle {
    pub async fn new(csv_path: Option<&String>) -> Result<Self, GatewayError> {
        let mut gateway_map = Self(Arc::new(RwLock::new(HashMap::new())));
        if let Some(csv_path) = csv_path {
            let file = File::open(csv_path)?;
            let mut reader = csv::ReaderBuilder::new()
                .has_headers(true)
                .from_reader(file);
            for record in reader.deserialize() {
                let csv_gw: CsvRecord = record?;
                let proto_gw = GatewayInfo {
                    address: csv_gw.0.into(),
                    owner: csv_gw.1.into(),
                    location: csv_gw.2,
                    staking_mode: {
                        match csv_gw.3.to_lowercase().as_ref() {
                            "light" => GatewayStakingMode::Light.into(),
                            "dataonly" => GatewayStakingMode::Dataonly.into(),
                            // everything else is a full hotspot, why not?
                            _ => GatewayStakingMode::Full.into(),
                        }
                    },
                    gain: csv_gw.4,
                    region: Region::from_str(csv_gw.5.as_str())?.into(),
                    region_params: region_params(),
                };
                gateway_map.insert(proto_gw.address.clone(), proto_gw).await
            }
        }
        tracing::info!(
            "Loaded {} testing gateway info records",
            gateway_map.len().await
        );
        Ok(gateway_map)
    }

    pub async fn get(&self, pubkey: &Vec<u8>) -> Option<GatewayInfo> {
        self.0.read().await.get(pubkey).cloned()
    }

    pub async fn insert(&mut self, pubkey: Vec<u8>, info: GatewayInfo) {
        self.0.write().await.insert(pubkey, info);
    }

    async fn len(&self) -> usize {
        self.0.read().await.len()
    }

    pub async fn values(&self) -> impl Iterator<Item = GatewayInfo> {
        self.0
            .read()
            .await
            .values()
            .cloned()
            .collect::<Vec<GatewayInfo>>()
            .into_iter()
    }
}

impl std::clone::Clone for GatewayOracle {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

type CsvRecord = (
    PublicKeyBinary,
    PublicKeyBinary,
    String,
    String,
    i32,
    String,
);

fn region_params() -> Option<BlockchainRegionParamsV1> {
    let tagged_spreading = vec![
        TaggedSpreading {
            region_spreading: RegionSpreading::Sf10.into(),
            max_packet_size: 25,
        },
        TaggedSpreading {
            region_spreading: RegionSpreading::Sf9.into(),
            max_packet_size: 67,
        },
        TaggedSpreading {
            region_spreading: RegionSpreading::Sf8.into(),
            max_packet_size: 139,
        },
        TaggedSpreading {
            region_spreading: RegionSpreading::Sf7.into(),
            max_packet_size: 256,
        },
    ];
    Some(BlockchainRegionParamsV1 {
        region_params: vec![
            BlockchainRegionParamV1 {
                channel_frequency: 903900000,
                bandwidth: 125000,
                max_eirp: 360,
                spreading: Some(BlockchainRegionSpreadingV1 {
                    tagged_spreading: tagged_spreading.clone(),
                }),
            },
            BlockchainRegionParamV1 {
                channel_frequency: 904100000,
                bandwidth: 125000,
                max_eirp: 360,
                spreading: Some(BlockchainRegionSpreadingV1 {
                    tagged_spreading: tagged_spreading.clone(),
                }),
            },
            BlockchainRegionParamV1 {
                channel_frequency: 904300000,
                bandwidth: 125000,
                max_eirp: 360,
                spreading: Some(BlockchainRegionSpreadingV1 {
                    tagged_spreading: tagged_spreading.clone(),
                }),
            },
            BlockchainRegionParamV1 {
                channel_frequency: 904500000,
                bandwidth: 125000,
                max_eirp: 360,
                spreading: Some(BlockchainRegionSpreadingV1 {
                    tagged_spreading: tagged_spreading.clone(),
                }),
            },
            BlockchainRegionParamV1 {
                channel_frequency: 904700000,
                bandwidth: 125000,
                max_eirp: 360,
                spreading: Some(BlockchainRegionSpreadingV1 {
                    tagged_spreading: tagged_spreading.clone(),
                }),
            },
            BlockchainRegionParamV1 {
                channel_frequency: 904900000,
                bandwidth: 125000,
                max_eirp: 360,
                spreading: Some(BlockchainRegionSpreadingV1 {
                    tagged_spreading: tagged_spreading.clone(),
                }),
            },
            BlockchainRegionParamV1 {
                channel_frequency: 905100000,
                bandwidth: 125000,
                max_eirp: 360,
                spreading: Some(BlockchainRegionSpreadingV1 {
                    tagged_spreading: tagged_spreading.clone(),
                }),
            },
            BlockchainRegionParamV1 {
                channel_frequency: 905300000,
                bandwidth: 125000,
                max_eirp: 360,
                spreading: Some(BlockchainRegionSpreadingV1 { tagged_spreading }),
            },
        ],
    })
}
