use std::{
    fmt::Debug,
    num::{NonZeroU64, ParseIntError},
};

use ics23::ibc_api::SDK_SPECS;
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::ErrorObject,
    Extensions,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tendermint_light_client_types::{ClientState, ConsensusState, Fraction};
use tracing::{error, instrument};
use unionlabs::{
    bech32::Bech32,
    ibc::core::{client::height::Height, commitment::merkle_root::MerkleRoot},
    option_unwrap,
    primitives::H256,
    result_unwrap, ErrorReporter,
};
use voyager_message::{
    core::{ChainId, ClientType},
    module::{ClientBootstrapModuleInfo, ClientBootstrapModuleServer},
    ClientBootstrapModule,
};
use voyager_vm::BoxDynError;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    Module::run().await
}

#[derive(Debug, Clone)]
pub struct Module {
    pub chain_id: ChainId,

    pub tm_client: cometbft_rpc::Client,
    pub chain_revision: u64,
    pub grpc_url: String,

    pub ibc_host_contract_address: H256,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub ws_url: String,
    pub grpc_url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ibc_host_contract_address: Option<Bech32<H256>>,
}

impl ClientBootstrapModule for Module {
    type Config = Config;

    async fn new(
        config: Self::Config,
        info: ClientBootstrapModuleInfo,
    ) -> Result<Self, BoxDynError> {
        let tm_client = cometbft_rpc::Client::new(config.ws_url).await?;

        let chain_id = tm_client.status().await?.node_info.network.to_string();

        info.ensure_chain_id(&chain_id)?;
        info.ensure_client_type(ClientType::TENDERMINT)?;

        let chain_revision = chain_id
            .split('-')
            .last()
            .ok_or_else(|| ChainIdParseError {
                found: chain_id.clone(),
                source: None,
            })?
            .parse()
            .map_err(|err| ChainIdParseError {
                found: chain_id.clone(),
                source: Some(err),
            })?;

        Ok(Self {
            tm_client,
            chain_id: ChainId::new(chain_id),
            chain_revision,
            grpc_url: config.grpc_url,
            ibc_host_contract_address: config
                .ibc_host_contract_address
                .map(|a| *a.data())
                .unwrap_or_default(),
        })
    }
}

#[derive(Debug, thiserror::Error)]
#[error("unable to parse chain id: expected format `<chain>-<revision-number>`, found `{found}`")]
pub struct ChainIdParseError {
    found: String,
    #[source]
    source: Option<ParseIntError>,
}

impl Module {
    #[must_use]
    pub fn make_height(&self, height: u64) -> Height {
        Height::new_with_revision(self.chain_revision, height)
    }
}

#[async_trait]
impl ClientBootstrapModuleServer for Module {
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn self_client_state(&self, _: &Extensions, height: Height) -> RpcResult<Value> {
        let params = protos::cosmos::staking::v1beta1::query_client::QueryClient::connect(
            self.grpc_url.clone(),
        )
        .await
        .unwrap()
        .params(protos::cosmos::staking::v1beta1::QueryParamsRequest {})
        .await
        .unwrap()
        .into_inner()
        .params
        .unwrap();

        let commit = self
            .tm_client
            .commit(Some(height.height().try_into().unwrap()))
            .await
            .unwrap();

        let height = commit.signed_header.header.height;

        let unbonding_period = std::time::Duration::new(
            params
                .unbonding_time
                .clone()
                .unwrap()
                .seconds
                .try_into()
                .unwrap(),
            params
                .unbonding_time
                .clone()
                .unwrap()
                .nanos
                .try_into()
                .unwrap(),
        );

        Ok(serde_json::to_value(ClientState {
            chain_id: self.chain_id.to_string(),
            // https://github.com/cometbft/cometbft/blob/da0e55604b075bac9e1d5866cb2e62eaae386dd9/light/verifier.go#L16
            trust_level: Fraction {
                numerator: 1,
                denominator: const { option_unwrap!(NonZeroU64::new(3)) },
            },
            // https://github.com/cosmos/relayer/blob/23d1e5c864b35d133cad6a0ef06970a2b1e1b03f/relayer/chains/cosmos/provider.go#L177
            trusting_period: unionlabs::google::protobuf::duration::Duration::new(
                (unbonding_period * 85 / 100).as_secs().try_into().unwrap(),
                (unbonding_period * 85 / 100)
                    .subsec_nanos()
                    .try_into()
                    .unwrap(),
            )
            .unwrap(),
            unbonding_period: unionlabs::google::protobuf::duration::Duration::new(
                unbonding_period.as_secs().try_into().unwrap(),
                unbonding_period.subsec_nanos().try_into().unwrap(),
            )
            .unwrap(),
            // https://github.com/cosmos/relayer/blob/23d1e5c864b35d133cad6a0ef06970a2b1e1b03f/relayer/chains/cosmos/provider.go#L177
            max_clock_drift: const {
                result_unwrap!(unionlabs::google::protobuf::duration::Duration::new(
                    60 * 10,
                    0
                ))
            },
            frozen_height: None,
            latest_height: Height::new_with_revision(
                self.chain_revision,
                height.inner().try_into().expect("is within bounds; qed;"),
            ),
            proof_specs: SDK_SPECS.into(),
            upgrade_path: vec!["upgrade".into(), "upgradedIBCState".into()],
            contract_address: self.ibc_host_contract_address,
        })
        .unwrap())
    }

    /// The consensus state on this chain at the specified `Height`.
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn self_consensus_state(&self, _: &Extensions, height: Height) -> RpcResult<Value> {
        let commit = self
            .tm_client
            .commit(Some(height.height().try_into().unwrap()))
            .await
            .map_err(|e| {
                ErrorObject::owned(
                    -1,
                    format!("error fetching commit: {}", ErrorReporter(e)),
                    None::<()>,
                )
            })?;

        Ok(serde_json::to_value(&ConsensusState {
            root: MerkleRoot {
                hash: commit.signed_header.header.app_hash.into_encoding(),
            },
            next_validators_hash: commit.signed_header.header.next_validators_hash,
            timestamp: commit.signed_header.header.time,
        })
        .unwrap())
    }
}