use anchor_lang::prelude::*;

/// Maximum number of outcomes for a market (e.g., Yes/No = 2, or multiple choice)
pub const MAX_OUTCOMES: usize = 10;
/// Maximum title length
pub const MAX_TITLE_LEN: usize = 128;
/// Maximum description length
pub const MAX_DESCRIPTION_LEN: usize = 512;
/// Maximum outcome string length
pub const MAX_OUTCOME_LEN: usize = 64;
/// Maximum oracle name length
pub const MAX_ORACLE_NAME_LEN: usize = 64;
/// Maximum data source URL length
pub const MAX_DATA_SOURCE_LEN: usize = 256;
/// Maximum allowed domains for a license
pub const MAX_ALLOWED_DOMAINS: usize = 5;
/// Maximum domain length
pub const MAX_DOMAIN_LEN: usize = 64;

/// License types for different feature tiers
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, InitSpace, Debug)]
#[repr(u8)]
pub enum LicenseType {
    /// Basic license - limited markets, basic features
    Basic = 0,
    /// Pro license - more markets, advanced features
    Pro = 1,
    /// Enterprise license - unlimited markets, all features
    Enterprise = 2,
    /// Custom license - specific feature set
    Custom = 3,
}

impl Default for LicenseType {
    fn default() -> Self {
        LicenseType::Basic
    }
}

impl LicenseType {
    /// Get license type from u8 value
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(LicenseType::Basic),
            1 => Some(LicenseType::Pro),
            2 => Some(LicenseType::Enterprise),
            3 => Some(LicenseType::Custom),
            _ => None,
        }
    }

    /// Get the name of the license type
    pub fn name(&self) -> &'static str {
        match self {
            LicenseType::Basic => "Basic",
            LicenseType::Pro => "Pro",
            LicenseType::Enterprise => "Enterprise",
            LicenseType::Custom => "Custom",
        }
    }

    /// Get max markets allowed for this license type
    pub fn max_markets(&self) -> u32 {
        match self {
            LicenseType::Basic => 5,
            LicenseType::Pro => 50,
            LicenseType::Enterprise => u32::MAX,
            LicenseType::Custom => u32::MAX,
        }
    }
}

/// License features flags
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, InitSpace, Debug, Default)]
pub struct LicenseFeatures {
    /// Can create markets
    pub can_create_markets: bool,
    /// Can use oracles
    pub can_use_oracles: bool,
    /// Can create private markets (wallet-locked)
    pub can_create_private_markets: bool,
    /// Can set custom fees (within limits)
    pub can_set_custom_fees: bool,
    /// Reserved feature flags for future use
    pub reserved: [bool; 4],
}

impl LicenseFeatures {
    /// Create features for a license type
    pub fn for_license_type(license_type: LicenseType) -> Self {
        match license_type {
            LicenseType::Basic => LicenseFeatures {
                can_create_markets: true,
                can_use_oracles: false,
                can_create_private_markets: false,
                can_set_custom_fees: false,
                reserved: [false; 4],
            },
            LicenseType::Pro => LicenseFeatures {
                can_create_markets: true,
                can_use_oracles: true,
                can_create_private_markets: true,
                can_set_custom_fees: false,
                reserved: [false; 4],
            },
            LicenseType::Enterprise => LicenseFeatures {
                can_create_markets: true,
                can_use_oracles: true,
                can_create_private_markets: true,
                can_set_custom_fees: true,
                reserved: [false; 4],
            },
            LicenseType::Custom => LicenseFeatures {
                can_create_markets: true,
                can_use_oracles: false,
                can_create_private_markets: false,
                can_set_custom_fees: false,
                reserved: [false; 4],
            },
        }
    }
}

/// License account - grants access to protocol features
#[account]
#[derive(InitSpace)]
pub struct License {
    /// Unique license key (hash of the actual key)
    pub license_key: [u8; 32],

    /// Wallet address that owns this license
    pub holder: Pubkey,

    /// License type (Basic, Pro, Enterprise, Custom)
    pub license_type: LicenseType,

    /// Enabled features for this license
    pub features: LicenseFeatures,

    /// Allowed domains (for domain locking) - empty means any domain
    #[max_len(5, 64)]
    pub allowed_domains: Vec<String>,

