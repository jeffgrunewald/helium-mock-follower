pub mod follower_service;
pub mod height_oracle;
pub mod gateway_oracle;
pub mod settings;

pub use follower_service::FollowerService;
pub use settings::Settings;

use tokio_stream::wrappers::ReceiverStream;
use tonic::{Response, Status};

pub type GrpcResult<T> = Result<Response<T>, Status>;
pub type GrpcStreamResult<T> = ReceiverStream<Result<T, Status>>;
