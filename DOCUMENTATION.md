# ICP Collateral System - Complete Documentation

## Overview
Sistem collateral lending/borrowing yang berjalan di Internet Computer Protocol (ICP) dengan fitur:
- Multi-token collateral support (WETH, WBTC)
- Dynamic interest rates
- Token locking with bonus rewards
- Liquidation mechanism
- Mock tokens untuk testing

## Architecture

### Smart Contracts
1. **icp_collateral_backend** - Main lending protocol
2. **usdc_token** - Mock USDC token (6 decimals)
3. **weth_token** - Mock WETH token (18 decimals)  
4. **wbtc_token** - Mock WBTC token (8 decimals)

### Token Configuration
| Token | Decimals | Price (USD) | Collateral Factor | Liquidation Threshold | Can Borrow | Can Collateralize |
|-------|----------|-------------|-------------------|----------------------|------------|-------------------|
| USDC  | 6        | $1          | 0%                | 0%                   | ✅         | ❌                |
| WETH  | 18       | $3,000      | 80%               | 85%                  | ❌         | ✅                |
| WBTC  | 8        | $45,000     | 75%               | 80%                  | ❌         | ✅                |

## Core Functions

### For Lenders (USDC Suppliers)
```bash
# Supply USDC to earn interest
dfx canister call icp_collateral_backend supply_liquidity '(variant { USDC }, AMOUNT)'
```

### For Borrowers
```bash
# 1. Deposit collateral (WETH/WBTC)
dfx canister call icp_collateral_backend deposit_collateral '(variant { WETH }, AMOUNT)'

# 2. Check borrowing power
dfx canister call icp_collateral_backend get_borrowing_power '(null)'

# 3. Borrow USDC
dfx canister call icp_collateral_backend borrow '(variant { USDC }, AMOUNT)'

# 4. Repay debt
dfx canister call icp_collateral_backend repay '(variant { USDC }, AMOUNT)'

# 5. Withdraw collateral (if health factor allows)
dfx canister call icp_collateral_backend withdraw_collateral '(variant { WETH }, AMOUNT)'
```

### Token Locking (Earn Bonus)
```bash
# Lock tokens for rewards
dfx canister call icp_collateral_backend lock_tokens '(variant { WETH }, AMOUNT, DAYS)'

# Check lock positions
dfx canister call icp_collateral_backend get_lock_positions '(null)'
```

### Lock Bonus Rates
- 1-30 days: 1% bonus
- 31-90 days: 2% bonus  
- 91-180 days: 3% bonus
- 181-365 days: 5% bonus
- >365 days: 10% bonus

## Mock Token Functions

### Get Free Tokens (Testing)
```bash
# Claim 1000 tokens from faucet
dfx canister call usdc_token faucet
dfx canister call weth_token faucet  
dfx canister call wbtc_token faucet
```

### Check Balances
```bash
USER_PRINCIPAL=$(dfx identity get-principal)
dfx canister call usdc_token balance_of "(principal \"$USER_PRINCIPAL\")"
```

## Query Functions

### Account Information
```bash
# Get your account details
dfx canister call icp_collateral_backend get_account_info '(null)'

# Check health factor (liquidation risk)
dfx canister call icp_collateral_backend get_user_health_factor '(null)'

# Check borrowing power
dfx canister call icp_collateral_backend get_borrowing_power '(null)'
```

### Pool Information
```bash
# Get specific pool info
dfx canister call icp_collateral_backend get_pool_info '(variant { USDC })'

# Get all pools
dfx canister call icp_collateral_backend get_all_pools

# Get token configuration
dfx canister call icp_collateral_backend get_token_info '(variant { WETH })'
```

## Interest Rate Model

**Base Rate**: 2% APY  
**Utilization Multiplier**: 15%  
**Formula**: Interest Rate = Base Rate + (Utilization Rate × Multiplier)

Example:
- 40% utilization = 2% + (40% × 15%) = 8% APY
- 80% utilization = 2% + (80% × 15%) = 14% APY

## Health Factor & Liquidation

**Health Factor** = (Collateral Value × Liquidation Threshold) ÷ Debt Value × 100

- **Healthy**: Health Factor ≥ 100%
- **At Risk**: Health Factor < 100% → Can be liquidated
- **Liquidation Bonus**: 5% for liquidators

## Example Workflow

### 1. Setup (Get Test Tokens)
```bash
# Get free tokens
dfx canister call usdc_token faucet    # Get 1000 USDC
dfx canister call weth_token faucet    # Get 1000 WETH  
dfx canister call wbtc_token faucet    # Get 1000 WBTC
```

### 2. Supply Liquidity (Earn Interest)
```bash
# Supply 2000 USDC to pool
dfx canister call icp_collateral_backend supply_liquidity '(variant { USDC }, 2000000000)'
```

### 3. Borrow Against Collateral
```bash
# Deposit 1 WETH as collateral ($3000 value)
dfx canister call icp_collateral_backend deposit_collateral '(variant { WETH }, 1000000000000000000)'

# Check borrowing power (should be ~$2400 = 80% of $3000)
dfx canister call icp_collateral_backend get_borrowing_power '(null)'

# Borrow 1000 USDC
dfx canister call icp_collateral_backend borrow '(variant { USDC }, 1000000000)'

# Check health factor
dfx canister call icp_collateral_backend get_user_health_factor '(null)'
```

### 4. Lock Tokens for Bonus
```bash
# Lock 0.5 WETH for 30 days (1% bonus)
dfx canister call icp_collateral_backend lock_tokens '(variant { WETH }, 500000000000000000, 30)'
```

## Canister IDs (Local)
Run this to get your local canister IDs:
```bash
dfx canister id icp_collateral_backend
dfx canister id usdc_token
dfx canister id weth_token  
dfx canister id wbtc_token
```

## Testing Script
Use the provided test script for comprehensive testing:
```bash
chmod +x test_script.sh
./test_script.sh
```

## Security Features
- Health factor monitoring
- Liquidation protection
- Token locking mechanism
- Admin controls (pause/unpause)
- Interest rate caps

## Future Enhancements
- Oracle price feeds
- More collateral assets
- Governance token
- Flash loans
- Cross-chain bridges
- Advanced liquidation strategies
