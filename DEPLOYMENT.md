# 🚀 Deployment Guide - ICP Collateral Protocol

## 📋 Prerequisites

### System Requirements
- **Operating System**: macOS, Linux, or Windows (WSL2)
- **Node.js**: Version 16+ 
- **Rust**: Latest stable version
- **Git**: For cloning the repository

### Install Internet Computer SDK

```bash
# Install dfx (IC SDK)
sh -ci "$(curl -fsSL https://internetcomputer.org/install.sh)"

# Verify installation
dfx --version
```

### Install Rust and Cargo
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add WebAssembly target
rustup target add wasm32-unknown-unknown

# Verify installation
cargo --version
rustc --version
```

## 📂 Project Setup

### 1. Clone Repository
```bash
git clone <repository-url>
cd icp_collateral
```

### 2. Project Structure Overview
```
icp_collateral/
├── src/
│   ├── icp_collateral_backend/    # Main lending protocol
│   │   ├── src/lib.rs
│   │   ├── Cargo.toml
│   │   └── icp_collateral_backend.did
│   ├── mock_tokens/               # Mock token contracts
│   │   ├── src/lib.rs
│   │   ├── Cargo.toml
│   │   └── mock_tokens.did
│   └── icp_collateral_frontend/   # Frontend UI
│       ├── src/
│       ├── package.json
│       └── vite.config.js
├── dfx.json                       # Canister configuration
├── Cargo.toml                     # Workspace configuration
└── README.md
```

## 🛠️ Local Development Deployment

### 1. Start Local IC Replica
```bash
# Start IC replica in background
dfx start --background

# Check status
dfx ping
```

### 2. Deploy All Canisters
```bash
# Deploy with sufficient cycles
dfx deploy --with-cycles 1000000000000

# Or deploy individually
dfx deploy usdc_token
dfx deploy weth_token
dfx deploy wbtc_token
dfx deploy icp_collateral_backend
dfx deploy icp_collateral_frontend
```

### 3. Verify Deployment
```bash
# Get canister IDs
dfx canister id icp_collateral_backend
dfx canister id usdc_token
dfx canister id weth_token
dfx canister id wbtc_token

# Check canister status
dfx canister status icp_collateral_backend
```

## 🧪 Testing Deployment

### 1. Get Test Tokens
```bash
# Claim free tokens from faucets
dfx canister call usdc_token faucet
dfx canister call weth_token faucet
dfx canister call wbtc_token faucet
```

### 2. Test Core Functions
```bash
# Supply liquidity
dfx canister call icp_collateral_backend supply_liquidity '(variant { USDC }, 1000000000)'

# Deposit collateral
dfx canister call icp_collateral_backend deposit_collateral '(variant { WETH }, 1000000000000000000)'

# Check borrowing power
dfx canister call icp_collateral_backend get_borrowing_power '(null)'

# Borrow USDC
dfx canister call icp_collateral_backend borrow '(variant { USDC }, 500000000)'
```

### 3. Run Test Suite
```bash
# Make script executable
chmod +x test_script.sh

# Run comprehensive tests
./test_script.sh
```

## 🌐 IC Mainnet Deployment

### 1. Prepare for Mainnet

#### Check Identity and Cycles
```bash
# Create new identity for mainnet
dfx identity new mainnet
dfx identity use mainnet

# Check principal
dfx identity get-principal

# Check cycles balance
dfx wallet --network ic balance
```

#### Update Configuration for Mainnet
```bash
# Update dfx.json for mainnet deployment
# Remove init_arg for real token integration
# Update frontend dependencies
```

### 2. Deploy to Mainnet
```bash
# Deploy backend first
dfx deploy --network ic icp_collateral_backend --with-cycles 2000000000000

# Deploy mock tokens (if needed for testing)
dfx deploy --network ic usdc_token --with-cycles 1000000000000
dfx deploy --network ic weth_token --with-cycles 1000000000000
dfx deploy --network ic wbtc_token --with-cycles 1000000000000

# Deploy frontend
dfx deploy --network ic icp_collateral_frontend
```

### 3. Verify Mainnet Deployment
```bash
# Get mainnet canister IDs
dfx canister --network ic id icp_collateral_backend

# Check status
dfx canister --network ic status icp_collateral_backend