    /// Allowed wallets (for wallet locking) - empty means only holder
    #[max_len(10)]
    pub allowed_wallets: Vec<Pubkey>,

    /// Maximum markets this license can create
    pub max_markets: u32,

    /// Current markets created under this license
    pub markets_created: u32,

    /// Whether the license is currently active
    pub is_active: bool,

    /// Whether the license is transferable
    pub is_transferable: bool,

    /// Unix timestamp when license was issued
    pub issued_at: i64,

    /// Unix timestamp when license expires (0 = never)
    pub expires_at: i64,

    /// Last activity timestamp
    pub last_used_at: i64,

    /// Who issued this license
    pub issued_by: Pubkey,

    /// Bump seed for PDA
    pub bump: u8,

    /// Reserved for future use
    #[max_len(32)]
    pub reserved: Vec<u8>,
}

impl License {
    /// Check if license is valid (active and not expired)
    pub fn is_valid(&self, current_time: i64) -> bool {
        if !self.is_active {
            return false;
        }
        if self.expires_at > 0 && current_time > self.expires_at {
            return false;
        }
        true
    }

    /// Check if license can create more markets
    pub fn can_create_market(&self) -> bool {
        self.features.can_create_markets && self.markets_created < self.max_markets
    }

    /// Check if a wallet is authorized under this license
    pub fn is_wallet_authorized(&self, wallet: &Pubkey) -> bool {
        // Holder is always authorized
        if self.holder == *wallet {
            return true;
        }
        // Check allowed wallets list
        self.allowed_wallets.contains(wallet)
    }

    /// Check if domain is allowed (empty list means any domain)
    pub fn is_domain_allowed(&self, domain: &str) -> bool {
        if self.allowed_domains.is_empty() {
            return true;
        }
        self.allowed_domains.iter().any(|d| d == domain)
    }
}

/// Market categories for prediction markets
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, InitSpace, Debug)]
#[repr(u8)]
pub enum MarketCategory {
    /// Political events and outcomes
    Politics = 0,
    /// Sports events and matches
    Sports = 1,
    /// Financial markets and indices
    Finance = 2,
    /// Cryptocurrency prices and events
    Crypto = 3,
    /// Geopolitical events
    Geopolitics = 4,
    /// Company earnings reports
    Earnings = 5,
    /// Technology events and releases
    Tech = 6,
    /// Cultural events, entertainment, awards
    Culture = 7,
    /// World events and news
    World = 8,
    /// Economic indicators and data
    Economy = 9,
    /// Election outcomes
    Elections = 10,
    /// Social media mentions and trends
    Mentions = 11,
}

impl Default for MarketCategory {
    fn default() -> Self {
        MarketCategory::Crypto
    }
}

impl MarketCategory {
    /// Get category from u8 value
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(MarketCategory::Politics),
            1 => Some(MarketCategory::Sports),
            2 => Some(MarketCategory::Finance),
            3 => Some(MarketCategory::Crypto),
            4 => Some(MarketCategory::Geopolitics),
            5 => Some(MarketCategory::Earnings),
            6 => Some(MarketCategory::Tech),
            7 => Some(MarketCategory::Culture),
            8 => Some(MarketCategory::World),
            9 => Some(MarketCategory::Economy),
            10 => Some(MarketCategory::Elections),
            11 => Some(MarketCategory::Mentions),
            _ => None,
        }
    }

    /// Get the string name of the category
    pub fn name(&self) -> &'static str {
        match self {
            MarketCategory::Politics => "Politics",
            MarketCategory::Sports => "Sports",
            MarketCategory::Finance => "Finance",
            MarketCategory::Crypto => "Crypto",
            MarketCategory::Geopolitics => "Geopolitics",
            MarketCategory::Earnings => "Earnings",
            MarketCategory::Tech => "Tech",
            MarketCategory::Culture => "Culture",
            MarketCategory::World => "World",
            MarketCategory::Economy => "Economy",
            MarketCategory::Elections => "Elections",
            MarketCategory::Mentions => "Mentions",
        }
    }
}

/// Protocol-wide configuration state
#[account]
#[derive(InitSpace)]
pub struct ProtocolState {
    /// Authority that can update protocol settings
    pub authority: Pubkey,

