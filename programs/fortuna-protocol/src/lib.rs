use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

declare_id!("FortunaProt1111111111111111111111111111111");

pub mod state;
pub mod errors;
pub mod instructions;
pub mod constants;

use state::*;
use errors::*;
use constants::*;

#[program]
pub mod fortuna_protocol {
    use super::*;

    /// Initialize the protocol with treasury and fee settings
    pub fn initialize_protocol(
        ctx: Context<InitializeProtocol>,
        protocol_fee_bps: u16,
        creator_fee_bps: u16,
        pool_fee_bps: u16,
    ) -> Result<()> {
        instructions::initialize_protocol(ctx, protocol_fee_bps, creator_fee_bps, pool_fee_bps)
    }

    /// Register a new oracle for automated market resolution
    pub fn register_oracle(
        ctx: Context<RegisterOracle>,
        oracle_id: u32,
        name: String,
        categories: [bool; 12],
        data_source: String,
    ) -> Result<()> {
        instructions::register_oracle(ctx, oracle_id, name, categories, data_source)
    }

    /// Update oracle settings
    pub fn update_oracle(
        ctx: Context<UpdateOracle>,
        name: Option<String>,
        categories: Option<[bool; 12]>,
        data_source: Option<String>,
        is_active: Option<bool>,
    ) -> Result<()> {
        instructions::update_oracle(ctx, name, categories, data_source, is_active)
    }

    /// Create a new prediction market with category
    pub fn create_market(
        ctx: Context<CreateMarket>,
        market_id: u64,
        category: u8,
        title: String,
        description: String,
        bet_amount: u64,
        resolution_deadline: i64,
        betting_deadline: i64,
        outcomes: Vec<String>,
        oracle_event_id: String,
    ) -> Result<()> {
        instructions::create_market(
            ctx,
            market_id,
            category,
            title,
            description,
            bet_amount,
            resolution_deadline,
            betting_deadline,
            outcomes,
            oracle_event_id,
        )
    }

    /// Assign an oracle to a market for automated resolution
    pub fn assign_oracle(
        ctx: Context<AssignOracle>,
    ) -> Result<()> {
        instructions::assign_oracle(ctx)
    }

    /// Place a bet on a specific outcome
    pub fn place_bet(
        ctx: Context<PlaceBet>,
        outcome_index: u8,
    ) -> Result<()> {
        instructions::place_bet(ctx, outcome_index)
    }

    /// Resolve the market with the winning outcome (creator only)
    pub fn resolve_market(
        ctx: Context<ResolveMarket>,
        winning_outcome: u8,
    ) -> Result<()> {
        instructions::resolve_market(ctx, winning_outcome)
    }

    /// Resolve the market via oracle (oracle authority only)
    pub fn oracle_resolve_market(
        ctx: Context<OracleResolveMarket>,
        winning_outcome: u8,
    ) -> Result<()> {
        instructions::oracle_resolve_market(ctx, winning_outcome)
    }

    /// Claim winnings after market resolution
    pub fn claim_winnings(ctx: Context<ClaimWinnings>) -> Result<()> {
        instructions::claim_winnings(ctx)
    }

    /// Cancel a market (only before any bets or by admin)
    pub fn cancel_market(ctx: Context<CancelMarket>) -> Result<()> {
        instructions::cancel_market(ctx)
    }

    /// Refund bet for cancelled market
    pub fn claim_refund(ctx: Context<ClaimRefund>) -> Result<()> {
        instructions::claim_refund(ctx)
    }

    /// Withdraw a bet before market resolution (user gets back their stake minus fees)
    pub fn withdraw_bet(ctx: Context<WithdrawBet>) -> Result<()> {
        instructions::withdraw_bet(ctx)
    }

    /// Update protocol settings (admin only)
    pub fn update_protocol(
        ctx: Context<UpdateProtocol>,
        new_treasury: Option<Pubkey>,
        new_protocol_fee_bps: Option<u16>,
        new_creator_fee_bps: Option<u16>,
        new_pool_fee_bps: Option<u16>,
    ) -> Result<()> {
        instructions::update_protocol(ctx, new_treasury, new_protocol_fee_bps, new_creator_fee_bps, new_pool_fee_bps)
    }

