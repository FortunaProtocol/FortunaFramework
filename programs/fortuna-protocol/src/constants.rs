use anchor_lang::prelude::*;

/// Treasury wallet address
pub const TREASURY_WALLET: Pubkey = pubkey!("6Lbx8fvKRf1aE8Zi977sGHYqNeKvzxyjnGt5pee9FwoZ");

/// Seed for protocol state PDA
pub const PROTOCOL_SEED: &[u8] = b"protocol";

/// Seed for market PDA
pub const MARKET_SEED: &[u8] = b"market";

/// Seed for market vault PDA
pub const MARKET_VAULT_SEED: &[u8] = b"market_vault";

/// Seed for pool vault PDA (bonus pool from fees)
pub const POOL_VAULT_SEED: &[u8] = b"pool_vault";

/// Seed for bet PDA
pub const BET_SEED: &[u8] = b"bet";

/// Seed for oracle PDA
pub const ORACLE_SEED: &[u8] = b"oracle";

/// Seed for license PDA
pub const LICENSE_SEED: &[u8] = b"license";

/// Maximum allowed domains for a license
pub const MAX_LICENSE_DOMAINS: usize = 5;

/// Maximum allowed wallets for a license
pub const MAX_LICENSE_WALLETS: usize = 10;

/// Maximum domain name length
pub const MAX_DOMAIN_NAME_LEN: usize = 64;

/// Default protocol fee (0.5% = 50 basis points)
pub const DEFAULT_PROTOCOL_FEE_BPS: u16 = 50;

/// Default creator fee (0.5% = 50 basis points)
pub const DEFAULT_CREATOR_FEE_BPS: u16 = 50;

/// Default pool fee (5% = 500 basis points)
pub const DEFAULT_POOL_FEE_BPS: u16 = 500;

/// Maximum total fee (10% = 1000 basis points)
pub const MAX_TOTAL_FEE_BPS: u16 = 1000;

/// Basis points denominator
pub const BPS_DENOMINATOR: u16 = 10000;
