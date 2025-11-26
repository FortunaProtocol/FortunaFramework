use anchor_lang::prelude::*;
use anchor_spl::token::{self, Transfer};

use crate::state::*;
use crate::errors::*;
use crate::constants::*;
use crate::{
    InitializeProtocol, RegisterOracle, UpdateOracle, CreateMarket, AssignOracle,
    PlaceBet, ResolveMarket, OracleResolveMarket, ClaimWinnings, CancelMarket,
    ClaimRefund, WithdrawBet, UpdateProtocol,
    IssueLicense, RevokeLicense, TransferLicense, UpdateLicense,
    ModifyLicenseWallets, ModifyLicenseDomains,
};

/// Initialize the protocol with treasury and fee settings
pub fn initialize_protocol(
    ctx: Context<InitializeProtocol>,
    protocol_fee_bps: u16,
    creator_fee_bps: u16,
    pool_fee_bps: u16,
) -> Result<()> {
    let total_fee = protocol_fee_bps + creator_fee_bps + pool_fee_bps;
    require!(total_fee <= MAX_TOTAL_FEE_BPS, FortunaError::InvalidFeeConfig);

    let protocol_state = &mut ctx.accounts.protocol_state;

    protocol_state.authority = ctx.accounts.authority.key();
    protocol_state.treasury = ctx.accounts.treasury.key();
    protocol_state.protocol_fee_bps = protocol_fee_bps;
    protocol_state.creator_fee_bps = creator_fee_bps;
    protocol_state.pool_fee_bps = pool_fee_bps;
    protocol_state.total_markets = 0;
    protocol_state.total_volume = 0;
    protocol_state.total_oracles = 0;
    protocol_state.total_licenses = 0;
    protocol_state.require_license = false;
    protocol_state.bump = ctx.bumps.protocol_state;
    protocol_state.reserved = vec![];

    msg!("Protocol initialized with fees: pool={}bps, creator={}bps, protocol={}bps",
        pool_fee_bps, creator_fee_bps, protocol_fee_bps);

    Ok(())
}

/// Register a new oracle for automated market resolution
pub fn register_oracle(
    ctx: Context<RegisterOracle>,
    oracle_id: u32,
    name: String,
    categories: [bool; 12],
    data_source: String,
) -> Result<()> {
    require!(name.len() <= MAX_ORACLE_NAME_LEN, FortunaError::OracleNameTooLong);
    require!(data_source.len() <= MAX_DATA_SOURCE_LEN, FortunaError::DataSourceTooLong);

    let clock = Clock::get()?;
    let oracle = &mut ctx.accounts.oracle;
    let protocol_state = &mut ctx.accounts.protocol_state;

    oracle.oracle_id = oracle_id;
    oracle.authority = ctx.accounts.oracle_authority.key();
    oracle.name = name.clone();
    oracle.categories = categories;
    oracle.data_source = data_source;
    oracle.is_active = true;
    oracle.markets_resolved = 0;
    oracle.registered_at = clock.unix_timestamp;
    oracle.last_resolution_at = 0;
    oracle.bump = ctx.bumps.oracle;
    oracle.reserved = vec![];

    protocol_state.total_oracles = protocol_state.total_oracles.checked_add(1)
        .ok_or(FortunaError::Overflow)?;

    msg!("Oracle registered: {} (ID: {})", name, oracle_id);

    Ok(())
}

