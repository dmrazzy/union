use ethereum_light_client_types::{AccountProof, StorageProof};
use unionlabs::ibc::core::client::height::Height;

use crate::L2Header;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Header {
    pub l1_height: Height,
    pub l1_account_proof: AccountProof,
    pub l2_ibc_account_proof: AccountProof,
    pub l1_next_node_num_slot_proof: StorageProof,
    pub l1_nodes_slot_proof: StorageProof,
    pub l2_header: L2Header,
}

#[cfg(feature = "proto")]
pub mod proto {
    use ethereum_light_client_types::{account_proof, storage_proof};
    use unionlabs::{errors::MissingField, impl_proto_via_try_from_into, required};

    use crate::{l2_header, Header};

    impl_proto_via_try_from_into!(Header => protos::union::ibc::lightclients::arbitrum::v1::Header);

    impl TryFrom<protos::union::ibc::lightclients::arbitrum::v1::Header> for Header {
        type Error = Error;

        fn try_from(
            value: protos::union::ibc::lightclients::arbitrum::v1::Header,
        ) -> Result<Self, Self::Error> {
            Ok(Self {
                l1_height: required!(value.l1_height)?.into(),
                l1_account_proof: required!(value.l1_account_proof)?
                    .try_into()
                    .map_err(Error::L1AccountProof)?,
                l2_ibc_account_proof: required!(value.l2_ibc_account_proof)?
                    .try_into()
                    .map_err(Error::L2IbcAccountProof)?,
                l1_next_node_num_slot_proof: required!(value.l1_next_node_num_slot_proof)?
                    .try_into()
                    .map_err(Error::L1NextNodeNumSlotProof)?,
                l1_nodes_slot_proof: required!(value.l1_nodes_slot_proof)?
                    .try_into()
                    .map_err(Error::L1NodesSlotProof)?,
                l2_header: required!(value.l2_header)?
                    .try_into()
                    .map_err(Error::L2Header)?,
            })
        }
    }

    #[derive(Debug, Clone, PartialEq, thiserror::Error)]
    pub enum Error {
        #[error(transparent)]
        MissingField(#[from] MissingField),
        #[error("invalid l1_account_proof")]
        L1AccountProof(#[source] account_proof::proto::Error),
        #[error("invalid l2_ibc_account_proof")]
        L2IbcAccountProof(#[source] account_proof::proto::Error),
        #[error("invalid l1_next_node_num_slot_proof")]
        L1NextNodeNumSlotProof(#[source] storage_proof::proto::Error),
        #[error("invalid l1_nodes_slot_proof")]
        L1NodesSlotProof(#[source] storage_proof::proto::Error),
        #[error("invalid l2_header")]
        L2Header(#[source] l2_header::proto::Error),
    }

    impl From<Header> for protos::union::ibc::lightclients::arbitrum::v1::Header {
        fn from(value: Header) -> Self {
            Self {
                l1_height: Some(value.l1_height.into()),
                l1_account_proof: Some(value.l1_account_proof.into()),
                l2_ibc_account_proof: Some(value.l2_ibc_account_proof.into()),
                l1_next_node_num_slot_proof: Some(value.l1_next_node_num_slot_proof.into()),
                l1_nodes_slot_proof: Some(value.l1_nodes_slot_proof.into()),
                l2_header: Some(value.l2_header.into()),
            }
        }
    }
}