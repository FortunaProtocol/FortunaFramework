import { PublicKey } from '@solana/web3.js';
import BN from 'bn.js';
import {
  FORTUNA_PROGRAM_ID,
  PROTOCOL_SEED,
  MARKET_SEED,
  MARKET_VAULT_SEED,
  POOL_VAULT_SEED,
  BET_SEED,
  ORACLE_SEED,
  LICENSE_SEED,
  BPS_DENOMINATOR,
} from './constants';
import { FeeBreakdown } from './types';

/**
 * Derive the protocol state PDA
 */
export function getProtocolStatePDA(
  programId: PublicKey = FORTUNA_PROGRAM_ID
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [PROTOCOL_SEED],
    programId
  );
}

/**
 * Derive the market PDA for a given market ID
 */
export function getMarketPDA(
  marketId: BN | number,
  programId: PublicKey = FORTUNA_PROGRAM_ID
): [PublicKey, number] {
  const id = typeof marketId === 'number' ? new BN(marketId) : marketId;
  return PublicKey.findProgramAddressSync(
    [MARKET_SEED, id.toArrayLike(Buffer, 'le', 8)],
    programId
  );
}

/**
 * Derive the market vault PDA
 */
export function getMarketVaultPDA(
  marketPubkey: PublicKey,
  programId: PublicKey = FORTUNA_PROGRAM_ID
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [MARKET_VAULT_SEED, marketPubkey.toBuffer()],
    programId
  );
}

/**
 * Derive the pool vault PDA
 */
export function getPoolVaultPDA(
  marketPubkey: PublicKey,
  programId: PublicKey = FORTUNA_PROGRAM_ID
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [POOL_VAULT_SEED, marketPubkey.toBuffer()],
    programId
  );
}

/**
 * Derive the bet PDA for a bettor on a specific market
 */
export function getBetPDA(
  marketPubkey: PublicKey,
  bettorPubkey: PublicKey,
  programId: PublicKey = FORTUNA_PROGRAM_ID
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [BET_SEED, marketPubkey.toBuffer(), bettorPubkey.toBuffer()],
    programId
  );
}

/**
 * Derive the oracle PDA for a given oracle ID
 */
export function getOraclePDA(
  oracleId: number,
  programId: PublicKey = FORTUNA_PROGRAM_ID
): [PublicKey, number] {
  const idBuffer = Buffer.alloc(4);
  idBuffer.writeUInt32LE(oracleId);
  return PublicKey.findProgramAddressSync(
    [ORACLE_SEED, idBuffer],
    programId
  );
}

/**
 * Derive the license PDA for a given license key
 */
export function getLicensePDA(
  licenseKey: number[] | Buffer | Uint8Array,
  programId: PublicKey = FORTUNA_PROGRAM_ID
): [PublicKey, number] {
  const keyBuffer = Buffer.from(licenseKey);
  return PublicKey.findProgramAddressSync(
    [LICENSE_SEED, keyBuffer],
    programId
  );
}

/**
 * Generate a license key from a string (hashes to 32 bytes)
 */
export function generateLicenseKey(seed: string): number[] {
  // Simple hash function for demo - in production use a proper hash
  const encoder = new TextEncoder();
  const data = encoder.encode(seed);
  const hash = new Uint8Array(32);
  for (let i = 0; i < data.length; i++) {
    hash[i % 32] ^= data[i];
  }
  return Array.from(hash);
}

/**
 * Calculate fee breakdown for a given bet amount
 */
export function calculateFees(
  amount: BN | number,
  protocolFeeBps: number,
  creatorFeeBps: number,
  poolFeeBps: number
): FeeBreakdown {
  const amountBN = typeof amount === 'number' ? new BN(amount) : amount;

  const poolFee = amountBN.mul(new BN(poolFeeBps)).div(new BN(BPS_DENOMINATOR));
  const creatorFee = amountBN.mul(new BN(creatorFeeBps)).div(new BN(BPS_DENOMINATOR));
  const protocolFee = amountBN.mul(new BN(protocolFeeBps)).div(new BN(BPS_DENOMINATOR));

  const totalFees = poolFee.add(creatorFee).add(protocolFee);
  const netAmount = amountBN.sub(totalFees);

  return {
    poolFee,
    creatorFee,
    protocolFee,
    netAmount,
    totalFees,
  };
}

/**
 * Calculate potential winnings for a bet
 */
export function calculatePotentialWinnings(
  betAmount: BN | number,
  currentOutcomeTotal: BN | number,
  totalPool: BN | number,
  bonusPool: BN | number,
  protocolFeeBps: number,
  creatorFeeBps: number,
  poolFeeBps: number
): BN {
  const fees = calculateFees(betAmount, protocolFeeBps, creatorFeeBps, poolFeeBps);

  const currentOutcomeBN = typeof currentOutcomeTotal === 'number'
    ? new BN(currentOutcomeTotal)
    : currentOutcomeTotal;
  const totalPoolBN = typeof totalPool === 'number' ? new BN(totalPool) : totalPool;
  const bonusPoolBN = typeof bonusPool === 'number' ? new BN(bonusPool) : bonusPool;

  // After this bet, the outcome total would be:
  const newOutcomeTotal = currentOutcomeBN.add(fees.netAmount);

  // Total distributable is total pool + bonus pool + this bet's net amount
  const totalDistributable = totalPoolBN.add(bonusPoolBN).add(fees.netAmount);

  // Share of winnings = (bet net amount / new outcome total) * total distributable
  if (newOutcomeTotal.isZero()) {
    return totalDistributable;
  }

  return fees.netAmount.mul(totalDistributable).div(newOutcomeTotal);
}

/**
 * Format BN amount to human-readable string with decimals
 */
export function formatAmount(amount: BN, decimals: number = 6): string {
  const str = amount.toString().padStart(decimals + 1, '0');
  const intPart = str.slice(0, -decimals) || '0';
  const decPart = str.slice(-decimals);
  return `${intPart}.${decPart}`;
}

/**
 * Parse human-readable amount to BN
 */
export function parseAmount(amount: string | number, decimals: number = 6): BN {
  const str = amount.toString();
  const [intPart, decPart = ''] = str.split('.');
  const paddedDec = decPart.padEnd(decimals, '0').slice(0, decimals);
  return new BN(intPart + paddedDec);
}

/**
 * Get Unix timestamp from Date or offset from now
 */
export function getTimestamp(date: Date | number): number {
  if (date instanceof Date) {
    return Math.floor(date.getTime() / 1000);
  }
  return date;
}

/**
 * Create a deadline timestamp from hours from now
 */
export function hoursFromNow(hours: number): number {
  return Math.floor(Date.now() / 1000) + hours * 3600;
}

/**
 * Create a deadline timestamp from days from now
 */
export function daysFromNow(days: number): number {
  return Math.floor(Date.now() / 1000) + days * 86400;
}
