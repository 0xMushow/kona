//! This module contains the [HintType] enum.

use crate::errors::{HintParsingError, OracleProviderError};
use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use alloy_primitives::{hex, Bytes, B256};
use core::{fmt::Display, str::FromStr};
use kona_preimage::{CommsClient, PreimageKey, PreimageKeyType};

/// A [Hint] is parsed in the format `<hint_type> <hint_data>`, where `<hint_type>` is a string that
/// represents the type of hint, and `<hint_data>` is the data associated with the hint (bytes
/// encoded as hex UTF-8).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Hint<HT> {
    /// The type of hint.
    pub hint_type: HT,
    /// The data associated with the hint.
    pub hint_data: Bytes,
}

impl<HT> Hint<HT> {
    /// Splits the [Hint] into its components.
    pub fn split(self) -> (HT, Bytes) {
        (self.hint_type, self.hint_data)
    }
}

impl<HT> FromStr for Hint<HT>
where
    HT: FromStr<Err = HintParsingError>,
{
    type Err = HintParsingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split(' ').collect::<Vec<_>>();

        if parts.len() != 2 {
            return Err(HintParsingError(alloc::format!("Invalid hint format: {}", s)));
        }

        let hint_type = parts.remove(0).parse::<HT>()?;
        let hint_data =
            hex::decode(parts.remove(0)).map_err(|e| HintParsingError(e.to_string()))?.into();

        Ok(Self { hint_type, hint_data })
    }
}

/// The [HintType] enum is used to specify the type of hint that was received.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HintType {
    /// A hint that specifies the block header of a layer 1 block.
    L1BlockHeader,
    /// A hint that specifies the transactions of a layer 1 block.
    L1Transactions,
    /// A hint that specifies the state node of a layer 1 block.
    L1Receipts,
    /// A hint that specifies a blob in the layer 1 beacon chain.
    L1Blob,
    /// A hint that specifies a precompile call on layer 1.
    L1Precompile,
    /// A hint that specifies the block header of a layer 2 block.
    L2BlockHeader,
    /// A hint that specifies the transactions of a layer 2 block.
    L2Transactions,
    /// A hint that specifies the code of a contract on layer 2.
    L2Code,
    /// A hint that specifies the preimage of the starting L2 output root on layer 2.
    StartingL2Output,
    /// A hint that specifies the state node in the L2 state trie.
    L2StateNode,
    /// A hint that specifies the proof on the path to an account in the L2 state trie.
    L2AccountProof,
    /// A hint that specifies the proof on the path to a storage slot in an account within in the
    /// L2 state trie.
    L2AccountStorageProof,
    /// A hint that specifies bulk storage of all the code, state and keys generated by an
    /// execution witness.
    L2PayloadWitness,
}

impl HintType {
    /// Encodes the hint type as a string.
    pub fn encode_with(&self, data: &[&[u8]]) -> String {
        let concatenated = hex::encode(data.iter().copied().flatten().copied().collect::<Vec<_>>());
        alloc::format!("{} {}", self, concatenated)
    }

    /// Retrieves a preimage through an oracle
    pub async fn get_preimage<T: CommsClient>(
        &self,
        oracle: &T,
        image: B256,
        preimage_key_type: PreimageKeyType,
    ) -> Result<Vec<u8>, OracleProviderError> {
        oracle
            .write(&self.encode_with(&[image.as_ref()]))
            .await
            .map_err(OracleProviderError::Preimage)?;
        oracle
            .get(PreimageKey::new(*image, preimage_key_type))
            .await
            .map_err(OracleProviderError::Preimage)
    }

    /// Retrieves a preimage through an oracle
    pub async fn get_exact_preimage<T: CommsClient>(
        &self,
        oracle: &T,
        image: B256,
        preimage_key_type: PreimageKeyType,
        buf: &mut [u8],
    ) -> Result<(), OracleProviderError> {
        oracle
            .write(&self.encode_with(&[image.as_ref()]))
            .await
            .map_err(OracleProviderError::Preimage)?;
        oracle
            .get_exact(PreimageKey::new(*image, preimage_key_type), buf)
            .await
            .map_err(OracleProviderError::Preimage)
    }
}

impl FromStr for HintType {
    type Err = HintParsingError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "l1-block-header" => Ok(Self::L1BlockHeader),
            "l1-transactions" => Ok(Self::L1Transactions),
            "l1-receipts" => Ok(Self::L1Receipts),
            "l1-blob" => Ok(Self::L1Blob),
            "l1-precompile" => Ok(Self::L1Precompile),
            "l2-block-header" => Ok(Self::L2BlockHeader),
            "l2-transactions" => Ok(Self::L2Transactions),
            "l2-code" => Ok(Self::L2Code),
            "starting-l2-output" => Ok(Self::StartingL2Output),
            "l2-state-node" => Ok(Self::L2StateNode),
            "l2-account-proof" => Ok(Self::L2AccountProof),
            "l2-account-storage-proof" => Ok(Self::L2AccountStorageProof),
            "l2-payload-witness" => Ok(Self::L2PayloadWitness),
            _ => Err(HintParsingError(value.to_string())),
        }
    }
}

impl From<HintType> for &str {
    fn from(value: HintType) -> Self {
        match value {
            HintType::L1BlockHeader => "l1-block-header",
            HintType::L1Transactions => "l1-transactions",
            HintType::L1Receipts => "l1-receipts",
            HintType::L1Blob => "l1-blob",
            HintType::L1Precompile => "l1-precompile",
            HintType::L2BlockHeader => "l2-block-header",
            HintType::L2Transactions => "l2-transactions",
            HintType::L2Code => "l2-code",
            HintType::StartingL2Output => "starting-l2-output",
            HintType::L2StateNode => "l2-state-node",
            HintType::L2AccountProof => "l2-account-proof",
            HintType::L2AccountStorageProof => "l2-account-storage-proof",
            HintType::L2PayloadWitness => "l2-payload-witness",
        }
    }
}

impl Display for HintType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let s: &str = (*self).into();
        write!(f, "{}", s)
    }
}