    /// Treasury wallet to receive protocol fees
    pub treasury: Pubkey,

    /// Protocol fee in basis points (0.5% = 50 bps)
    pub protocol_fee_bps: u16,

    /// Creator fee in basis points (0.5% = 50 bps)
    pub creator_fee_bps: u16,

    /// Pool fee in basis points (5% = 500 bps)
    pub pool_fee_bps: u16,

    /// Total markets created
    pub total_markets: u64,

    /// Total volume processed (in smallest token unit)
    pub total_volume: u128,

    /// Number of registered oracles
    pub total_oracles: u32,

    /// Number of issued licenses
    pub total_licenses: u32,

    /// Whether a valid license is required to create markets
    pub require_license: bool,

    /// Bump seed for PDA
    pub bump: u8,

    /// Reserved for future use
    #[max_len(64)]
    pub reserved: Vec<u8>,
}

/// Market status enum
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, InitSpace)]
pub enum MarketStatus {
    /// Market is open for betting
    Open,
    /// Market is resolved with a winning outcome
    Resolved,
    /// Market is cancelled (all bets refundable)
    Cancelled,
}

impl Default for MarketStatus {
    fn default() -> Self {
        MarketStatus::Open
    }
}

/// Individual outcome tracking
#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
pub struct Outcome {
    /// Outcome label (e.g., "Yes", "No", "Team A")
    #[max_len(64)]
    pub label: String,

    /// Total amount bet on this outcome (after fees)
    pub total_amount: u64,

    /// Number of bettors on this outcome
    pub bettor_count: u32,
}

/// Oracle account for automated market resolution
#[account]
#[derive(InitSpace)]
pub struct Oracle {
    /// Oracle identifier (unique per category)
    pub oracle_id: u32,

    /// Oracle authority (can submit results)
    pub authority: Pubkey,

    /// Oracle name
    #[max_len(64)]
    pub name: String,

    /// Categories this oracle can resolve
    pub categories: [bool; 12],

    /// Data source URL or identifier
    #[max_len(256)]
    pub data_source: String,

    /// Whether the oracle is active
    pub is_active: bool,

    /// Total markets resolved by this oracle
    pub markets_resolved: u64,

    /// Timestamp when oracle was registered
    pub registered_at: i64,

    /// Last resolution timestamp
    pub last_resolution_at: i64,

    /// Bump seed for PDA
    pub bump: u8,

    /// Reserved for future use
    #[max_len(32)]
    pub reserved: Vec<u8>,
}

impl Oracle {
    /// Check if oracle can resolve a specific category
    pub fn can_resolve_category(&self, category: MarketCategory) -> bool {
        let index = category as usize;
        if index < 12 {
            self.categories[index]
        } else {
            false
        }
    }

    /// Enable a category for this oracle
    pub fn enable_category(&mut self, category: MarketCategory) {
        let index = category as usize;
        if index < 12 {
            self.categories[index] = true;
        }
    }

    /// Disable a category for this oracle
    pub fn disable_category(&mut self, category: MarketCategory) {
        let index = category as usize;
        if index < 12 {
            self.categories[index] = false;
        }
    }
}

/// Prediction market account
#[account]
#[derive(InitSpace)]
pub struct Market {
    /// Unique market identifier
    pub market_id: u64,

    /// Market creator
    pub creator: Pubkey,

    /// Creator's fee wallet
    pub creator_fee_wallet: Pubkey,

    /// Token mint used for betting (e.g., USDC)
    pub token_mint: Pubkey,

    /// Market category
    pub category: MarketCategory,

    /// Assigned oracle for automated resolution (optional)
    pub oracle: Pubkey,

    /// External event ID for oracle resolution (e.g., match ID, stock symbol)
    #[max_len(64)]
    pub oracle_event_id: String,

    /// Market title
    #[max_len(128)]
    pub title: String,

    /// Market description
    #[max_len(512)]
    pub description: String,

    /// Fixed bet amount (same for all participants)
    pub bet_amount: u64,

    /// Unix timestamp for when betting closes
    pub betting_deadline: i64,

    /// Unix timestamp for when market should be resolved
    pub resolution_deadline: i64,

    /// Current market status
    pub status: MarketStatus,