    /// Toggle whether license is required to create markets
    pub fn set_require_license(
        ctx: Context<UpdateProtocol>,
        require_license: bool,
    ) -> Result<()> {
        instructions::set_require_license(ctx, require_license)
    }

    // =========================================================================
    // License Management
    // =========================================================================

    /// Issue a new license to a wallet
    pub fn issue_license(
        ctx: Context<IssueLicense>,
        license_key: [u8; 32],
        license_type: u8,
        allowed_domains: Vec<String>,
        allowed_wallets: Vec<Pubkey>,
        max_markets: u32,
        is_transferable: bool,
        expires_at: i64,
    ) -> Result<()> {
        instructions::issue_license(
            ctx,
            license_key,
            license_type,
            allowed_domains,
            allowed_wallets,
            max_markets,
            is_transferable,
            expires_at,
        )
    }

    /// Revoke/deactivate a license
    pub fn revoke_license(ctx: Context<RevokeLicense>) -> Result<()> {
        instructions::revoke_license(ctx)
    }

    /// Activate a previously deactivated license
    pub fn activate_license(ctx: Context<RevokeLicense>) -> Result<()> {
        instructions::activate_license(ctx)
    }

    /// Transfer a license to a new holder
    pub fn transfer_license(ctx: Context<TransferLicense>) -> Result<()> {
        instructions::transfer_license(ctx)
    }

    /// Update license settings
    pub fn update_license(
        ctx: Context<UpdateLicense>,
        new_max_markets: Option<u32>,
        new_expires_at: Option<i64>,
        new_features: Option<LicenseFeatures>,
    ) -> Result<()> {
        instructions::update_license(ctx, new_max_markets, new_expires_at, new_features)
    }

    /// Add an authorized wallet to a license
    pub fn add_authorized_wallet(
        ctx: Context<ModifyLicenseWallets>,
        wallet: Pubkey,
    ) -> Result<()> {
        instructions::add_authorized_wallet(ctx, wallet)
    }

    /// Remove an authorized wallet from a license
    pub fn remove_authorized_wallet(
        ctx: Context<ModifyLicenseWallets>,
        wallet: Pubkey,
    ) -> Result<()> {
        instructions::remove_authorized_wallet(ctx, wallet)
    }

    /// Add an authorized domain to a license
    pub fn add_authorized_domain(
        ctx: Context<ModifyLicenseDomains>,
        domain: String,
    ) -> Result<()> {
        instructions::add_authorized_domain(ctx, domain)
    }

    /// Remove an authorized domain from a license
    pub fn remove_authorized_domain(
        ctx: Context<ModifyLicenseDomains>,
        domain: String,
    ) -> Result<()> {
        instructions::remove_authorized_domain(ctx, domain)
    }
}

// ============================================================================
// Account Contexts
// ============================================================================

#[derive(Accounts)]
pub struct InitializeProtocol<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + ProtocolState::INIT_SPACE,
        seeds = [PROTOCOL_SEED],
        bump
    )]
    pub protocol_state: Account<'info, ProtocolState>,

    #[account(mut)]
    pub authority: Signer<'info>,

    /// CHECK: Treasury wallet to receive protocol fees
    pub treasury: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(oracle_id: u32)]