/// Update oracle settings
pub fn update_oracle(
    ctx: Context<UpdateOracle>,
    name: Option<String>,
    categories: Option<[bool; 12]>,
    data_source: Option<String>,
    is_active: Option<bool>,
) -> Result<()> {
    let oracle = &mut ctx.accounts.oracle;

    if let Some(new_name) = name {
        require!(new_name.len() <= MAX_ORACLE_NAME_LEN, FortunaError::OracleNameTooLong);
        oracle.name = new_name;
    }

    if let Some(new_categories) = categories {
        oracle.categories = new_categories;
    }

    if let Some(new_data_source) = data_source {
        require!(new_data_source.len() <= MAX_DATA_SOURCE_LEN, FortunaError::DataSourceTooLong);
        oracle.data_source = new_data_source;
    }

    if let Some(active) = is_active {
        oracle.is_active = active;
    }

    msg!("Oracle updated: {}", oracle.name);

    Ok(())
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
    let protocol_state = &ctx.accounts.protocol_state;
    let clock = Clock::get()?;
    let current_time = clock.unix_timestamp;

    // Check license if required
    if protocol_state.require_license {
        let license = ctx.accounts.license.as_mut()
            .ok_or(FortunaError::LicenseRequired)?;

        // Validate license is active and not expired
        require!(license.is_valid(current_time), FortunaError::LicenseExpired);
        require!(license.is_active, FortunaError::LicenseNotActive);

        // Validate wallet is authorized
        require!(
            license.is_wallet_authorized(&ctx.accounts.creator.key()),
            FortunaError::WalletNotAuthorized
        );

        // Validate license can create markets
        require!(license.can_create_market(), FortunaError::LicenseMarketLimitReached);
        require!(license.features.can_create_markets, FortunaError::FeatureNotEnabled);

        // Update license usage
        license.markets_created = license.markets_created.checked_add(1)
            .ok_or(FortunaError::Overflow)?;
        license.last_used_at = current_time;
    }

    // Validate inputs
    require!(title.len() <= MAX_TITLE_LEN, FortunaError::TitleTooLong);
    require!(description.len() <= MAX_DESCRIPTION_LEN, FortunaError::DescriptionTooLong);
    require!(outcomes.len() >= 2, FortunaError::TooFewOutcomes);
    require!(outcomes.len() <= MAX_OUTCOMES, FortunaError::TooManyOutcomes);
    require!(bet_amount > 0, FortunaError::InvalidBetAmount);
    require!(oracle_event_id.len() <= 64, FortunaError::OracleEventIdTooLong);

    // Validate category
    let market_category = MarketCategory::from_u8(category)
        .ok_or(FortunaError::InvalidCategory)?;

    require!(betting_deadline > current_time, FortunaError::InvalidDeadline);
    require!(resolution_deadline >= betting_deadline, FortunaError::InvalidDeadline);

    // Validate outcome labels
    for outcome in &outcomes {
        require!(outcome.len() <= MAX_OUTCOME_LEN, FortunaError::OutcomeLabelTooLong);
    }

    let market = &mut ctx.accounts.market;

    market.market_id = market_id;
    market.creator = ctx.accounts.creator.key();
    market.creator_fee_wallet = ctx.accounts.creator_fee_wallet.key();
    market.token_mint = ctx.accounts.token_mint.key();
    market.category = market_category;
    market.oracle = Pubkey::default(); // No oracle assigned initially
    market.oracle_event_id = oracle_event_id;
    market.title = title.clone();
    market.description = description;
    market.bet_amount = bet_amount;
    market.betting_deadline = betting_deadline;
    market.resolution_deadline = resolution_deadline;
    market.status = MarketStatus::Open;
    market.winning_outcome = 0;
    market.total_pool = 0;
    market.bonus_pool = 0;
    market.created_at = current_time;
    market.resolved_at = 0;
    market.resolved_by_oracle = false;
    market.vault_bump = ctx.bumps.market_vault;
    market.pool_vault_bump = ctx.bumps.pool_vault;
    market.bump = ctx.bumps.market;
    market.reserved = vec![];

    // Initialize outcomes
    market.outcomes = outcomes
        .iter()
        .map(|label| Outcome {
            label: label.clone(),
            total_amount: 0,
            bettor_count: 0,
        })
        .collect();

    msg!("Market created: {} [{}] with {} outcomes, bet amount: {}",
        title, market_category.name(), market.outcomes.len(), bet_amount);

    Ok(())
}

/// Assign an oracle to a market for automated resolution
pub fn assign_oracle(ctx: Context<AssignOracle>) -> Result<()> {
    let market = &mut ctx.accounts.market;
    let oracle = &ctx.accounts.oracle;

    // Verify oracle can resolve this category
    require!(
        oracle.can_resolve_category(market.category),
        FortunaError::OracleNotAuthorizedForCategory
    );

    market.oracle = oracle.key();

    msg!("Oracle {} assigned to market {}", oracle.name, market.title);

    Ok(())
}

