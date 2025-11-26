import { PublicKey } from '@solana/web3.js';
import BN from 'bn.js';
import { MarketCategory } from './constants';

/**
 * Market status enum
 */
export enum MarketStatus {
  Open = 0,
  Resolved = 1,
  Cancelled = 2,
}

/**
 * License type enum
 */
export enum LicenseType {
  Basic = 0,
  Pro = 1,
  Enterprise = 2,
  Custom = 3,
}

/**
 * License features flags
 */
export interface LicenseFeatures {
  canCreateMarkets: boolean;
  canUseOracles: boolean;
  canCreatePrivateMarkets: boolean;
  canSetCustomFees: boolean;
  reserved: boolean[];
}

/**
 * Outcome data for a market
 */
export interface Outcome {
  label: string;
  totalAmount: BN;
  bettorCount: number;
}

/**
 * Protocol state account data
 */
export interface ProtocolState {
  authority: PublicKey;
  treasury: PublicKey;
  protocolFeeBps: number;
  creatorFeeBps: number;
  poolFeeBps: number;
  totalMarkets: BN;
  totalVolume: BN;
  totalOracles: number;
  totalLicenses: number;
  requireLicense: boolean;
  bump: number;
}

/**
 * License account data
 */
export interface License {
  /** Hash of the license key */
  licenseKey: number[];
  /** Wallet that holds this license */
  holder: PublicKey;
  /** License type (Basic, Pro, Enterprise, Custom) */
  licenseType: LicenseType;
  /** Enabled features */
  features: LicenseFeatures;
  /** Allowed domains for domain locking */
  allowedDomains: string[];
  /** Allowed wallets for wallet locking */
  allowedWallets: PublicKey[];
  /** Max markets this license can create */
  maxMarkets: number;
  /** Current markets created */
  marketsCreated: number;
  /** Is license active */
  isActive: boolean;
  /** Is license transferable */
  isTransferable: boolean;
  /** When license was issued */
  issuedAt: BN;
  /** When license expires (0 = never) */
  expiresAt: BN;
  /** Last activity timestamp */
  lastUsedAt: BN;
  /** Who issued this license */
  issuedBy: PublicKey;
  /** Bump seed */
  bump: number;
}

/**
 * Oracle account data for automated market resolution
 */
export interface Oracle {
  /** Oracle identifier */
  oracleId: number;
  /** Oracle authority (can submit results) */
  authority: PublicKey;
  /** Oracle name */
  name: string;
  /** Categories this oracle can resolve (array of 12 booleans) */
  categories: boolean[];
  /** Data source URL or identifier */
  dataSource: string;
  /** Whether the oracle is active */
  isActive: boolean;
  /** Total markets resolved by this oracle */
  marketsResolved: BN;
  /** Timestamp when oracle was registered */
  registeredAt: BN;
  /** Last resolution timestamp */
  lastResolutionAt: BN;
  /** Bump seed for PDA */
  bump: number;
}

/**
 * Market account data
 */
export interface Market {
  marketId: BN;
  creator: PublicKey;
  creatorFeeWallet: PublicKey;
  tokenMint: PublicKey;
  /** Market category */
  category: MarketCategory;
  /** Assigned oracle for automated resolution */
  oracle: PublicKey;
  /** External event ID for oracle resolution */
  oracleEventId: string;
  title: string;
  description: string;
  betAmount: BN;
  bettingDeadline: BN;
  resolutionDeadline: BN;
  status: MarketStatus;
  winningOutcome: number;
  totalPool: BN;
  bonusPool: BN;
  outcomes: Outcome[];
  createdAt: BN;
  resolvedAt: BN;
  /** Whether market was resolved by oracle */
  resolvedByOracle: boolean;
  vaultBump: number;
  poolVaultBump: number;
  bump: number;
}

/**
 * Bet account data
 */
export interface Bet {
  market: PublicKey;
  bettor: PublicKey;
  outcomeIndex: number;
  originalAmount: BN;
  poolAmount: BN;
  claimed: boolean;
  placedAt: BN;
  bump: number;
}

/**
 * Configuration for creating a new market
 */
