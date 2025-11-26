use anchor_lang::prelude::*;

#[error_code]
pub enum FortunaError {
    #[msg("Market is not open for betting")]
    MarketNotOpen,

    #[msg("Market has not been resolved yet")]
    MarketNotResolved,

    #[msg("Market has not been cancelled")]
    MarketNotCancelled,

    #[msg("Betting deadline has passed")]
    BettingDeadlinePassed,

    #[msg("Resolution deadline has not passed yet")]
    ResolutionDeadlineNotReached,

    #[msg("Invalid outcome index")]
    InvalidOutcome,

    #[msg("Unauthorized action")]
    Unauthorized,

    #[msg("Winnings already claimed")]
    AlreadyClaimed,

    #[msg("Bet already placed for this market")]
    BetAlreadyPlaced,

    #[msg("Title too long")]
    TitleTooLong,

    #[msg("Description too long")]
    DescriptionTooLong,

    #[msg("Too many outcomes")]
    TooManyOutcomes,

    #[msg("Need at least 2 outcomes")]
    TooFewOutcomes,

    #[msg("Invalid bet amount")]
    InvalidBetAmount,

    #[msg("Invalid deadline configuration")]
    InvalidDeadline,

    #[msg("Invalid fee configuration")]
    InvalidFeeConfig,

    #[msg("Market has active bets and cannot be cancelled")]
    MarketHasBets,

    #[msg("Arithmetic overflow")]
    Overflow,

    #[msg("Market cannot be resolved before betting deadline")]
    CannotResolveBeforeBettingDeadline,

    #[msg("Outcome label too long")]
    OutcomeLabelTooLong,

    #[msg("Lost bet - no winnings to claim")]
    LostBet,

    #[msg("Insufficient funds")]
    InsufficientFunds,

    #[msg("Oracle is not active")]
    OracleNotActive,

    #[msg("Oracle not authorized for this category")]
    OracleNotAuthorizedForCategory,

    #[msg("Market does not have an assigned oracle")]
    MarketHasNoOracle,

    #[msg("Oracle mismatch - wrong oracle for this market")]
    OracleMismatch,

    #[msg("Invalid category")]
    InvalidCategory,

    #[msg("Oracle name too long")]
    OracleNameTooLong,

    #[msg("Data source URL too long")]
    DataSourceTooLong,

    #[msg("Oracle event ID too long")]
    OracleEventIdTooLong,

    #[msg("Market already has an oracle assigned")]
    MarketAlreadyHasOracle,

    #[msg("Bet already withdrawn or claimed")]
    BetAlreadyWithdrawn,

    #[msg("Cannot withdraw after betting deadline")]
    WithdrawDeadlinePassed,

    #[msg("Valid license required to perform this action")]
    LicenseRequired,

    #[msg("License is not active")]
    LicenseNotActive,

    #[msg("License has expired")]
    LicenseExpired,

    #[msg("License market limit reached")]
    LicenseMarketLimitReached,

    #[msg("Wallet not authorized under this license")]
    WalletNotAuthorized,

    #[msg("Domain not authorized under this license")]
    DomainNotAuthorized,

    #[msg("License is not transferable")]
    LicenseNotTransferable,

    #[msg("Invalid license type")]
    InvalidLicenseType,

    #[msg("Too many domains specified")]
    TooManyDomains,

    #[msg("Domain name too long")]
    DomainTooLong,

    #[msg("Too many wallets specified")]
    TooManyWallets,

    #[msg("Feature not enabled for this license")]
    FeatureNotEnabled,

    #[msg("License already exists for this key")]
    LicenseAlreadyExists,
}