pub struct RegisterOracle<'info> {
    #[account(
        mut,
        seeds = [PROTOCOL_SEED],
        bump = protocol_state.bump,
        constraint = protocol_state.authority == authority.key() @ FortunaError::Unauthorized
    )]
    pub protocol_state: Account<'info, ProtocolState>,

    #[account(
        init,
        payer = authority,
        space = 8 + Oracle::INIT_SPACE,
        seeds = [ORACLE_SEED, &oracle_id.to_le_bytes()],
        bump
    )]
    pub oracle: Account<'info, Oracle>,

    /// CHECK: Oracle authority that can submit results
    pub oracle_authority: UncheckedAccount<'info>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateOracle<'info> {
    #[account(
        seeds = [PROTOCOL_SEED],
        bump = protocol_state.bump,
        constraint = protocol_state.authority == authority.key() @ FortunaError::Unauthorized
    )]
    pub protocol_state: Account<'info, ProtocolState>,

    #[account(
        mut,
        seeds = [ORACLE_SEED, &oracle.oracle_id.to_le_bytes()],
        bump = oracle.bump
    )]
    pub oracle: Account<'info, Oracle>,

    #[account(mut)]
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(market_id: u64)]
pub struct CreateMarket<'info> {
    #[account(
        seeds = [PROTOCOL_SEED],
        bump = protocol_state.bump
    )]
    pub protocol_state: Account<'info, ProtocolState>,

    #[account(
        init,
        payer = creator,
        space = 8 + Market::INIT_SPACE,
        seeds = [MARKET_SEED, &market_id.to_le_bytes()],
        bump
    )]
    pub market: Account<'info, Market>,

    /// The token mint for betting (e.g., USDC)
    pub token_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = creator,
        token::mint = token_mint,
        token::authority = market,
        seeds = [MARKET_VAULT_SEED, market.key().as_ref()],
        bump
    )]
    pub market_vault: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = creator,
        token::mint = token_mint,
        token::authority = market,
        seeds = [POOL_VAULT_SEED, market.key().as_ref()],
        bump
    )]
    pub pool_vault: Account<'info, TokenAccount>,

    /// Optional license account - required if protocol.require_license is true
    #[account(
        mut,
        seeds = [LICENSE_SEED, &license.license_key],
        bump = license.bump
    )]
    pub license: Option<Account<'info, License>>,

    #[account(mut)]
    pub creator: Signer<'info>,

    /// CHECK: Creator's wallet to receive creator fees
    pub creator_fee_wallet: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct AssignOracle<'info> {
    #[account(
        mut,
        seeds = [MARKET_SEED, &market.market_id.to_le_bytes()],
        bump = market.bump,
        constraint = market.status == MarketStatus::Open @ FortunaError::MarketNotOpen,
        constraint = market.creator == creator.key() @ FortunaError::Unauthorized,
        constraint = market.oracle == Pubkey::default() @ FortunaError::MarketAlreadyHasOracle
    )]
    pub market: Account<'info, Market>,

    #[account(
        seeds = [ORACLE_SEED, &oracle.oracle_id.to_le_bytes()],
        bump = oracle.bump,
        constraint = oracle.is_active @ FortunaError::OracleNotActive
    )]
    pub oracle: Account<'info, Oracle>,

    #[account(mut)]
    pub creator: Signer<'info>,
}

#[derive(Accounts)]
pub struct PlaceBet<'info> {
    #[account(
        seeds = [PROTOCOL_SEED],
        bump = protocol_state.bump
    )]
    pub protocol_state: Account<'info, ProtocolState>,

    #[account(
        mut,
        seeds = [MARKET_SEED, &market.market_id.to_le_bytes()],
        bump = market.bump,
        constraint = market.status == MarketStatus::Open @ FortunaError::MarketNotOpen
    )]
    pub market: Account<'info, Market>,

    #[account(
        init,
        payer = bettor,
        space = 8 + Bet::INIT_SPACE,
        seeds = [BET_SEED, market.key().as_ref(), bettor.key().as_ref()],
        bump
    )]
    pub bet: Account<'info, Bet>,

    #[account(
        mut,
        seeds = [MARKET_VAULT_SEED, market.key().as_ref()],
        bump = market.vault_bump
    )]
    pub market_vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [POOL_VAULT_SEED, market.key().as_ref()],
        bump = market.pool_vault_bump
    )]
    pub pool_vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = bettor_token_account.owner == bettor.key(),
        constraint = bettor_token_account.mint == market.token_mint
    )]
    pub bettor_token_account: Account<'info, TokenAccount>,

    /// CHECK: Treasury wallet to receive protocol fees
    #[account(
        mut,
        constraint = treasury_token_account.owner == protocol_state.treasury,
        constraint = treasury_token_account.mint == market.token_mint
    )]
    pub treasury_token_account: Account<'info, TokenAccount>,

    /// CHECK: Creator's token account for fees
    #[account(
        mut,
        constraint = creator_token_account.owner == market.creator_fee_wallet,
        constraint = creator_token_account.mint == market.token_mint
    )]
    pub creator_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub bettor: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ResolveMarket<'info> {
    #[account(
        mut,
        seeds = [MARKET_SEED, &market.market_id.to_le_bytes()],
        bump = market.bump,
        constraint = market.status == MarketStatus::Open @ FortunaError::MarketNotOpen,
        constraint = market.creator == resolver.key() @ FortunaError::Unauthorized
    )]
    pub market: Account<'info, Market>,

    #[account(mut)]
    pub resolver: Signer<'info>,
}

