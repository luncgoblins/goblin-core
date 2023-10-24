#[cfg(not(feature = "testnet"))]
pub mod proposal_constants {
    use std::ops::RangeInclusive;

    pub const MINIMUM_PROPOSAL_REQUIRED_THRESHOLD_PERCENTAGE: u64 = 33;
    pub const MAX_PROPOSAL_REQUIRED_THRESHOLD_PERCENTAGE: u64 = 100;
    pub const MAX_PROPOSAL_REQUIRED_QUORUM_PERCENTAGE: &str = "1";
    pub const MINIMUM_PROPOSAL_REQUIRED_QUORUM_PERCENTAGE: &str = "0.01";
    pub const VOTING_PERIOD_INTERVAL: RangeInclusive<u64> = 12342..=7 * 12342;
    // from 0.5 to 1 day in blocks (7 seconds per block)
    pub const DELAY_INTERVAL: RangeInclusive<u64> = 6171..=14400;
    pub const EXPIRATION_PERIOD_INTERVAL: RangeInclusive<u64> = 12342..=100_800;
    // from 10k to 60k $xASTRO
    pub const DEPOSIT_INTERVAL: RangeInclusive<u128> = 10000000000..=60000000000;
    /// Proposal validation attributes
    pub const MIN_TITLE_LENGTH: usize = 4;
    pub const MAX_TITLE_LENGTH: usize = 64;
    pub const MIN_DESC_LENGTH: usize = 4;
    pub const MAX_DESC_LENGTH: usize = 1024;
    pub const MIN_LINK_LENGTH: usize = 12;
    pub const MAX_LINK_LENGTH: usize = 128;
    /// Special characters that are allowed in proposal text
    pub const SAFE_TEXT_CHARS: &str = "!&?#()*+'-./\"";
}

#[cfg(feature = "testnet")]
mod proposal_constants {
    use std::ops::RangeInclusive;

    pub const MINIMUM_PROPOSAL_REQUIRED_THRESHOLD_PERCENTAGE: u64 = 33;
    pub const MAX_PROPOSAL_REQUIRED_THRESHOLD_PERCENTAGE: u64 = 100;
    pub const MAX_PROPOSAL_REQUIRED_QUORUM_PERCENTAGE: &str = "1";
    pub const MINIMUM_PROPOSAL_REQUIRED_QUORUM_PERCENTAGE: &str = "0.001";
    pub const VOTING_PERIOD_INTERVAL: RangeInclusive<u64> = 200..=7 * 12342;
    // from ~350 sec to 1 day in blocks (7 seconds per block)
    pub const DELAY_INTERVAL: RangeInclusive<u64> = 50..=14400;
    pub const EXPIRATION_PERIOD_INTERVAL: RangeInclusive<u64> = 400..=100_800;
    // from 0.001 to 60k $xASTRO
    pub const DEPOSIT_INTERVAL: RangeInclusive<u128> = 1000..=60000000000;
}