# Test basic functionality
dfx canister --network ic call icp_collateral_backend get_all_tokens
```

## 🔧 Configuration

### Environment Variables
Create `.env` file:
```env
# Network configuration
DFX_NETWORK=local
CANISTER_ID_ICP_COLLATERAL_BACKEND=uxrrr-q7777-77774-qaaaq-cai
CANISTER_ID_USDC_TOKEN=umunu-kh777-77774-qaaca-cai
CANISTER_ID_WETH_TOKEN=ucwa4-rx777-77774-qaada-cai
CANISTER_ID_WBTC_TOKEN=ulvla-h7777-77774-qaacq-cai
```

### Token Configuration
Update token prices and parameters in `lib.rs`:
```rust
// USDC - Main lending asset
tokens.insert(TokenType::USDC, TokenInfo {
    symbol: "USDC".to_string(),
    decimals: 6,
    price_usd: 100_000_000, // $1 with 8 decimals
    is_collateral: false,
    collateral_factor: 0,
    liquidation_threshold: 0,
    liquidation_bonus: 0,
});

// WETH - Collateral asset
tokens.insert(TokenType::WETH, TokenInfo {
    symbol: "WETH".to_string(),
    decimals: 18,
    price_usd: 300_000_000_000, // $3000 with 8 decimals
    is_collateral: true,
    collateral_factor: 8000, // 80%
    liquidation_threshold: 8500, // 85%
    liquidation_bonus: 500, // 5%
});
```

## 🔄 Upgrade Process

### 1. Prepare Upgrade
```bash
# Make changes to code
# Update version in Cargo.toml
# Test locally first
```

### 2. Upgrade Canisters
```bash
# Upgrade backend
dfx deploy icp_collateral_backend --mode upgrade

# Upgrade specific canister on mainnet
dfx deploy --network ic icp_collateral_backend --mode upgrade
```

### 3. Verify Upgrade
```bash
# Check version or state
dfx canister call icp_collateral_backend get_all_pools

# Verify no data loss
dfx canister call icp_collateral_backend get_account_info '(null)'
```

## 📊 Monitoring & Maintenance

### Health Checks
```bash
# Check canister status
dfx canister status icp_collateral_backend

# Monitor cycles
dfx canister --network ic status icp_collateral_backend

# Check error logs
dfx canister logs icp_collateral_backend
```

### Regular Maintenance
```bash
# Top up cycles if needed
dfx canister --network ic deposit-cycles 1000000000000 icp_collateral_backend

# Update token prices (admin function)
dfx canister call icp_collateral_backend update_token_price '(variant { WETH }, 350000000000)'
```

## 🚨 Emergency Procedures

### Pause Contract
```bash
# Pause all operations (admin only)
dfx canister call icp_collateral_backend pause_contract

# Unpause when ready
dfx canister call icp_collateral_backend unpause_contract
```

### Backup State
```bash
# Export critical data
dfx canister call icp_collateral_backend get_all_pools > pools_backup.json
dfx canister call icp_collateral_backend get_all_tokens > tokens_backup.json
```

## 🔐 Security Considerations

### Access Control
- Only admin can update token prices
- Only admin can pause/unpause contract
- Verify admin principal before deployment

### Code Verification
- Test all functions thoroughly
- Run security audits
- Verify arithmetic operations
- Check for integer overflow/underflow

### Monitoring
- Set up alerts for unusual activity
- Monitor health factors
- Track liquidation events
- Monitor cycle balance

## 🌍 Frontend Deployment

### Local Development
```bash
cd src/icp_collateral_frontend
npm install
npm start
```

### Production Build
```bash
npm run build
dfx deploy icp_collateral_frontend
```

### Access URLs
- **Local**: `http://<frontend-canister-id>.localhost:4943/`
- **Mainnet**: `https://<frontend-canister-id>.ic0.app/`

## 📝 Post-Deployment Checklist

- [ ] All canisters deployed successfully
- [ ] Test token faucets working
- [ ] Core lending functions operational
- [ ] Interest rate calculations correct
- [ ] Health factor monitoring active
- [ ] Liquidation mechanism tested
- [ ] Frontend connecting to backend
- [ ] Admin functions accessible
- [ ] Monitoring systems in place
- [ ] Emergency procedures documented
- [ ] Backup procedures established

## 🐛 Troubleshooting

### Common Issues

1. **Deployment Fails**
   ```bash
   # Check cycles balance
   dfx wallet balance
   
   # Check canister status
   dfx canister status <canister-id>
   ```

2. **Function Calls Fail**
   ```bash
   # Check canister logs
   dfx canister logs <canister-id>
   
   # Verify argument format
   dfx canister call <canister-id> <function> --help
   ```

3. **Frontend Not Loading**
   ```bash
   # Rebuild and redeploy
   npm run build
   dfx deploy icp_collateral_frontend
   ```

### Getting Help
- [IC Developer Discord](https://discord.gg/jnjVVQe)
- [IC Developer Forum](https://forum.dfinity.org/)
- [IC Documentation](https://internetcomputer.org/docs/)

---

**Successfully deployed ICP Collateral Protocol! 🎉**
