use cosmwasm_std::StdError;
use ethereum_light_client::client::EthereumLightClient;
use scroll_codec::batch_header::BatchHeaderV3DecodeError;
use union_ibc_light_client::IbcClientError;
use unionlabs::{hash::H256, ibc::core::client::height::Height, ics24::PathParseError};

use crate::client::ScrollLightClient;

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum Error {
    // REVIEW: Move this variant to IbcClientError?
    #[error("consensus state not found at height {0}")]
    ConsensusStateNotFound(Height),

    #[error("IBC path is empty")]
    EmptyIbcPath,

    #[error("proof is empty")]
    EmptyProof,

    #[error("expected value ({expected}) and stored value ({stored}) don't match")]
    StoredValueMismatch { expected: H256, stored: H256 },

    #[error("unable to parse ics24 path")]
    PathParse(#[from] PathParseError),

    #[error("failed to verify scroll header")]
    Verify(#[from] scroll_verifier::Error),

    #[error("the operation has not been implemented yet")]
    Unimplemented,

    #[error("error while calling custom query: {0}")]
    CustomQuery(#[from] unionlabs::cosmwasm::wasm::union::custom_query::Error),

    #[error("error decoding batch header")]
    BatchHeaderDecode(#[from] BatchHeaderV3DecodeError),

    #[error("substitute client is frozen")]
    SubstituteClientFrozen,

    #[error("forbidden fields have been changed during state migration")]
    MigrateFieldsChanged,

    #[error(transparent)]
    StdError(#[from] StdError),

    #[error("commitment key must be 32 bytes but we got: {0:?}")]
    InvalidCommitmentKeyLength(Vec<u8>),

    #[error("commitment value must be 32 bytes but we got: {0:?}")]
    InvalidCommitmentValueLength(Vec<u8>),

    #[error(transparent)]
    Evm(#[from] ethereum_light_client::errors::Error),

    #[error(transparent)]
    EvmIbcClient(#[from] IbcClientError<EthereumLightClient>),
}

impl From<Error> for IbcClientError<ScrollLightClient> {
    fn from(value: Error) -> Self {
        IbcClientError::ClientSpecific(value)
    }
}

impl From<Error> for StdError {
    fn from(value: Error) -> Self {
        StdError::generic_err(value.to_string())
    }
}