/// Place a bet on a specific outcome
pub fn place_bet(
    ctx: Context<PlaceBet>,
    outcome_index: u8,
) -> Result<()> {
    let market = &mut ctx.accounts.market;
    let protocol_state = &ctx.accounts.protocol_state;

    // Validate outcome
    require!(
        (outcome_index as usize) < market.outcomes.len(),
        FortunaError::InvalidOutcome
    );

    // Check betting deadline
    let clock = Clock::get()?;
    require!(
        !market.is_betting_closed(clock.unix_timestamp),
        FortunaError::BettingDeadlinePassed
    );

    let bet_amount = market.bet_amount;

    // Calculate fees
    let (pool_fee, creator_fee, protocol_fee, net_amount) =
        protocol_state.calculate_fees(bet_amount);

    // Transfer bet amount to market vault
    let cpi_accounts = Transfer {
        from: ctx.accounts.bettor_token_account.to_account_info(),
        to: ctx.accounts.market_vault.to_account_info(),
        authority: ctx.accounts.bettor.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program.clone(), cpi_accounts);
    token::transfer(cpi_ctx, net_amount)?;

    // Transfer pool fee to pool vault
    let cpi_accounts_pool = Transfer {
        from: ctx.accounts.bettor_token_account.to_account_info(),
        to: ctx.accounts.pool_vault.to_account_info(),
        authority: ctx.accounts.bettor.to_account_info(),
    };
    let cpi_ctx_pool = CpiContext::new(cpi_program.clone(), cpi_accounts_pool);
    token::transfer(cpi_ctx_pool, pool_fee)?;

    // Transfer protocol fee to treasury
    let cpi_accounts_treasury = Transfer {
        from: ctx.accounts.bettor_token_account.to_account_info(),
        to: ctx.accounts.treasury_token_account.to_account_info(),
        authority: ctx.accounts.bettor.to_account_info(),
    };
    let cpi_ctx_treasury = CpiContext::new(cpi_program.clone(), cpi_accounts_treasury);
    token::transfer(cpi_ctx_treasury, protocol_fee)?;

    // Transfer creator fee
    let cpi_accounts_creator = Transfer {
        from: ctx.accounts.bettor_token_account.to_account_info(),
        to: ctx.accounts.creator_token_account.to_account_info(),
        authority: ctx.accounts.bettor.to_account_info(),
    };
    let cpi_ctx_creator = CpiContext::new(cpi_program, cpi_accounts_creator);
    token::transfer(cpi_ctx_creator, creator_fee)?;

    // Update market state
    market.total_pool = market.total_pool.checked_add(net_amount)
        .ok_or(FortunaError::Overflow)?;
    market.bonus_pool = market.bonus_pool.checked_add(pool_fee)
        .ok_or(FortunaError::Overflow)?;

    // Update outcome
    let outcome = &mut market.outcomes[outcome_index as usize];
    outcome.total_amount = outcome.total_amount.checked_add(net_amount)
        .ok_or(FortunaError::Overflow)?;
    outcome.bettor_count = outcome.bettor_count.checked_add(1)
        .ok_or(FortunaError::Overflow)?;

    // Create bet record
    let bet = &mut ctx.accounts.bet;
    bet.market = ctx.accounts.market.key();
    bet.bettor = ctx.accounts.bettor.key();
    bet.outcome_index = outcome_index;
    bet.original_amount = bet_amount;
    bet.pool_amount = net_amount;
    bet.claimed = false;
    bet.placed_at = clock.unix_timestamp;
    bet.bump = ctx.bumps.bet;
    bet.reserved = vec![];

    msg!("Bet placed: {} on outcome {} (index {})",
        bet_amount, market.outcomes[outcome_index as usize].label, outcome_index);

    Ok(())
}

/// Resolve the market with the winning outcome (creator only)
pub fn resolve_market(
    ctx: Context<ResolveMarket>,
    winning_outcome: u8,
) -> Result<()> {
    let market = &mut ctx.accounts.market;

    // Validate winning outcome
    require!(
        (winning_outcome as usize) < market.outcomes.len(),
        FortunaError::InvalidOutcome
    );

    // Check if betting deadline has passed
    let clock = Clock::get()?;
    require!(
        market.is_betting_closed(clock.unix_timestamp),
        FortunaError::CannotResolveBeforeBettingDeadline
    );

    // Update market state
    market.status = MarketStatus::Resolved;
    market.winning_outcome = winning_outcome;
    market.resolved_at = clock.unix_timestamp;
    market.resolved_by_oracle = false;

    msg!("Market resolved by creator: winning outcome = {} ({})",
        winning_outcome, market.outcomes[winning_outcome as usize].label);

    Ok(())
}

