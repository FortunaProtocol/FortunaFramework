# Fortuna Protocol

A decentralized prediction market framework for Solana. Create and participate in prediction markets with fixed bet sizes, automated oracle resolution, and transparent fee distribution.

## Features

- **Fixed Bet Sizes**: All participants bet the same amount, ensuring fair play
- **Transparent Fee Structure**:
  - 5% to bonus pool (redistributed to winners)
  - 0.5% to market creator
  - 0.5% to protocol treasury (`6Lbx8fvKRf1aE8Zi977sGHYqNeKvzxyjnGt5pee9FwoZ`)
- **Market Categories**: 12 categories for organized prediction markets
- **Oracle System**: Automated market resolution via registered oracles
- **Multiple Outcomes**: Support for binary (Yes/No) or multi-outcome markets
- **On-chain Resolution**: Fully transparent market resolution
- **SPL Token Support**: Use any SPL token for betting (USDC, SOL, etc.)
- **License System**: Domain/wallet locking with tiered access control
- **Bet Withdrawal**: Users can withdraw bets before betting deadline (minus fees)

## Market Categories

| Category | Description |
|----------|-------------|
| Politics | Political events and outcomes |
| Sports | Sports events and matches |
| Finance | Financial markets and indices |
| Crypto | Cryptocurrency prices and events |
| Geopolitics | Geopolitical events |
| Earnings | Company earnings reports |
| Tech | Technology events and releases |
| Culture | Cultural events, entertainment, awards |
| World | World events and news |
| Economy | Economic indicators and data |
| Elections | Election outcomes |
| Mentions | Social media mentions and trends |

## Architecture

```
fortuna-protocol/
├── programs/
│   └── fortuna-protocol/     # Anchor program (smart contract)
│       └── src/
│           ├── lib.rs        # Program entry & account contexts
│           ├── state.rs      # Account structures (Market, Oracle, Bet)
│           ├── errors.rs     # Error definitions
│           ├── constants.rs  # Constants & seeds
│           └── instructions.rs # Instruction handlers
├── sdk/                      # TypeScript SDK
│   └── src/
│       ├── fortuna-client.ts # High-level client
│       ├── types.ts          # TypeScript types
│       ├── utils.ts          # Utility functions
│       └── constants.ts      # SDK constants & categories
├── tests/                    # Integration tests
└── target/                   # Build artifacts
```

## Quick Start

### Prerequisites

