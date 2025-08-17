#!/bin/bash

# ICP Collateral Testing Script
echo "=== ICP Collateral Smart Contract Testing ==="
echo

# Get user principal
USER_PRINCIPAL=$(dfx identity get-principal)
echo "User Principal: $USER_PRINCIPAL"
echo

# Get canister IDs
echo "=== Canister IDs ==="
USDC_CANISTER=$(dfx canister id usdc_token)
WETH_CANISTER=$(dfx canister id weth_token)
WBTC_CANISTER=$(dfx canister id wbtc_token)
BACKEND_CANISTER=$(dfx canister id icp_collateral_backend)

echo "USDC Token: $USDC_CANISTER"
echo "WETH Token: $WETH_CANISTER"
echo "WBTC Token: $WBTC_CANISTER"
echo "Collateral Backend: $BACKEND_CANISTER"
echo

# Test 1: Check initial balances
echo "=== Test 1: Token Balances ==="
echo "USDC Balance:"
dfx canister call usdc_token balance_of "(principal \"$USER_PRINCIPAL\")"
echo "WETH Balance:"
dfx canister call weth_token balance_of "(principal \"$USER_PRINCIPAL\")"
echo "WBTC Balance:"
dfx canister call wbtc_token balance_of "(principal \"$USER_PRINCIPAL\")"
echo

# Test 2: Get token info
echo "=== Test 2: Token Information ==="
echo "USDC Info:"
dfx canister call icp_collateral_backend get_token_info '(variant { USDC })'
echo "WETH Info:"
dfx canister call icp_collateral_backend get_token_info '(variant { WETH })'
echo "WBTC Info:"
dfx canister call icp_collateral_backend get_token_info '(variant { WBTC })'
echo

# Test 3: Current account info
echo "=== Test 3: Account Information ==="
dfx canister call icp_collateral_backend get_account_info '(null)'
echo

# Test 4: Pool information
echo "=== Test 4: Pool Information ==="
dfx canister call icp_collateral_backend get_all_pools
echo

# Test 5: Health factor and borrowing power
echo "=== Test 5: Health Factor & Borrowing Power ==="
echo "Health Factor:"
dfx canister call icp_collateral_backend get_user_health_factor '(null)'
echo "Borrowing Power:"
dfx canister call icp_collateral_backend get_borrowing_power '(null)'
echo

# Test 6: Lock positions
echo "=== Test 6: Lock Positions ==="
dfx canister call icp_collateral_backend get_lock_positions '(null)'
echo

echo "=== Testing Complete ==="
