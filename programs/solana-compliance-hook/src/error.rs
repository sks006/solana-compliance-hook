use anchor_lang::prelude::*;

#[error_code]
pub enum ComplianceError {
    // 🏛️ Initialization & Structural Failures
    #[msg("Invalid TLV data structure in extra accounts.")]
    InvalidTlvData,
    #[msg("Metadata count mismatch.")]
    MetadataCountMismatch,
    #[msg("Invalid remaining accounts.")]
    InvalidRemainingAccounts,
    #[msg("Calculation failure during data sizing.")]
    CalculationFailure,
    #[msg("Provided account key does not match the extra metas PDA.")]
    InvalidExtraAccountMetaList,
    #[msg("Address is not in the list.")]
    AddressNotInList,
    #[msg("The target compliance vector has reached maximum capacity.")]
    ListFull,

    // 🛡️ Compliance Enforcement Failures
    #[msg("Source or destination not on allow list.")]
    NotAllowlisted,
    #[msg("Source is on the black list.")]
    SourceBlacklisted,
    #[msg("Destination is on the black list.")]
    DestinationBlacklisted,

    // 🪙 Token-2022 Extension Metrics
    #[msg("Invalid mint state for compliance checks.")]
    InvalidMintState,
    #[msg("Missing transfer fee extension.")]
    MissingFeeExtension,
    #[msg("Fee mismatch with extension.")]
    FeeMismatchedWithExtension,
    #[msg("Invalid fee basis points.")]
    InvalidFeeBps
}