/// Resolve the market via oracle (oracle authority only)
pub fn oracle_resolve_market(
    ctx: Context<OracleResolveMarket>,
    winning_outcome: u8,
) -> Result<()> {
    let market = &mut ctx.accounts.market;
    let oracle = &mut ctx.accounts.oracle;

    // Validate winning outcome
    require!(
        (winning_outcome as usize) < market.outcomes.len(),
        FortunaError::InvalidOutcome
    );

    // Verify oracle can resolve this category
    require!(
        oracle.can_resolve_category(market.category),
        FortunaError::OracleNotAuthorizedForCategory
    );

    // Check if betting deadline has passed
    let clock = Clock::get()?;
    require!(
        market.is_betting_closed(clock.unix_timestamp),
        FortunaError::CannotResolveBeforeBettingDeadline
    );

    // Update market state
    market.status = MarketStatus::Resolved;
    market.winning_outcome = winning_outcome;
    market.resolved_at = clock.unix_timestamp;
    market.resolved_by_oracle = true;

    // Update oracle stats
    oracle.markets_resolved = oracle.markets_resolved.checked_add(1)
        .ok_or(FortunaError::Overflow)?;
    oracle.last_resolution_at = clock.unix_timestamp;

    msg!("Market resolved by oracle {}: winning outcome = {} ({})",
        oracle.name, winning_outcome, market.outcomes[winning_outcome as usize].label);

    Ok(())
}