#[derive(Accounts)]
pub struct OracleResolveMarket<'info> {
    #[account(
        mut,
        seeds = [MARKET_SEED, &market.market_id.to_le_bytes()],
        bump = market.bump,
        constraint = market.status == MarketStatus::Open @ FortunaError::MarketNotOpen,
        constraint = market.oracle == oracle.key() @ FortunaError::OracleMismatch
    )]
    pub market: Account<'info, Market>,

    #[account(
        mut,
        seeds = [ORACLE_SEED, &oracle.oracle_id.to_le_bytes()],
        bump = oracle.bump,
        constraint = oracle.is_active @ FortunaError::OracleNotActive,
        constraint = oracle.authority == oracle_authority.key() @ FortunaError::Unauthorized
    )]
    pub oracle: Account<'info, Oracle>,

    #[account(mut)]
    pub oracle_authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct ClaimWinnings<'info> {
    #[account(
        seeds = [PROTOCOL_SEED],
        bump = protocol_state.bump
    )]
    pub protocol_state: Account<'info, ProtocolState>,

    #[account(
        seeds = [MARKET_SEED, &market.market_id.to_le_bytes()],
        bump = market.bump,
        constraint = market.status == MarketStatus::Resolved @ FortunaError::MarketNotResolved
    )]
    pub market: Account<'info, Market>,

    #[account(
        mut,
        seeds = [BET_SEED, market.key().as_ref(), claimer.key().as_ref()],
        bump = bet.bump,
        constraint = bet.bettor == claimer.key() @ FortunaError::Unauthorized,
        constraint = !bet.claimed @ FortunaError::AlreadyClaimed
    )]
    pub bet: Account<'info, Bet>,

    #[account(
        mut,
        seeds = [MARKET_VAULT_SEED, market.key().as_ref()],
        bump = market.vault_bump
    )]
    pub market_vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = claimer_token_account.owner == claimer.key(),
        constraint = claimer_token_account.mint == market.token_mint
    )]
    pub claimer_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub claimer: Signer<'info>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct CancelMarket<'info> {
    #[account(
        mut,
        seeds = [MARKET_SEED, &market.market_id.to_le_bytes()],
        bump = market.bump,
        constraint = market.status == MarketStatus::Open @ FortunaError::MarketNotOpen,
        constraint = market.creator == authority.key() @ FortunaError::Unauthorized
    )]
    pub market: Account<'info, Market>,

    #[account(mut)]
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct ClaimRefund<'info> {
    #[account(
        seeds = [MARKET_SEED, &market.market_id.to_le_bytes()],
        bump = market.bump,
        constraint = market.status == MarketStatus::Cancelled @ FortunaError::MarketNotCancelled
    )]
    pub market: Account<'info, Market>,

    #[account(
        mut,
        seeds = [BET_SEED, market.key().as_ref(), claimer.key().as_ref()],
        bump = bet.bump,
        constraint = bet.bettor == claimer.key() @ FortunaError::Unauthorized,
        constraint = !bet.claimed @ FortunaError::AlreadyClaimed
    )]
    pub bet: Account<'info, Bet>,

    #[account(
        mut,
        seeds = [MARKET_VAULT_SEED, market.key().as_ref()],
        bump = market.vault_bump
    )]
    pub market_vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = claimer_token_account.owner == claimer.key(),
        constraint = claimer_token_account.mint == market.token_mint
    )]
    pub claimer_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub claimer: Signer<'info>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct WithdrawBet<'info> {
    #[account(
        mut,
        seeds = [MARKET_SEED, &market.market_id.to_le_bytes()],
        bump = market.bump,
        constraint = market.status == MarketStatus::Open @ FortunaError::MarketNotOpen
    )]
    pub market: Account<'info, Market>,

    #[account(
        mut,
        seeds = [BET_SEED, market.key().as_ref(), bettor.key().as_ref()],
        bump = bet.bump,
        constraint = bet.bettor == bettor.key() @ FortunaError::Unauthorized,
        constraint = !bet.claimed @ FortunaError::BetAlreadyWithdrawn
    )]
    pub bet: Account<'info, Bet>,

    #[account(
        mut,
        seeds = [MARKET_VAULT_SEED, market.key().as_ref()],
        bump = market.vault_bump
    )]
    pub market_vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = bettor_token_account.owner == bettor.key(),
        constraint = bettor_token_account.mint == market.token_mint
    )]
    pub bettor_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub bettor: Signer<'info>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct UpdateProtocol<'info> {
    #[account(
        mut,
        seeds = [PROTOCOL_SEED],
        bump = protocol_state.bump,
        constraint = protocol_state.authority == authority.key() @ FortunaError::Unauthorized
    )]
    pub protocol_state: Account<'info, ProtocolState>,

    #[account(mut)]
    pub authority: Signer<'info>,
}