- [Rust](https://rustup.rs/) (1.70+)
- [Solana CLI](https://docs.solana.com/cli/install-solana-cli-tools) (1.16+)
- [Anchor](https://www.anchor-lang.com/docs/installation) (0.29+)
- [Node.js](https://nodejs.org/) (18+)

### Installation

```bash
# Clone the repository
git clone https://github.com/your-org/fortuna-protocol.git
cd fortuna-protocol

# Install dependencies
yarn install

# Build the program
anchor build

# Run tests
anchor test
```

### Deployment

```bash
# Deploy to devnet
anchor deploy --provider.cluster devnet

# Deploy to mainnet
anchor deploy --provider.cluster mainnet
```

## Fee Structure

| Fee Type | Percentage | Destination |
|----------|------------|-------------|
| Pool Fee | 5% | Bonus pool (distributed to winners) |
| Creator Fee | 0.5% | Market creator's wallet |
| Protocol Fee | 0.5% | Treasury wallet (`6Lbx8fvKRf1aE8Zi977sGHYqNeKvzxyjnGt5pee9FwoZ`) |

**Example**: For a $10 bet:
- $9.40 goes to the betting pool
- $0.50 goes to the bonus pool
- $0.05 goes to the creator
- $0.05 goes to the treasury

## Usage

### Initialize Protocol (One-time setup)

```typescript
import { FortunaClient } from '@fortuna-protocol/sdk';
import { Connection, Keypair } from '@solana/web3.js';

const connection = new Connection('https://api.devnet.solana.com');
const wallet = // your wallet
const client = new FortunaClient(connection, wallet);

// Initialize with default treasury (6Lbx8fvKRf1aE8Zi977sGHYqNeKvzxyjnGt5pee9FwoZ)
await client.initializeProtocol();

// Or initialize with custom settings
await client.initializeProtocol({
  treasury: customTreasuryWallet.publicKey, // Optional, defaults to protocol treasury
  protocolFeeBps: 50,   // 0.5%
  creatorFeeBps: 50,    // 0.5%
  poolFeeBps: 500,      // 5%
});
```

### Register an Oracle

```typescript
import { MarketCategory } from '@fortuna-protocol/sdk';

// Register an oracle for automated market resolution
await client.registerOracle({
  oracleId: 1,
  name: "Sports Oracle",
  categories: [MarketCategory.Sports, MarketCategory.Elections],
  dataSource: "https://api.sports-data.com",
  oracleAuthority: oracleWallet.publicKey,
});
```

### Create a Market with Category

```typescript
import { parseAmount, daysFromNow, MarketCategory } from '@fortuna-protocol/sdk';

await client.createMarket({
  marketId: 1,
  category: MarketCategory.Crypto,
  title: "Will BTC reach $100k by end of 2024?",
  description: "Bitcoin price prediction market. Resolves YES if BTC trades above $100,000.",
  betAmount: parseAmount("10", 6), // 10 USDC (6 decimals)
  bettingDeadline: daysFromNow(7),    // 7 days from now
  resolutionDeadline: daysFromNow(30), // 30 days from now
  outcomes: ["Yes", "No"],
  tokenMint: USDC_MINT,
  creatorFeeWallet: wallet.publicKey,
  oracleEventId: "BTC-100K-2024", // Optional: for oracle resolution
});
```

### Assign Oracle to Market

```typescript
// Assign an oracle to handle automated resolution
await client.assignOracle(marketId, oracleId);
```

### Place a Bet

```typescript
// Get market info
const market = await client.getMarket(1);
console.log(`Category: ${market.category}`);
console.log(`Bet amount: ${market.betAmount}`);
console.log(`Outcomes: ${market.outcomes.map(o => o.label)}`);

// Place a bet on outcome 0 (Yes)
await client.placeBet(1, 0);
```

### Resolve a Market

```typescript
// Manual resolution by creator
await client.resolveMarket(1, 0); // Resolve with outcome 0 (Yes) winning

// Or automated resolution by oracle
await client.oracleResolveMarket(1, 0); // Called by oracle authority
```

### Claim Winnings

```typescript
// Winners can claim their share of the pool
await client.claimWinnings(1);
```

### Withdraw Bet

```typescript
// Withdraw your bet before the betting deadline
// You get back your stake minus fees (fees are non-refundable)
await client.withdrawBet(1);
```

### Cancel & Refund

```typescript
// Creator can cancel an open market
await client.cancelMarket(1);

// Bettors can claim refunds on cancelled markets
await client.claimRefund(1);
```

## SDK Reference

### FortunaClient

```typescript
class FortunaClient {
  // Protocol Management
  initializeProtocol(config?: InitializeProtocolConfig): Promise<string>;
  getProtocolState(): Promise<ProtocolState | null>;
  updateProtocol(...): Promise<string>;

  // Oracle Management
  registerOracle(config: RegisterOracleConfig): Promise<string>;
  updateOracle(oracleId: number, config: UpdateOracleConfig): Promise<string>;
  getOracle(oracleId: number): Promise<Oracle | null>;
  getAllOracles(): Promise<Oracle[]>;

  // Market Management
  createMarket(config: CreateMarketConfig): Promise<string>;
  assignOracle(marketId: BN | number, oracleId: number): Promise<string>;
  getMarket(marketId: BN | number): Promise<Market | null>;
  getMarketStats(marketId: BN | number): Promise<MarketStats | null>;
  getAllMarkets(): Promise<Market[]>;
  getMarketsByStatus(status: MarketStatus): Promise<Market[]>;
  getMarketsByCategory(category: MarketCategory): Promise<Market[]>;

  // Betting
  placeBet(marketId: BN | number, outcomeIndex: number): Promise<string>;
  getBet(marketId: BN | number, bettor?: PublicKey): Promise<Bet | null>;
  getMarketBets(marketId: BN | number): Promise<Bet[]>;

  // Resolution & Claims
  resolveMarket(marketId: BN | number, winningOutcome: number): Promise<string>;
  oracleResolveMarket(marketId: BN | number, winningOutcome: number): Promise<string>;
  claimWinnings(marketId: BN | number): Promise<string>;
  cancelMarket(marketId: BN | number): Promise<string>;
  claimRefund(marketId: BN | number): Promise<string>;
  withdrawBet(marketId: BN | number): Promise<string>;

  // Utilities
  calculateBetFees(betAmount: BN | number): Promise<FeeBreakdown>;
  getMarketPDA(marketId: BN | number): PublicKey;
  getBetPDA(marketId: BN | number, bettor?: PublicKey): PublicKey;
  getOraclePDA(oracleId: number): PublicKey;
}
```

### Utility Functions

```typescript
// PDA derivation
getProtocolStatePDA(programId?): [PublicKey, number];
getMarketPDA(marketId, programId?): [PublicKey, number];
getMarketVaultPDA(marketPubkey, programId?): [PublicKey, number];
getPoolVaultPDA(marketPubkey, programId?): [PublicKey, number];
getBetPDA(marketPubkey, bettorPubkey, programId?): [PublicKey, number];
getOraclePDA(oracleId, programId?): [PublicKey, number];

// Fee calculation
calculateFees(amount, protocolFeeBps, creatorFeeBps, poolFeeBps): FeeBreakdown;
calculatePotentialWinnings(...): BN;

// Amount formatting
formatAmount(amount: BN, decimals?: number): string;
parseAmount(amount: string | number, decimals?: number): BN;

// Timestamps
hoursFromNow(hours: number): number;
daysFromNow(days: number): number;

// Categories
getCategoryName(category: MarketCategory): string;
getAllCategories(): MarketCategory[];
categoriesToBoolArray(categories: MarketCategory[]): boolean[];
boolArrayToCategories(boolArray: boolean[]): MarketCategory[];
```

## Program Instructions

| Instruction | Description | Authority |
|-------------|-------------|-----------|
| `initialize_protocol` | Initialize protocol settings | Protocol admin |
| `register_oracle` | Register a new oracle | Protocol admin |
| `update_oracle` | Update oracle settings | Protocol admin |
| `create_market` | Create a new prediction market | Anyone (or license holder) |
| `assign_oracle` | Assign oracle to a market | Market creator |
| `place_bet` | Place a bet on an outcome | Anyone |
| `withdraw_bet` | Withdraw bet before deadline (minus fees) | Bettor |
| `resolve_market` | Resolve market (manual) | Market creator |
| `oracle_resolve_market` | Resolve market (automated) | Oracle authority |
| `claim_winnings` | Claim winnings after resolution | Winning bettors |
| `cancel_market` | Cancel an open market | Market creator |
| `claim_refund` | Claim refund for cancelled market | Bettors |
| `update_protocol` | Update protocol settings | Protocol admin |
| `set_require_license` | Toggle license requirement | Protocol admin |
| `issue_license` | Issue a new license | Protocol admin |
| `revoke_license` | Revoke/deactivate a license | Protocol admin |
| `activate_license` | Activate a license | Protocol admin |
| `transfer_license` | Transfer license to new holder | License holder |
| `update_license` | Update license settings | Protocol admin |
| `add_authorized_wallet` | Add wallet to license | License holder |
| `remove_authorized_wallet` | Remove wallet from license | License holder |
| `add_authorized_domain` | Add domain to license | License holder |
| `remove_authorized_domain` | Remove domain from license | License holder |

## Account Structure

### ProtocolState
- Global protocol configuration
- Fee settings
- Treasury address
- Total oracles count
- Total licenses count
- License requirement flag

### Oracle
- Oracle identifier and name
- Authority (can submit results)
- Enabled categories (boolean array)
- Data source URL
- Resolution statistics

### Market
- Market metadata (title, description)
- Category (Politics, Sports, Crypto, etc.)
- Assigned oracle (optional)
- Oracle event ID (for automation)
- Fixed bet amount
- Betting/resolution deadlines
- Outcome definitions and totals
- Status (Open/Resolved/Cancelled)
- Resolution method tracking

### Bet
- Bettor address
- Selected outcome
- Original and net amounts
- Claim status

### License
- License key (32-byte hash)
- Holder wallet address
- License type (Basic, Pro, Enterprise, Custom)
- Feature flags
- Allowed domains (for domain locking)
- Allowed wallets (for wallet locking)
- Market limits and usage tracking
- Expiration and transferability settings

## License System

The license system enables domain/wallet locking and tiered access control for the protocol.

### License Types

| Type | Markets | Features |
|------|---------|----------|
| Basic | 5 | Create markets |
| Pro | 50 | Create markets, use oracles, private markets |
| Enterprise | Unlimited | All features including custom fees |
| Custom | Configurable | Admin-defined feature set |

### Domain/Wallet Locking

Licenses can restrict access to specific:
- **Domains**: e.g., `myapp.com`, `predictions.io`
- **Wallets**: Specific wallet addresses authorized to use the license

### Usage Examples

```typescript
import { generateLicenseKey, LicenseType } from '@fortuna/sdk';

// Issue a license
const licenseKey = generateLicenseKey('unique-license-string');
await client.issueLicense({
  licenseKey,
  licenseType: LicenseType.Pro,
  holder: userWallet,
  allowedDomains: ['myapp.com'],
  allowedWallets: [],
  maxMarkets: 50,
  isTransferable: true,
  expiresAt: 0, // Never expires
});

// Enable license requirement
await client.setRequireLicense(true);

// Add authorized wallet to license
await client.addAuthorizedWallet(licenseKey, anotherWallet);

// Add authorized domain
await client.addAuthorizedDomain(licenseKey, 'newdomain.com');

// Transfer license to new holder
await client.transferLicense(licenseKey, newHolderWallet);
```

## Oracle Integration

Oracles enable automated market resolution. Here's how the oracle system works:

1. **Registration**: Protocol admin registers oracles with specific category permissions
2. **Assignment**: Market creators assign oracles to their markets
3. **Data Source**: Oracles monitor external data sources (APIs, feeds)
4. **Resolution**: When conditions are met, oracle authority calls `oracle_resolve_market`
5. **Verification**: Market tracks whether resolution was manual or oracle-based

### Building an Oracle Service

```typescript
// Example oracle service structure
class SportsOracle {
  async checkAndResolve(market: Market) {
    // Fetch result from sports API using market.oracleEventId
    const result = await this.sportsApi.getResult(market.oracleEventId);

    if (result.isFinished) {
      const winningOutcome = result.homeWin ? 0 : 1;
      await client.oracleResolveMarket(market.marketId, winningOutcome);
    }
  }
}
```

## Security Considerations

1. **Fixed Bet Amounts**: Ensures fair participation regardless of capital
2. **One Bet Per Address**: Each address can only bet once per market
3. **Dual Resolution**: Markets can be resolved by creator OR assigned oracle
4. **Oracle Authorization**: Oracles can only resolve their assigned categories
5. **Deadline Enforcement**: Betting closes at deadline, resolution only after betting ends
6. **PDA Authority**: Token vaults are controlled by PDA, not individual accounts
7. **Withdrawal Window**: Bettors can withdraw before betting deadline; fees are non-refundable
8. **License Validation**: Optional license requirement for market creation
9. **Domain/Wallet Locking**: Restrict license usage to specific domains or wallets
10. **License Expiration**: Time-bound licenses with automatic expiration checks

## Testing

```bash
# Run all tests
anchor test

# Run specific test file
anchor test --skip-local-validator tests/fortuna-protocol.ts
```

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

MIT License - see [LICENSE](LICENSE) for details.

## Disclaimer

This software is provided "as is" without warranty. Use at your own risk. This is experimental software and should not be used for production betting platforms without thorough auditing and legal compliance review.
