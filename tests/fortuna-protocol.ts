import * as anchor from '@coral-xyz/anchor';
import { Program, BN } from '@coral-xyz/anchor';
import {
  Keypair,
  PublicKey,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
  LAMPORTS_PER_SOL,
} from '@solana/web3.js';
import {
  TOKEN_PROGRAM_ID,
  createMint,
  createAccount,
  mintTo,
  getAccount,
  getAssociatedTokenAddress,
  createAssociatedTokenAccountInstruction,
} from '@solana/spl-token';
import { expect } from 'chai';
import { FortunaProtocol } from '../target/types/fortuna_protocol';

describe('fortuna-protocol', () => {
  // Configure the client
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.FortunaProtocol as Program<FortunaProtocol>;

  // Test accounts
  let authority: Keypair;
  let treasury: Keypair;
  let creator: Keypair;
  let bettor1: Keypair;
  let bettor2: Keypair;

  // Token accounts
  let tokenMint: PublicKey;
  let treasuryTokenAccount: PublicKey;
  let creatorTokenAccount: PublicKey;
  let bettor1TokenAccount: PublicKey;
  let bettor2TokenAccount: PublicKey;

  // PDAs
  let protocolStatePDA: PublicKey;
  let protocolStateBump: number;

  // Constants
  const PROTOCOL_SEED = Buffer.from('protocol');
  const MARKET_SEED = Buffer.from('market');
  const MARKET_VAULT_SEED = Buffer.from('market_vault');
  const POOL_VAULT_SEED = Buffer.from('pool_vault');
  const BET_SEED = Buffer.from('bet');

  // Fee configuration (in basis points)
  const PROTOCOL_FEE_BPS = 50;  // 0.5%
  const CREATOR_FEE_BPS = 50;   // 0.5%
  const POOL_FEE_BPS = 500;     // 5%

  // Test market configuration
  const MARKET_ID = new BN(1);
  const BET_AMOUNT = new BN(10_000_000); // 10 USDC (6 decimals)

  before(async () => {
    // Generate keypairs
    authority = Keypair.generate();
    treasury = Keypair.generate();
    creator = Keypair.generate();
    bettor1 = Keypair.generate();
    bettor2 = Keypair.generate();

    // Airdrop SOL to accounts
    const airdropPromises = [authority, treasury, creator, bettor1, bettor2].map(
      async (kp) => {
        const sig = await provider.connection.requestAirdrop(
          kp.publicKey,
          10 * LAMPORTS_PER_SOL
        );
        await provider.connection.confirmTransaction(sig);
      }
    );
    await Promise.all(airdropPromises);

    // Create token mint (simulating USDC with 6 decimals)
    tokenMint = await createMint(
      provider.connection,
      authority,
      authority.publicKey,
      null,
      6 // decimals
    );

    // Create token accounts
    treasuryTokenAccount = await createAccount(
      provider.connection,
      authority,
      tokenMint,
      treasury.publicKey
    );

    creatorTokenAccount = await createAccount(
      provider.connection,
      authority,
      tokenMint,
      creator.publicKey
    );

    bettor1TokenAccount = await createAccount(
      provider.connection,
      authority,
      tokenMint,
      bettor1.publicKey
    );

    bettor2TokenAccount = await createAccount(
      provider.connection,
      authority,
      tokenMint,
      bettor2.publicKey
    );

    // Mint tokens to bettors (100 USDC each)
    const mintAmount = 100_000_000; // 100 USDC
    await mintTo(
      provider.connection,
      authority,
      tokenMint,
      bettor1TokenAccount,
      authority,
      mintAmount
    );
    await mintTo(
      provider.connection,
      authority,
      tokenMint,
      bettor2TokenAccount,
      authority,
      mintAmount
    );

    // Derive protocol state PDA
    [protocolStatePDA, protocolStateBump] = PublicKey.findProgramAddressSync(
      [PROTOCOL_SEED],
      program.programId
    );
  });

  describe('initialize_protocol', () => {
    it('initializes the protocol with correct fees', async () => {
      await program.methods
        .initializeProtocol(PROTOCOL_FEE_BPS, CREATOR_FEE_BPS, POOL_FEE_BPS)
        .accounts({
          protocolState: protocolStatePDA,
          authority: authority.publicKey,
          treasury: treasury.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([authority])
        .rpc();

      const protocolState = await program.account.protocolState.fetch(
        protocolStatePDA
      );

      expect(protocolState.authority.toString()).to.equal(
        authority.publicKey.toString()
      );
      expect(protocolState.treasury.toString()).to.equal(
        treasury.publicKey.toString()
      );
      expect(protocolState.protocolFeeBps).to.equal(PROTOCOL_FEE_BPS);
      expect(protocolState.creatorFeeBps).to.equal(CREATOR_FEE_BPS);
      expect(protocolState.poolFeeBps).to.equal(POOL_FEE_BPS);
    });

    it('fails to initialize twice', async () => {
      try {
        await program.methods
          .initializeProtocol(PROTOCOL_FEE_BPS, CREATOR_FEE_BPS, POOL_FEE_BPS)
          .accounts({
            protocolState: protocolStatePDA,
            authority: authority.publicKey,
            treasury: treasury.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .signers([authority])
          .rpc();
        expect.fail('Should have thrown an error');
      } catch (error) {
        // Expected to fail
      }
    });
  });

  describe('create_market', () => {
    let marketPDA: PublicKey;
    let marketVaultPDA: PublicKey;
    let poolVaultPDA: PublicKey;

    before(async () => {
      [marketPDA] = PublicKey.findProgramAddressSync(
        [MARKET_SEED, MARKET_ID.toArrayLike(Buffer, 'le', 8)],
        program.programId
      );
      [marketVaultPDA] = PublicKey.findProgramAddressSync(
        [MARKET_VAULT_SEED, marketPDA.toBuffer()],
        program.programId
      );
      [poolVaultPDA] = PublicKey.findProgramAddressSync(
        [POOL_VAULT_SEED, marketPDA.toBuffer()],
        program.programId
      );
    });

    it('creates a market with valid parameters', async () => {
      const now = Math.floor(Date.now() / 1000);
      const bettingDeadline = new BN(now + 86400); // 1 day
      const resolutionDeadline = new BN(now + 172800); // 2 days

      await program.methods
        .createMarket(
          MARKET_ID,
          'Will BTC reach $100k?',
          'Bitcoin price prediction market',
          BET_AMOUNT,
          resolutionDeadline,
          bettingDeadline,
          ['Yes', 'No']
        )
        .accounts({
          protocolState: protocolStatePDA,
          market: marketPDA,
          tokenMint: tokenMint,
          marketVault: marketVaultPDA,
          poolVault: poolVaultPDA,
          creator: creator.publicKey,
          creatorFeeWallet: creator.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
          rent: SYSVAR_RENT_PUBKEY,
        })
        .signers([creator])
        .rpc();

      const market = await program.account.market.fetch(marketPDA);

      expect(market.marketId.toString()).to.equal(MARKET_ID.toString());
      expect(market.title).to.equal('Will BTC reach $100k?');
      expect(market.betAmount.toString()).to.equal(BET_AMOUNT.toString());
      expect(market.outcomes.length).to.equal(2);
      expect(market.outcomes[0].label).to.equal('Yes');
      expect(market.outcomes[1].label).to.equal('No');
      expect(market.status).to.deep.equal({ open: {} });
    });

    it('fails to create market with invalid deadline', async () => {
      const invalidMarketId = new BN(999);
      const [invalidMarketPDA] = PublicKey.findProgramAddressSync(
        [MARKET_SEED, invalidMarketId.toArrayLike(Buffer, 'le', 8)],
        program.programId
      );
      const [invalidMarketVaultPDA] = PublicKey.findProgramAddressSync(
        [MARKET_VAULT_SEED, invalidMarketPDA.toBuffer()],
        program.programId
      );
      const [invalidPoolVaultPDA] = PublicKey.findProgramAddressSync(
        [POOL_VAULT_SEED, invalidMarketPDA.toBuffer()],
        program.programId
      );

      const now = Math.floor(Date.now() / 1000);
      const pastDeadline = new BN(now - 86400); // 1 day ago

      try {
        await program.methods
          .createMarket(
            invalidMarketId,
            'Invalid Market',
            'Should fail',
            BET_AMOUNT,
            pastDeadline,
            pastDeadline,
            ['Yes', 'No']
          )
          .accounts({
            protocolState: protocolStatePDA,
            market: invalidMarketPDA,
            tokenMint: tokenMint,
            marketVault: invalidMarketVaultPDA,
            poolVault: invalidPoolVaultPDA,
            creator: creator.publicKey,
            creatorFeeWallet: creator.publicKey,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
            rent: SYSVAR_RENT_PUBKEY,
          })
          .signers([creator])
          .rpc();
        expect.fail('Should have thrown an error');
      } catch (error: any) {
        expect(error.error.errorCode.code).to.equal('InvalidDeadline');
      }
    });
  });

  describe('place_bet', () => {
    let marketPDA: PublicKey;
    let marketVaultPDA: PublicKey;
    let poolVaultPDA: PublicKey;
    let bettor1BetPDA: PublicKey;
    let bettor2BetPDA: PublicKey;

    before(async () => {
      [marketPDA] = PublicKey.findProgramAddressSync(
        [MARKET_SEED, MARKET_ID.toArrayLike(Buffer, 'le', 8)],
        program.programId
      );
      [marketVaultPDA] = PublicKey.findProgramAddressSync(
        [MARKET_VAULT_SEED, marketPDA.toBuffer()],
        program.programId
      );
      [poolVaultPDA] = PublicKey.findProgramAddressSync(
        [POOL_VAULT_SEED, marketPDA.toBuffer()],
        program.programId
      );
      [bettor1BetPDA] = PublicKey.findProgramAddressSync(
        [BET_SEED, marketPDA.toBuffer(), bettor1.publicKey.toBuffer()],
        program.programId
      );
      [bettor2BetPDA] = PublicKey.findProgramAddressSync(
        [BET_SEED, marketPDA.toBuffer(), bettor2.publicKey.toBuffer()],
        program.programId
      );
    });

    it('bettor1 places a bet on Yes (outcome 0)', async () => {
      const initialBalance = await getAccount(
        provider.connection,
        bettor1TokenAccount
      );

      await program.methods
        .placeBet(0) // Yes
        .accounts({
          protocolState: protocolStatePDA,
          market: marketPDA,
          bet: bettor1BetPDA,
          marketVault: marketVaultPDA,
          poolVault: poolVaultPDA,
          bettorTokenAccount: bettor1TokenAccount,
          treasuryTokenAccount: treasuryTokenAccount,
          creatorTokenAccount: creatorTokenAccount,
          bettor: bettor1.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([bettor1])
        .rpc();

      // Verify bet was created
      const bet = await program.account.bet.fetch(bettor1BetPDA);
      expect(bet.outcomeIndex).to.equal(0);
      expect(bet.originalAmount.toString()).to.equal(BET_AMOUNT.toString());
      expect(bet.claimed).to.be.false;

      // Verify token transfer
      const finalBalance = await getAccount(
        provider.connection,
        bettor1TokenAccount
      );
      const balanceDiff =
        Number(initialBalance.amount) - Number(finalBalance.amount);
      expect(balanceDiff).to.equal(BET_AMOUNT.toNumber());

      // Verify market updated
      const market = await program.account.market.fetch(marketPDA);
      expect(market.outcomes[0].bettorCount).to.equal(1);
    });

    it('bettor2 places a bet on No (outcome 1)', async () => {
      await program.methods
        .placeBet(1) // No
        .accounts({
          protocolState: protocolStatePDA,
          market: marketPDA,
          bet: bettor2BetPDA,
          marketVault: marketVaultPDA,
          poolVault: poolVaultPDA,
          bettorTokenAccount: bettor2TokenAccount,
          treasuryTokenAccount: treasuryTokenAccount,
          creatorTokenAccount: creatorTokenAccount,
          bettor: bettor2.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([bettor2])
        .rpc();

      const bet = await program.account.bet.fetch(bettor2BetPDA);
      expect(bet.outcomeIndex).to.equal(1);

      const market = await program.account.market.fetch(marketPDA);
      expect(market.outcomes[1].bettorCount).to.equal(1);
    });

    it('verifies fee distribution', async () => {
      // Check treasury received protocol fees (0.5% * 2 bets)
      const treasuryBalance = await getAccount(
        provider.connection,
        treasuryTokenAccount
      );
      const expectedProtocolFee = (BET_AMOUNT.toNumber() * PROTOCOL_FEE_BPS) / 10000;
      expect(Number(treasuryBalance.amount)).to.equal(expectedProtocolFee * 2);

      // Check creator received creator fees (0.5% * 2 bets)
      const creatorBalance = await getAccount(
        provider.connection,
        creatorTokenAccount
      );
      const expectedCreatorFee = (BET_AMOUNT.toNumber() * CREATOR_FEE_BPS) / 10000;
      expect(Number(creatorBalance.amount)).to.equal(expectedCreatorFee * 2);

      // Check pool vault received pool fees (5% * 2 bets)
      const poolBalance = await getAccount(provider.connection, poolVaultPDA);
      const expectedPoolFee = (BET_AMOUNT.toNumber() * POOL_FEE_BPS) / 10000;
      expect(Number(poolBalance.amount)).to.equal(expectedPoolFee * 2);
    });
  });

  describe('resolve_market', () => {
    let marketPDA: PublicKey;

    before(async () => {
      [marketPDA] = PublicKey.findProgramAddressSync(
        [MARKET_SEED, MARKET_ID.toArrayLike(Buffer, 'le', 8)],
        program.programId
      );
    });

    it('fails when non-creator tries to resolve', async () => {
      try {
        await program.methods
          .resolveMarket(0)
          .accounts({
            market: marketPDA,
            resolver: bettor1.publicKey,
          })
          .signers([bettor1])
          .rpc();
        expect.fail('Should have thrown an error');
      } catch (error: any) {
        expect(error.error.errorCode.code).to.equal('Unauthorized');
      }
    });

    // Note: In a real test, we would need to wait for betting deadline to pass
    // For this test, we skip the deadline check or use a special test configuration
  });

  describe('update_protocol', () => {
    it('allows authority to update fees', async () => {
      const newPoolFeeBps = 400; // 4%

      await program.methods
        .updateProtocol(null, null, null, newPoolFeeBps)
        .accounts({
          protocolState: protocolStatePDA,
          authority: authority.publicKey,
        })
        .signers([authority])
        .rpc();

      const protocolState = await program.account.protocolState.fetch(
        protocolStatePDA
      );
      expect(protocolState.poolFeeBps).to.equal(newPoolFeeBps);
    });

    it('fails when non-authority tries to update', async () => {
      try {
        await program.methods
          .updateProtocol(null, null, null, 300)
          .accounts({
            protocolState: protocolStatePDA,
            authority: bettor1.publicKey,
          })
          .signers([bettor1])
          .rpc();
        expect.fail('Should have thrown an error');
      } catch (error: any) {
        expect(error.error.errorCode.code).to.equal('Unauthorized');
      }
    });
  });

  describe('cancel_market', () => {
    let cancelMarketId: BN;
    let cancelMarketPDA: PublicKey;
    let cancelMarketVaultPDA: PublicKey;
    let cancelPoolVaultPDA: PublicKey;

    before(async () => {
      cancelMarketId = new BN(100);
      [cancelMarketPDA] = PublicKey.findProgramAddressSync(
        [MARKET_SEED, cancelMarketId.toArrayLike(Buffer, 'le', 8)],
        program.programId
      );
      [cancelMarketVaultPDA] = PublicKey.findProgramAddressSync(
        [MARKET_VAULT_SEED, cancelMarketPDA.toBuffer()],
        program.programId
      );
      [cancelPoolVaultPDA] = PublicKey.findProgramAddressSync(
        [POOL_VAULT_SEED, cancelMarketPDA.toBuffer()],
        program.programId
      );

      // Create a market to cancel
      const now = Math.floor(Date.now() / 1000);
      await program.methods
        .createMarket(
          cancelMarketId,
          'Market to Cancel',
          'This market will be cancelled',
          BET_AMOUNT,
          new BN(now + 172800),
          new BN(now + 86400),
          ['Yes', 'No']
        )
        .accounts({
          protocolState: protocolStatePDA,
          market: cancelMarketPDA,
          tokenMint: tokenMint,
          marketVault: cancelMarketVaultPDA,
          poolVault: cancelPoolVaultPDA,
          creator: creator.publicKey,
          creatorFeeWallet: creator.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
          rent: SYSVAR_RENT_PUBKEY,
        })
        .signers([creator])
        .rpc();
    });

    it('creator can cancel the market', async () => {
      await program.methods
        .cancelMarket()
        .accounts({
          market: cancelMarketPDA,
          authority: creator.publicKey,
        })
        .signers([creator])
        .rpc();

      const market = await program.account.market.fetch(cancelMarketPDA);
      expect(market.status).to.deep.equal({ cancelled: {} });
    });

    it('fails to cancel an already cancelled market', async () => {
      try {
        await program.methods
          .cancelMarket()
          .accounts({
            market: cancelMarketPDA,
            authority: creator.publicKey,
          })
          .signers([creator])
          .rpc();
        expect.fail('Should have thrown an error');
      } catch (error: any) {
        expect(error.error.errorCode.code).to.equal('MarketNotOpen');
      }
    });
  });
});