// ============================================================================
// License Account Contexts
// ============================================================================

#[derive(Accounts)]
#[instruction(license_key: [u8; 32])]
pub struct IssueLicense<'info> {
    #[account(
        mut,
        seeds = [PROTOCOL_SEED],
        bump = protocol_state.bump,
        constraint = protocol_state.authority == authority.key() @ FortunaError::Unauthorized
    )]
    pub protocol_state: Account<'info, ProtocolState>,

    #[account(
        init,
        payer = authority,
        space = 8 + License::INIT_SPACE,
        seeds = [LICENSE_SEED, &license_key],
        bump
    )]
    pub license: Account<'info, License>,

    /// CHECK: The wallet that will hold this license
    pub holder: UncheckedAccount<'info>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RevokeLicense<'info> {
    #[account(
        seeds = [PROTOCOL_SEED],
        bump = protocol_state.bump,
        constraint = protocol_state.authority == authority.key() @ FortunaError::Unauthorized
    )]
    pub protocol_state: Account<'info, ProtocolState>,

    #[account(
        mut,
        seeds = [LICENSE_SEED, &license.license_key],
        bump = license.bump
    )]
    pub license: Account<'info, License>,

    #[account(mut)]
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct TransferLicense<'info> {
    #[account(
        mut,
        seeds = [LICENSE_SEED, &license.license_key],
        bump = license.bump,
        constraint = license.holder == current_holder.key() @ FortunaError::Unauthorized,
        constraint = license.is_transferable @ FortunaError::LicenseNotTransferable
    )]
    pub license: Account<'info, License>,

    /// CHECK: The new holder of the license
    pub new_holder: UncheckedAccount<'info>,

    #[account(mut)]
    pub current_holder: Signer<'info>,
}

#[derive(Accounts)]
pub struct UpdateLicense<'info> {
    #[account(
        seeds = [PROTOCOL_SEED],
        bump = protocol_state.bump,
        constraint = protocol_state.authority == authority.key() @ FortunaError::Unauthorized
    )]
    pub protocol_state: Account<'info, ProtocolState>,

    #[account(
        mut,
        seeds = [LICENSE_SEED, &license.license_key],
        bump = license.bump
    )]
    pub license: Account<'info, License>,

    #[account(mut)]
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct ModifyLicenseWallets<'info> {
    #[account(
        mut,
        seeds = [LICENSE_SEED, &license.license_key],
        bump = license.bump,
        constraint = license.holder == holder.key() @ FortunaError::Unauthorized
    )]
    pub license: Account<'info, License>,

    #[account(mut)]
    pub holder: Signer<'info>,
}

#[derive(Accounts)]
pub struct ModifyLicenseDomains<'info> {
    #[account(
        mut,
        seeds = [LICENSE_SEED, &license.license_key],
        bump = license.bump,
        constraint = license.holder == holder.key() @ FortunaError::Unauthorized
    )]
    pub license: Account<'info, License>,

    #[account(mut)]
    pub holder: Signer<'info>,
}
