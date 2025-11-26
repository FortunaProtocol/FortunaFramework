import { PublicKey } from '@solana/web3.js';

// Program ID - Update this after deployment
export const FORTUNA_PROGRAM_ID = new PublicKey('FortunaProt1111111111111111111111111111111');

// Treasury wallet address
export const TREASURY_WALLET = new PublicKey('6Lbx8fvKRf1aE8Zi977sGHYqNeKvzxyjnGt5pee9FwoZ');

// PDA Seeds
export const PROTOCOL_SEED = Buffer.from('protocol');
export const MARKET_SEED = Buffer.from('market');
export const MARKET_VAULT_SEED = Buffer.from('market_vault');
export const POOL_VAULT_SEED = Buffer.from('pool_vault');
export const BET_SEED = Buffer.from('bet');
export const ORACLE_SEED = Buffer.from('oracle');
export const LICENSE_SEED = Buffer.from('license');

// Default fee configuration (in basis points)
export const DEFAULT_PROTOCOL_FEE_BPS = 50;  // 0.5%
export const DEFAULT_CREATOR_FEE_BPS = 50;   // 0.5%
export const DEFAULT_POOL_FEE_BPS = 500;     // 5%

// Maximum values
export const MAX_TOTAL_FEE_BPS = 1000;       // 10%
export const MAX_OUTCOMES = 10;
export const MAX_TITLE_LENGTH = 128;
export const MAX_DESCRIPTION_LENGTH = 512;
export const MAX_OUTCOME_LENGTH = 64;
export const MAX_ORACLE_NAME_LENGTH = 64;
export const MAX_DATA_SOURCE_LENGTH = 256;
export const MAX_LICENSE_DOMAINS = 5;
export const MAX_LICENSE_WALLETS = 10;
export const MAX_DOMAIN_NAME_LENGTH = 64;

// Basis points denominator
export const BPS_DENOMINATOR = 10000;

/**
 * Market categories for prediction markets
 */
export enum MarketCategory {
  /** Political events and outcomes */
  Politics = 0,
  /** Sports events and matches */
  Sports = 1,
  /** Financial markets and indices */
  Finance = 2,
  /** Cryptocurrency prices and events */
  Crypto = 3,
  /** Geopolitical events */
  Geopolitics = 4,
  /** Company earnings reports */
  Earnings = 5,
  /** Technology events and releases */
  Tech = 6,
  /** Cultural events, entertainment, awards */
  Culture = 7,
  /** World events and news */
  World = 8,
  /** Economic indicators and data */
  Economy = 9,
  /** Election outcomes */
  Elections = 10,
  /** Social media mentions and trends */
  Mentions = 11,
}

/**
 * Category names for display
 */
export const CATEGORY_NAMES: Record<MarketCategory, string> = {
  [MarketCategory.Politics]: 'Politics',
  [MarketCategory.Sports]: 'Sports',
  [MarketCategory.Finance]: 'Finance',
  [MarketCategory.Crypto]: 'Crypto',
  [MarketCategory.Geopolitics]: 'Geopolitics',
  [MarketCategory.Earnings]: 'Earnings',
  [MarketCategory.Tech]: 'Tech',
  [MarketCategory.Culture]: 'Culture',
  [MarketCategory.World]: 'World',
  [MarketCategory.Economy]: 'Economy',
  [MarketCategory.Elections]: 'Elections',
  [MarketCategory.Mentions]: 'Mentions',
};

/**
 * Get category name from enum value
 */
export function getCategoryName(category: MarketCategory): string {
  return CATEGORY_NAMES[category] || 'Unknown';
}

/**
 * Get all categories
 */
export function getAllCategories(): MarketCategory[] {
  return [
    MarketCategory.Politics,
    MarketCategory.Sports,
    MarketCategory.Finance,
    MarketCategory.Crypto,
    MarketCategory.Geopolitics,
    MarketCategory.Earnings,
    MarketCategory.Tech,
    MarketCategory.Culture,
    MarketCategory.World,
    MarketCategory.Economy,
    MarketCategory.Elections,
    MarketCategory.Mentions,
  ];
}