/// Claim winnings after market resolution
pub fn claim_winnings(ctx: Context<ClaimWinnings>) -> Result<()> {
    let market = &ctx.accounts.market;
    let bet = &mut ctx.accounts.bet;

    // Check if bet won
    require!(
        bet.outcome_index == market.winning_outcome,
        FortunaError::LostBet
    );

    // Calculate payout
    let payout = market.calculate_payout(bet);
    require!(payout > 0, FortunaError::LostBet);

    // Transfer winnings from market vault to claimer
    let market_id_bytes = market.market_id.to_le_bytes();
    let seeds = &[
        MARKET_SEED,
        market_id_bytes.as_ref(),
        &[market.bump],
    ];
    let signer = &[&seeds[..]];

    let cpi_accounts = Transfer {
        from: ctx.accounts.market_vault.to_account_info(),
        to: ctx.accounts.claimer_token_account.to_account_info(),
        authority: ctx.accounts.market.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
    token::transfer(cpi_ctx, payout)?;

    // Mark bet as claimed
    bet.claimed = true;

    msg!("Winnings claimed: {} tokens", payout);

    Ok(())
}

/// Cancel a market (only before any bets or by admin)
pub fn cancel_market(ctx: Context<CancelMarket>) -> Result<()> {
    let market = &mut ctx.accounts.market;

    // Update market status
    market.status = MarketStatus::Cancelled;

    msg!("Market cancelled: {}", market.title);

    Ok(())
}

/// Refund bet for cancelled market
pub fn claim_refund(ctx: Context<ClaimRefund>) -> Result<()> {
    let market = &ctx.accounts.market;
    let bet = &mut ctx.accounts.bet;

    // Transfer refund from market vault
    let market_id_bytes = market.market_id.to_le_bytes();
    let seeds = &[
        MARKET_SEED,
        market_id_bytes.as_ref(),
        &[market.bump],
    ];
    let signer = &[&seeds[..]];

    // Refund the pool amount (after fees were taken)
    let cpi_accounts = Transfer {
        from: ctx.accounts.market_vault.to_account_info(),
        to: ctx.accounts.claimer_token_account.to_account_info(),
        authority: ctx.accounts.market.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
    token::transfer(cpi_ctx, bet.pool_amount)?;

    // Mark bet as claimed
    bet.claimed = true;

    msg!("Refund claimed: {} tokens", bet.pool_amount);

    Ok(())
}

/// Withdraw a bet before market resolution (user gets back their stake minus fees)
pub fn withdraw_bet(ctx: Context<WithdrawBet>) -> Result<()> {
    let market = &mut ctx.accounts.market;
    let bet = &mut ctx.accounts.bet;

    // Check betting is still open (can only withdraw before deadline)
    let clock = Clock::get()?;
    require!(
        !market.is_betting_closed(clock.unix_timestamp),
        FortunaError::WithdrawDeadlinePassed
    );

    let withdraw_amount = bet.pool_amount;

    // Update market totals
    market.total_pool = market.total_pool.checked_sub(withdraw_amount)
        .ok_or(FortunaError::Overflow)?;

    // Update outcome totals
    let outcome = &mut market.outcomes[bet.outcome_index as usize];
    outcome.total_amount = outcome.total_amount.checked_sub(withdraw_amount)
        .ok_or(FortunaError::Overflow)?;
    outcome.bettor_count = outcome.bettor_count.checked_sub(1)
        .ok_or(FortunaError::Overflow)?;

    // Transfer tokens back to bettor from market vault
    let market_id_bytes = market.market_id.to_le_bytes();
    let seeds = &[
        MARKET_SEED,
        market_id_bytes.as_ref(),
        &[market.bump],
    ];
    let signer = &[&seeds[..]];

    let cpi_accounts = Transfer {
        from: ctx.accounts.market_vault.to_account_info(),
        to: ctx.accounts.bettor_token_account.to_account_info(),
        authority: ctx.accounts.market.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
    token::transfer(cpi_ctx, withdraw_amount)?;

    // Mark bet as claimed/withdrawn
    bet.claimed = true;

    msg!("Bet withdrawn: {} tokens (fees non-refundable)", withdraw_amount);

    Ok(())
}

/// Update protocol settings (admin only)
pub fn update_protocol(
    ctx: Context<UpdateProtocol>,
    new_treasury: Option<Pubkey>,
    new_protocol_fee_bps: Option<u16>,
    new_creator_fee_bps: Option<u16>,
    new_pool_fee_bps: Option<u16>,
) -> Result<()> {
    let protocol_state = &mut ctx.accounts.protocol_state;

    // Update treasury if provided
    if let Some(treasury) = new_treasury {
        protocol_state.treasury = treasury;
        msg!("Treasury updated to: {}", treasury);
    }

    // Calculate new total fee
    let protocol_fee = new_protocol_fee_bps.unwrap_or(protocol_state.protocol_fee_bps);
    let creator_fee = new_creator_fee_bps.unwrap_or(protocol_state.creator_fee_bps);
    let pool_fee = new_pool_fee_bps.unwrap_or(protocol_state.pool_fee_bps);

    let total_fee = protocol_fee + creator_fee + pool_fee;
    require!(total_fee <= MAX_TOTAL_FEE_BPS, FortunaError::InvalidFeeConfig);

    // Update fees if provided
    if let Some(fee) = new_protocol_fee_bps {
        protocol_state.protocol_fee_bps = fee;
        msg!("Protocol fee updated to: {}bps", fee);
    }

    if let Some(fee) = new_creator_fee_bps {
        protocol_state.creator_fee_bps = fee;
        msg!("Creator fee updated to: {}bps", fee);
    }

    if let Some(fee) = new_pool_fee_bps {
        protocol_state.pool_fee_bps = fee;
        msg!("Pool fee updated to: {}bps", fee);
    }

    Ok(())
}

/// Toggle whether license is required to create markets
pub fn set_require_license(
    ctx: Context<UpdateProtocol>,
    require_license: bool,
) -> Result<()> {
    let protocol_state = &mut ctx.accounts.protocol_state;
    protocol_state.require_license = require_license;
    msg!("License requirement set to: {}", require_license);
    Ok(())
}

// ============================================================================
// License Management
// ============================================================================

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
    // Validate license type
    let lt = LicenseType::from_u8(license_type)
        .ok_or(FortunaError::InvalidLicenseType)?;

    // Validate domains
    require!(allowed_domains.len() <= MAX_LICENSE_DOMAINS, FortunaError::TooManyDomains);
    for domain in &allowed_domains {
        require!(domain.len() <= MAX_DOMAIN_NAME_LEN, FortunaError::DomainTooLong);
    }

    // Validate wallets
    require!(allowed_wallets.len() <= MAX_LICENSE_WALLETS, FortunaError::TooManyWallets);

    let clock = Clock::get()?;
    let license = &mut ctx.accounts.license;
    let protocol_state = &mut ctx.accounts.protocol_state;

    license.license_key = license_key;
    license.holder = ctx.accounts.holder.key();
    license.license_type = lt;
    license.features = LicenseFeatures::for_license_type(lt);
    license.allowed_domains = allowed_domains;
    license.allowed_wallets = allowed_wallets;
    license.max_markets = if max_markets == 0 { lt.max_markets() } else { max_markets };
    license.markets_created = 0;
    license.is_active = true;
    license.is_transferable = is_transferable;
    license.issued_at = clock.unix_timestamp;
    license.expires_at = expires_at;
    license.last_used_at = 0;
    license.issued_by = ctx.accounts.authority.key();
    license.bump = ctx.bumps.license;
    license.reserved = vec![];

    protocol_state.total_licenses = protocol_state.total_licenses.checked_add(1)
        .ok_or(FortunaError::Overflow)?;

    msg!("License issued: {} license to {}", lt.name(), license.holder);

    Ok(())
}

/// Revoke/deactivate a license
pub fn revoke_license(ctx: Context<RevokeLicense>) -> Result<()> {
    let license = &mut ctx.accounts.license;
    license.is_active = false;
    msg!("License revoked for holder: {}", license.holder);
    Ok(())
}

/// Activate a previously deactivated license
pub fn activate_license(ctx: Context<RevokeLicense>) -> Result<()> {
    let license = &mut ctx.accounts.license;
    license.is_active = true;
    msg!("License activated for holder: {}", license.holder);
    Ok(())
}

/// Transfer a license to a new holder
pub fn transfer_license(ctx: Context<TransferLicense>) -> Result<()> {
    let license = &mut ctx.accounts.license;
    let old_holder = license.holder;
    license.holder = ctx.accounts.new_holder.key();
    // Clear allowed wallets on transfer (new holder can add their own)
    license.allowed_wallets = vec![];
    msg!("License transferred from {} to {}", old_holder, license.holder);
    Ok(())
}

/// Update license settings (admin only)
pub fn update_license(
    ctx: Context<UpdateLicense>,
    new_max_markets: Option<u32>,
    new_expires_at: Option<i64>,
    new_features: Option<LicenseFeatures>,
) -> Result<()> {
    let license = &mut ctx.accounts.license;

    if let Some(max_markets) = new_max_markets {
        license.max_markets = max_markets;
        msg!("License max markets updated to: {}", max_markets);
    }

    if let Some(expires_at) = new_expires_at {
        license.expires_at = expires_at;
        msg!("License expiration updated to: {}", expires_at);
    }

    if let Some(features) = new_features {
        license.features = features;
        msg!("License features updated");
    }

    Ok(())
}

/// Add an authorized wallet to a license
pub fn add_authorized_wallet(
    ctx: Context<ModifyLicenseWallets>,
    wallet: Pubkey,
) -> Result<()> {
    let license = &mut ctx.accounts.license;
    require!(license.allowed_wallets.len() < MAX_LICENSE_WALLETS, FortunaError::TooManyWallets);

    if !license.allowed_wallets.contains(&wallet) {
        license.allowed_wallets.push(wallet);
        msg!("Wallet {} added to license", wallet);
    }

    Ok(())
}

/// Remove an authorized wallet from a license
pub fn remove_authorized_wallet(
    ctx: Context<ModifyLicenseWallets>,
    wallet: Pubkey,
) -> Result<()> {
    let license = &mut ctx.accounts.license;
    license.allowed_wallets.retain(|w| *w != wallet);
    msg!("Wallet {} removed from license", wallet);
    Ok(())
}

/// Add an authorized domain to a license
pub fn add_authorized_domain(
    ctx: Context<ModifyLicenseDomains>,
    domain: String,
) -> Result<()> {
    let license = &mut ctx.accounts.license;
    require!(license.allowed_domains.len() < MAX_LICENSE_DOMAINS, FortunaError::TooManyDomains);
    require!(domain.len() <= MAX_DOMAIN_NAME_LEN, FortunaError::DomainTooLong);

    if !license.allowed_domains.contains(&domain) {
        license.allowed_domains.push(domain.clone());
        msg!("Domain {} added to license", domain);
    }

    Ok(())
}

/// Remove an authorized domain from a license
pub fn remove_authorized_domain(
    ctx: Context<ModifyLicenseDomains>,
    domain: String,
) -> Result<()> {
    let license = &mut ctx.accounts.license;
    license.allowed_domains.retain(|d| *d != domain);
    msg!("Domain {} removed from license", domain);
    Ok(())
}