export interface CreateMarketConfig {
  /** Unique market identifier */
  marketId: BN | number;
  /** Market category */
  category: MarketCategory;
  /** Market title (max 128 chars) */
  title: string;
  /** Market description (max 512 chars) */
  description: string;
  /** Fixed bet amount in token's smallest unit */
  betAmount: BN | number;
  /** Unix timestamp for when betting closes */
  bettingDeadline: number;
  /** Unix timestamp for resolution deadline */
  resolutionDeadline: number;
  /** Array of outcome labels (2-10 outcomes) */
  outcomes: string[];
  /** Token mint for betting (e.g., USDC) */
  tokenMint: PublicKey;
  /** Wallet to receive creator fees */
  creatorFeeWallet: PublicKey;
  /** External event ID for oracle resolution (optional) */
  oracleEventId?: string;
  /** License key (required if protocol.requireLicense is true) */
  licenseKey?: number[];
}

/**
 * Configuration for issuing a license
 */
export interface IssueLicenseConfig {
  /** 32-byte license key hash */
  licenseKey: number[];
  /** License type */
  licenseType: LicenseType;
  /** Wallet to hold the license */
  holder: PublicKey;
  /** Allowed domains (optional) */
  allowedDomains?: string[];
  /** Allowed wallets (optional) */
  allowedWallets?: PublicKey[];
  /** Max markets (0 = use default for license type) */
  maxMarkets?: number;
  /** Whether license can be transferred */
  isTransferable?: boolean;
  /** Expiration timestamp (0 = never expires) */
  expiresAt?: number;
}

/**
 * Configuration for registering an oracle
 */
export interface RegisterOracleConfig {
  /** Unique oracle identifier */
  oracleId: number;
  /** Oracle name */
  name: string;
  /** Categories this oracle can resolve */
  categories: MarketCategory[];
  /** Data source URL or identifier */
  dataSource: string;
  /** Oracle authority wallet (can submit results) */
  oracleAuthority: PublicKey;
}

/**
 * Configuration for updating an oracle
 */
export interface UpdateOracleConfig {
  /** New oracle name (optional) */
  name?: string;
  /** New categories (optional) */
  categories?: MarketCategory[];
  /** New data source (optional) */
  dataSource?: string;
  /** Active status (optional) */
  isActive?: boolean;
}

/**
 * Configuration for initializing the protocol
 */
export interface InitializeProtocolConfig {
  /** Protocol fee in basis points (default: 50 = 0.5%) */
  protocolFeeBps?: number;
  /** Creator fee in basis points (default: 50 = 0.5%) */
  creatorFeeBps?: number;
  /** Pool fee in basis points (default: 500 = 5%) */
  poolFeeBps?: number;
  /** Treasury wallet to receive protocol fees (default: 6Lbx8fvKRf1aE8Zi977sGHYqNeKvzxyjnGt5pee9FwoZ) */
  treasury?: PublicKey;
}

/**
 * Fee breakdown for a bet
 */
export interface FeeBreakdown {
  /** Amount going to pool (5% default) */
  poolFee: BN;
  /** Amount going to creator (0.5% default) */
  creatorFee: BN;
  /** Amount going to protocol treasury (0.5% default) */
  protocolFee: BN;
  /** Net amount going to betting pool */
  netAmount: BN;
  /** Total fees */
  totalFees: BN;
}

/**
 * Market statistics
 */
export interface MarketStats {
  totalBettors: number;
  totalPool: BN;
  bonusPool: BN;
  category: MarketCategory;
  hasOracle: boolean;
  outcomeStats: {
    label: string;
    totalAmount: BN;
    bettorCount: number;
    percentage: number;
  }[];
}

/**
 * Oracle statistics
 */
export interface OracleStats {
  oracleId: number;
  name: string;
  marketsResolved: number;
  categoriesEnabled: MarketCategory[];
  isActive: boolean;
}

/**
 * Helper to convert category array to boolean array for oracle
 */
export function categoriesToBoolArray(categories: MarketCategory[]): boolean[] {
  const result = new Array(12).fill(false);
  for (const cat of categories) {
    if (cat >= 0 && cat < 12) {
      result[cat] = true;
    }
  }
  return result;
}

/**
 * Helper to convert boolean array to category array
 */
export function boolArrayToCategories(boolArray: boolean[]): MarketCategory[] {
  const categories: MarketCategory[] = [];
  for (let i = 0; i < boolArray.length && i < 12; i++) {
    if (boolArray[i]) {
      categories.push(i as MarketCategory);
    }
  }
  return categories;
}