    /// Winning outcome index (only valid when status == Resolved)
    pub winning_outcome: u8,

    /// Total amount in the market vault (betting pool after fees)
    pub total_pool: u64,

    /// Total amount in the bonus pool (from pool fees)
    pub bonus_pool: u64,

    /// All possible outcomes
    #[max_len(10)]
    pub outcomes: Vec<Outcome>,

    /// Timestamp when market was created
    pub created_at: i64,

    /// Timestamp when market was resolved (0 if not resolved)
    pub resolved_at: i64,

    /// Whether market was resolved by oracle
    pub resolved_by_oracle: bool,

    /// Market vault bump seed
    pub vault_bump: u8,

    /// Pool vault bump seed
    pub pool_vault_bump: u8,

    /// Market account bump seed
    pub bump: u8,

    /// Reserved for future use
    #[max_len(32)]
    pub reserved: Vec<u8>,
}

/// Individual bet record
#[account]
#[derive(InitSpace)]
pub struct Bet {
    /// The market this bet belongs to
    pub market: Pubkey,

    /// The bettor's wallet
    pub bettor: Pubkey,

    /// Outcome index the bettor selected
    pub outcome_index: u8,

    /// Original bet amount (before fees)
    pub original_amount: u64,

    /// Amount added to pool (after fees)
    pub pool_amount: u64,

    /// Whether winnings have been claimed
    pub claimed: bool,

    /// Timestamp when bet was placed
    pub placed_at: i64,

    /// Bump seed for PDA
    pub bump: u8,

    /// Reserved for future use
    #[max_len(16)]
    pub reserved: Vec<u8>,
}

impl Market {
    /// Calculate the payout for a winning bet
    pub fn calculate_payout(&self, bet: &Bet) -> u64 {
        if self.status != MarketStatus::Resolved {
            return 0;
        }

        if bet.outcome_index != self.winning_outcome {
            return 0;
        }

        let winning_outcome = &self.outcomes[self.winning_outcome as usize];

        if winning_outcome.total_amount == 0 {
            return 0;
        }

        // Calculate share of the total pool + bonus pool
        let total_distributable = self.total_pool + self.bonus_pool;

        // Proportional share based on bet amount
        let share = (bet.pool_amount as u128)
            .checked_mul(total_distributable as u128)
            .unwrap()
            .checked_div(winning_outcome.total_amount as u128)
            .unwrap();

        share as u64
    }

    /// Get the total number of bettors across all outcomes
    pub fn total_bettors(&self) -> u32 {
        self.outcomes.iter().map(|o| o.bettor_count).sum()
    }

    /// Check if betting deadline has passed
    pub fn is_betting_closed(&self, current_time: i64) -> bool {
        current_time > self.betting_deadline
    }

    /// Check if resolution deadline has passed
    pub fn is_past_resolution_deadline(&self, current_time: i64) -> bool {
        current_time > self.resolution_deadline
    }

    /// Check if market has an assigned oracle
    pub fn has_oracle(&self) -> bool {
        self.oracle != Pubkey::default()
    }
}

impl ProtocolState {
    /// Calculate all fees for a given bet amount
    /// Returns (pool_fee, creator_fee, protocol_fee, net_amount)
    pub fn calculate_fees(&self, amount: u64) -> (u64, u64, u64, u64) {
        let pool_fee = (amount as u128)
            .checked_mul(self.pool_fee_bps as u128)
            .unwrap()
            .checked_div(10000)
            .unwrap() as u64;

        let creator_fee = (amount as u128)
            .checked_mul(self.creator_fee_bps as u128)
            .unwrap()
            .checked_div(10000)
            .unwrap() as u64;

        let protocol_fee = (amount as u128)
            .checked_mul(self.protocol_fee_bps as u128)
            .unwrap()
            .checked_div(10000)
            .unwrap() as u64;

        let total_fees = pool_fee + creator_fee + protocol_fee;
        let net_amount = amount.checked_sub(total_fees).unwrap();

        (pool_fee, creator_fee, protocol_fee, net_amount)
    }

    /// Total fee percentage in basis points
    pub fn total_fee_bps(&self) -> u16 {
        self.pool_fee_bps + self.creator_fee_bps + self.protocol_fee_bps
    }
}
