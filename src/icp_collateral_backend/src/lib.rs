use ic_cdk::{caller, query, update, init, pre_upgrade, post_upgrade};
use std::cell::RefCell;
use std::collections::HashMap;
use candid::{CandidType, Deserialize, Principal};
use serde::Serialize;
use ic_cdk_timers::{TimerId, set_timer_interval};
use std::time::Duration;

// === TYPE ALIASES ===
type TokenAmount = u128;
type Timestamp = u64;

// === CONSTANTS ===
const BASE_RATE_BPS: u64 = 200; // 2% base rate
const UTILIZATION_MULTIPLIER_BPS: u64 = 1500; // 15% at 100% utilization
const LIQUIDATION_THRESHOLD: u64 = 85; // 85% threshold for liquidation
const LIQUIDATION_BONUS: u64 = 5; // 5% bonus for liquidators
const INTEREST_COMPOUND_INTERVAL: u64 = 3600; // 1 hour in seconds
const SECONDS_PER_YEAR: u64 = 31536000; // 365 days * 24 hours * 3600 seconds

// === TOKEN DEFINITIONS ===
#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq, Eq, Hash, Copy)]
pub enum TokenType {
    USDC,
    WETH,
    WBTC,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct TokenInfo {
    pub symbol: String,
    pub decimals: u8,
    pub price_usd: TokenAmount, // Price in USD with 8 decimals
    pub is_collateral: bool,
    pub collateral_factor: u64, // Basis points (e.g., 7500 = 75%)
    pub liquidation_threshold: u64, // Basis points
    pub liquidation_bonus: u64, // Basis points
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct Position {
    pub token_type: TokenType,
    pub amount: TokenAmount,
    pub is_collateral: bool,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct Account {
    pub principal: Principal,
    pub collateral_positions: HashMap<TokenType, TokenAmount>,
    pub debt_positions: HashMap<TokenType, TokenAmount>,
    pub locked_until: Option<Timestamp>,
    pub last_interest_update: Timestamp,
}

impl Default for Account {
    fn default() -> Self {
        Account {
            principal: Principal::anonymous(),
            collateral_positions: HashMap::new(),
            debt_positions: HashMap::new(),
            locked_until: None,
            last_interest_update: ic_cdk::api::time(),
        }
    }
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct LiquidityPool {
    pub token_type: TokenType,
    pub total_liquidity: TokenAmount,
    pub total_borrowed: TokenAmount,
    pub interest_rate_bps: u64,
    pub utilization_rate_bps: u64,
    pub last_update: Timestamp,
    pub cumulative_interest_index: TokenAmount, // Scaled by 1e18
}

impl Default for LiquidityPool {
    fn default() -> Self {
        LiquidityPool {
            token_type: TokenType::USDC,
            total_liquidity: 0,
            total_borrowed: 0,
            interest_rate_bps: BASE_RATE_BPS,
            utilization_rate_bps: 0,
            last_update: ic_cdk::api::time(),
            cumulative_interest_index: 1_000_000_000_000_000_000, // 1e18
        }
    }
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct LockInfo {
    pub amount: TokenAmount,
    pub token_type: TokenType,
    pub lock_duration: u64, // in seconds
    pub unlock_time: Timestamp,
    pub bonus_rate: u64, // bonus rate in basis points
}

#[derive(Default)]
struct State {
    accounts: HashMap<Principal, Account>,
    pools: HashMap<TokenType, LiquidityPool>,
    token_info: HashMap<TokenType, TokenInfo>,
    lock_positions: HashMap<Principal, Vec<LockInfo>>,
    total_supply: HashMap<TokenType, TokenAmount>,
    admin: Option<Principal>,
    is_paused: bool,
}

// === STATE MANAGEMENT ===
thread_local! {
    static STATE: RefCell<State> = RefCell::new(State::default());
    static TIMER_ID: RefCell<Option<TimerId>> = RefCell::new(None);
}

// === INITIALIZATION ===
#[init]
fn init() {
    let caller = caller();
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        state.admin = Some(caller);
        
        // Initialize token information
        let mut tokens = HashMap::new();
        
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
        
        // WBTC - Collateral asset
        tokens.insert(TokenType::WBTC, TokenInfo {
            symbol: "WBTC".to_string(),
            decimals: 8,
            price_usd: 4500_000_000_000, // $45000 with 8 decimals
            is_collateral: true,
            collateral_factor: 7500, // 75%
            liquidation_threshold: 8000, // 80%
            liquidation_bonus: 500, // 5%
        });
        
        state.token_info = tokens;
        
        // Initialize pools
        let mut pools = HashMap::new();
        for token_type in [TokenType::USDC, TokenType::WETH, TokenType::WBTC] {
            pools.insert(token_type.clone(), LiquidityPool {
                token_type,
                ..Default::default()
            });
        }
        state.pools = pools;
    });
    
    // Start interest compounding timer
    start_interest_timer();
}

fn start_interest_timer() {
    let timer_id = set_timer_interval(Duration::from_secs(INTEREST_COMPOUND_INTERVAL), || {
        update_all_interest_rates();
    });
    
    TIMER_ID.with(|id| {
        *id.borrow_mut() = Some(timer_id);
    });
}

// === CORE FUNCTIONS ===

#[update]
fn supply_liquidity(token_type: TokenType, amount: TokenAmount) -> Result<String, String> {
    if amount == 0 {
        return Err("Amount cannot be zero".to_string());
    }
    
    let caller = caller();
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        
        if state.is_paused {
            return Err("Contract is paused".to_string());
        }
        
        // Only USDC can be supplied for lending
        if token_type != TokenType::USDC {
            return Err("Only USDC can be supplied for lending".to_string());
        }
        
        let pool = state.pools.get_mut(&token_type).ok_or("Pool not found")?;
        pool.total_liquidity += amount;
        
        // Update supplier balance (simplified - in production, use proper token accounting)
        let account = state.accounts.entry(caller).or_insert_with(|| Account {
            principal: caller,
            ..Default::default()
        });
        
        *account.collateral_positions.entry(token_type).or_insert(0) += amount;
        
        update_pool_interest_rate(&mut state, &token_type)?;
        
        Ok(format!("Successfully supplied {} USDC", amount))
    })
}

#[update]
fn deposit_collateral(token_type: TokenType, amount: TokenAmount) -> Result<String, String> {
    if amount == 0 {
        return Err("Amount cannot be zero".to_string());
    }
    
    let caller = caller();
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        
        if state.is_paused {
            return Err("Contract is paused".to_string());
        }
        
        let is_collateral = state.token_info.get(&token_type)
            .map(|info| info.is_collateral)
            .ok_or("Token not supported")?;
        
        if !is_collateral {
            return Err("Token cannot be used as collateral".to_string());
        }
        
        let symbol = state.token_info.get(&token_type).unwrap().symbol.clone();
        
        let account = state.accounts.entry(caller).or_insert_with(|| Account {
            principal: caller,
            ..Default::default()
        });
        
        *account.collateral_positions.entry(token_type).or_insert(0) += amount;
        
        Ok(format!("Successfully deposited {} {} as collateral", amount, symbol))
    })
}

#[update]
fn borrow(token_type: TokenType, amount: TokenAmount) -> Result<String, String> {
    if amount == 0 {
        return Err("Amount cannot be zero".to_string());
    }
    
    let caller = caller();
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        
        if state.is_paused {
            return Err("Contract is paused".to_string());
        }
        
        // Only USDC can be borrowed
        if token_type != TokenType::USDC {
            return Err("Only USDC can be borrowed".to_string());
        }
        
        // Check available liquidity first
        let (available_liquidity, current_debt) = {
            let pool = state.pools.get(&token_type)
                .ok_or("Pool not found")?;
            let available = pool.total_liquidity.saturating_sub(pool.total_borrowed);
            
            let _account = state.accounts.get(&caller)
                .ok_or("Account not found. Please deposit collateral first")?;
            let debt = calculate_total_debt(&state, &caller)?;
            
            (available, debt)
        };
        
        if amount > available_liquidity {
            return Err("Insufficient liquidity in pool".to_string());
        }
        
        // Calculate borrowing power
        let borrowing_power = calculate_borrowing_power(&state, &caller)?;
        let new_total_debt = current_debt + amount;
        
        if new_total_debt > borrowing_power {
            return Err(format!("Insufficient collateral. Max borrow: {}, Requested: {}", 
                             borrowing_power.saturating_sub(current_debt), amount));
        }
        
        // Check if account is locked
        {
            let account = state.accounts.get(&caller).unwrap();
            if let Some(unlock_time) = account.locked_until {
                if ic_cdk::api::time() < unlock_time {
                    return Err("Account is locked".to_string());
                }
            }
        }
        
        // Update account and pool
        let account = state.accounts.get_mut(&caller).unwrap();
        *account.debt_positions.entry(token_type).or_insert(0) += amount;
        
        let pool = state.pools.get_mut(&token_type).unwrap();
        pool.total_borrowed += amount;
        
        update_pool_interest_rate(&mut state, &token_type)?;
        
        Ok(format!("Successfully borrowed {} USDC", amount))
    })
}

#[update]
fn repay(token_type: TokenType, amount: TokenAmount) -> Result<String, String> {
    if amount == 0 {
        return Err("Amount cannot be zero".to_string());
    }
    
    let caller = caller();
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        
        if state.is_paused {
            return Err("Contract is paused".to_string());
        }
        
        let symbol = state.token_info.get(&token_type).unwrap().symbol.clone();
        
        let account = state.accounts.get_mut(&caller)
            .ok_or("Account not found")?;
        
        let current_debt = account.debt_positions.get(&token_type).unwrap_or(&0);
        let repay_amount = amount.min(*current_debt);
        
        if repay_amount == 0 {
            return Err("No debt to repay".to_string());
        }
        
        *account.debt_positions.entry(token_type).or_insert(0) -= repay_amount;
        
        let pool = state.pools.get_mut(&token_type)
            .ok_or("Pool not found")?;
        pool.total_borrowed = pool.total_borrowed.saturating_sub(repay_amount);
        
        update_pool_interest_rate(&mut state, &token_type)?;
        
        Ok(format!("Successfully repaid {} {}", repay_amount, symbol))
    })
}

#[update]
fn withdraw_collateral(token_type: TokenType, amount: TokenAmount) -> Result<String, String> {
    if amount == 0 {
        return Err("Amount cannot be zero".to_string());
    }
    
    let caller = caller();
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        
        if state.is_paused {
            return Err("Contract is paused".to_string());
        }
        
        let symbol = state.token_info.get(&token_type).unwrap().symbol.clone();
        
        // Check if account is locked and has sufficient balance
        {
            let account = state.accounts.get(&caller)
                .ok_or("Account not found")?;
            
            if let Some(unlock_time) = account.locked_until {
                if ic_cdk::api::time() < unlock_time {
                    return Err("Account is locked".to_string());
                }
            }
            
            let current_collateral = account.collateral_positions.get(&token_type).unwrap_or(&0);
            if amount > *current_collateral {
                return Err("Insufficient collateral balance".to_string());
            }
        }
        
        // Check health factor after withdrawal
        let original_amount = {
            let account = state.accounts.get(&caller).unwrap();
            *account.collateral_positions.get(&token_type).unwrap_or(&0)
        };
        
        // Temporarily modify the collateral amount to check health
        {
            let account = state.accounts.get_mut(&caller).unwrap();
            *account.collateral_positions.entry(token_type).or_insert(0) -= amount;
        }
        
        let health_factor = calculate_health_factor(&state, &caller)?;
        
        if health_factor < 100 { // Health factor below 1.0
            // Revert the change
            let account = state.accounts.get_mut(&caller).unwrap();
            *account.collateral_positions.entry(token_type).or_insert(0) = original_amount;
            return Err("Withdrawal would cause liquidation".to_string());
        }
        
        Ok(format!("Successfully withdrew {} {}", amount, symbol))
    })
}

#[update]
fn lock_tokens(token_type: TokenType, amount: TokenAmount, duration_days: u64) -> Result<String, String> {
    if amount == 0 || duration_days == 0 {
        return Err("Amount and duration must be greater than zero".to_string());
    }
    
    let caller = caller();
    let duration_seconds = duration_days * 24 * 3600;
    let current_time = ic_cdk::api::time();
    let unlock_time = current_time + duration_seconds * 1_000_000_000; // Convert to nanoseconds
    
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        
        if state.is_paused {
            return Err("Contract is paused".to_string());
        }
        
        let symbol = state.token_info.get(&token_type).unwrap().symbol.clone();
        
        let account = state.accounts.get_mut(&caller)
            .ok_or("Account not found")?;
        
        let current_balance = account.collateral_positions.get(&token_type).unwrap_or(&0);
        if amount > *current_balance {
            return Err("Insufficient balance to lock".to_string());
        }
        
        // Calculate bonus rate based on duration
        let bonus_rate = match duration_days {
            1..=30 => 100,    // 1% bonus
            31..=90 => 200,   // 2% bonus
            91..=180 => 300,  // 3% bonus
            181..=365 => 500, // 5% bonus
            _ => 1000,        // 10% bonus for >1 year
        };
        
        let lock_info = LockInfo {
            amount,
            token_type,
            lock_duration: duration_seconds,
            unlock_time,
            bonus_rate,
        };
        
        account.locked_until = Some(unlock_time);
        state.lock_positions.entry(caller).or_insert_with(Vec::new).push(lock_info);
        
        Ok(format!("Successfully locked {} {} for {} days with {}% bonus", 
                  amount, symbol, duration_days, bonus_rate as f64 / 100.0))
    })
}

#[update]
fn liquidate(user: Principal, collateral_token: TokenType, debt_token: TokenType, 
            repay_amount: TokenAmount) -> Result<String, String> {
    let caller = caller();
    
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        
        if state.is_paused {
            return Err("Contract is paused".to_string());
        }
        
        if caller == user {
            return Err("Cannot liquidate yourself".to_string());
        }
        
        let health_factor = calculate_health_factor(&state, &user)?;
        if health_factor >= 100 { // Health factor >= 1.0
            return Err("Position is healthy, cannot liquidate".to_string());
        }
        
        // Get token info first
        let debt_symbol = state.token_info.get(&debt_token).unwrap().symbol.clone();
        let collateral_symbol = state.token_info.get(&collateral_token).unwrap().symbol.clone();
        let debt_price = state.token_info.get(&debt_token).unwrap().price_usd;
        let debt_decimals = state.token_info.get(&debt_token).unwrap().decimals;
        let collateral_price = state.token_info.get(&collateral_token).unwrap().price_usd;
        let collateral_decimals = state.token_info.get(&collateral_token).unwrap().decimals;
        let liquidation_bonus = state.token_info.get(&collateral_token).unwrap().liquidation_bonus;
        
        let user_account = state.accounts.get_mut(&user)
            .ok_or("User account not found")?;
        
        let debt_balance = user_account.debt_positions.get(&debt_token).unwrap_or(&0);
        let actual_repay = repay_amount.min(*debt_balance);
        
        if actual_repay == 0 {
            return Err("No debt to liquidate".to_string());
        }
        
        // Calculate collateral to seize
        let debt_value_usd = (actual_repay * debt_price) / (10_u128.pow(debt_decimals as u32));
        let bonus_value_usd = (debt_value_usd * liquidation_bonus as u128) / 10000;
        let total_seize_value_usd = debt_value_usd + bonus_value_usd;
        
        let collateral_to_seize = (total_seize_value_usd * (10_u128.pow(collateral_decimals as u32))) / collateral_price;
        
        let collateral_balance = user_account.collateral_positions.get(&collateral_token).unwrap_or(&0);
        let actual_seize = collateral_to_seize.min(*collateral_balance);
        
        // Update user account
        *user_account.debt_positions.entry(debt_token).or_insert(0) -= actual_repay;
        *user_account.collateral_positions.entry(collateral_token).or_insert(0) -= actual_seize;
        
        // Update liquidator account
        let liquidator_account = state.accounts.entry(caller).or_insert_with(|| Account {
            principal: caller,
            ..Default::default()
        });
        *liquidator_account.collateral_positions.entry(collateral_token).or_insert(0) += actual_seize;
        
        // Update pool
        let pool = state.pools.get_mut(&debt_token).unwrap();
        pool.total_borrowed = pool.total_borrowed.saturating_sub(actual_repay);
        
        Ok(format!("Liquidation successful. Repaid {} {}, seized {} {}", 
                  actual_repay, debt_symbol, 
                  actual_seize, collateral_symbol))
    })
}

// === HELPER FUNCTIONS ===

fn calculate_borrowing_power(state: &State, user: &Principal) -> Result<TokenAmount, String> {
    let account = state.accounts.get(user).ok_or("Account not found")?;
    let mut total_collateral_value_usd = 0u128;
    
    for (token_type, amount) in &account.collateral_positions {
        let token_info = state.token_info.get(token_type).unwrap();
        if token_info.is_collateral {
            let value_usd = (amount * token_info.price_usd) / (10_u128.pow(token_info.decimals as u32));
            let collateral_value = (value_usd * token_info.collateral_factor as u128) / 10000;
            total_collateral_value_usd += collateral_value;
        }
    }
    
    // Convert back to USDC amount (6 decimals)
    let usdc_info = state.token_info.get(&TokenType::USDC).unwrap();
    let borrowing_power = (total_collateral_value_usd * (10_u128.pow(usdc_info.decimals as u32))) / usdc_info.price_usd;
    
    Ok(borrowing_power)
}

fn calculate_total_debt(state: &State, user: &Principal) -> Result<TokenAmount, String> {
    let account = state.accounts.get(user).ok_or("Account not found")?;
    let mut total_debt_usd = 0u128;
    
    for (token_type, amount) in &account.debt_positions {
        let token_info = state.token_info.get(token_type).unwrap();
        let debt_value_usd = (amount * token_info.price_usd) / (10_u128.pow(token_info.decimals as u32));
        total_debt_usd += debt_value_usd;
    }
    
    // Convert back to USDC amount
    let usdc_info = state.token_info.get(&TokenType::USDC).unwrap();
    let total_debt = (total_debt_usd * (10_u128.pow(usdc_info.decimals as u32))) / usdc_info.price_usd;
    
    Ok(total_debt)
}

fn calculate_health_factor(state: &State, user: &Principal) -> Result<u64, String> {
    let account = state.accounts.get(user).ok_or("Account not found")?;
    let mut total_collateral_value_usd = 0u128;
    let mut total_debt_value_usd = 0u128;
    
    // Calculate total collateral value with liquidation threshold
    for (token_type, amount) in &account.collateral_positions {
        let token_info = state.token_info.get(token_type).unwrap();
        if token_info.is_collateral {
            let value_usd = (amount * token_info.price_usd) / (10_u128.pow(token_info.decimals as u32));
            let threshold_value = (value_usd * token_info.liquidation_threshold as u128) / 10000;
            total_collateral_value_usd += threshold_value;
        }
    }
    
    // Calculate total debt value
    for (token_type, amount) in &account.debt_positions {
        let token_info = state.token_info.get(token_type).unwrap();
        let debt_value_usd = (amount * token_info.price_usd) / (10_u128.pow(token_info.decimals as u32));
        total_debt_value_usd += debt_value_usd;
    }
    
    if total_debt_value_usd == 0 {
        return Ok(u64::MAX); // No debt, infinite health factor
    }
    
    // Health factor = (collateral * liquidation_threshold) / debt
    // Multiply by 100 to get percentage
    let health_factor = (total_collateral_value_usd * 100) / total_debt_value_usd;
    Ok(health_factor as u64)
}

fn update_pool_interest_rate(state: &mut State, token_type: &TokenType) -> Result<(), String> {
    let pool = state.pools.get_mut(token_type).ok_or("Pool not found")?;
    
    let utilization_rate = if pool.total_liquidity == 0 {
        0
    } else {
        (pool.total_borrowed * 10000) / pool.total_liquidity
    };
    
    pool.utilization_rate_bps = utilization_rate as u64;
    pool.interest_rate_bps = BASE_RATE_BPS + (utilization_rate as u64 * UTILIZATION_MULTIPLIER_BPS / 10000);
    pool.last_update = ic_cdk::api::time();
    
    Ok(())
}

fn update_all_interest_rates() {
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        let token_types: Vec<TokenType> = state.pools.keys().cloned().collect();
        
        for token_type in token_types {
            let _ = update_pool_interest_rate(&mut state, &token_type);
        }
    });
}

// === QUERY FUNCTIONS ===

#[query]
fn get_account_info(user: Option<Principal>) -> Result<Account, String> {
    let target = user.unwrap_or_else(|| caller());
    STATE.with(|state| {
        state.borrow().accounts.get(&target)
            .cloned()
            .ok_or("Account not found".to_string())
    })
}

#[query]
fn get_pool_info(token_type: TokenType) -> Result<LiquidityPool, String> {
    STATE.with(|state| {
        state.borrow().pools.get(&token_type)
            .cloned()
            .ok_or("Pool not found".to_string())
    })
}

#[query]
fn get_all_pools() -> Vec<LiquidityPool> {
    STATE.with(|state| {
        state.borrow().pools.values().cloned().collect()
    })
}

#[query]
fn get_token_info(token_type: TokenType) -> Result<TokenInfo, String> {
    STATE.with(|state| {
        state.borrow().token_info.get(&token_type)
            .cloned()
            .ok_or("Token not found".to_string())
    })
}

#[query]
fn get_all_tokens() -> Vec<(TokenType, TokenInfo)> {
    STATE.with(|state| {
        state.borrow().token_info.iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    })
}

#[query]
fn get_user_health_factor(user: Option<Principal>) -> Result<u64, String> {
    let target = user.unwrap_or_else(|| caller());
    STATE.with(|state| {
        calculate_health_factor(&state.borrow(), &target)
    })
}

#[query]
fn get_borrowing_power(user: Option<Principal>) -> Result<TokenAmount, String> {
    let target = user.unwrap_or_else(|| caller());
    STATE.with(|state| {
        calculate_borrowing_power(&state.borrow(), &target)
    })
}

#[query]
fn get_lock_positions(user: Option<Principal>) -> Vec<LockInfo> {
    let target = user.unwrap_or_else(|| caller());
    STATE.with(|state| {
        state.borrow().lock_positions.get(&target)
            .cloned()
            .unwrap_or_default()
    })
}

// === ADMIN FUNCTIONS ===

#[update]
fn update_token_price(token_type: TokenType, new_price: TokenAmount) -> Result<String, String> {
    let caller = caller();
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        
        if Some(caller) != state.admin {
            return Err("Only admin can update prices".to_string());
        }
        
        let token_info = state.token_info.get_mut(&token_type)
            .ok_or("Token not found")?;
        
        token_info.price_usd = new_price;
        
        Ok(format!("Updated {} price to ${}", token_info.symbol, new_price as f64 / 1e8))
    })
}

#[update]
fn pause_contract() -> Result<String, String> {
    let caller = caller();
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        
        if Some(caller) != state.admin {
            return Err("Only admin can pause contract".to_string());
        }
        
        state.is_paused = true;
        Ok("Contract paused".to_string())
    })
}

#[update]
fn unpause_contract() -> Result<String, String> {
    let caller = caller();
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        
        if Some(caller) != state.admin {
            return Err("Only admin can unpause contract".to_string());
        }
        
        state.is_paused = false;
        Ok("Contract unpaused".to_string())
    })
}

// === UPGRADE FUNCTIONS ===

#[pre_upgrade]
fn pre_upgrade() {
    // In a production system, you would serialize and store the state
}

#[post_upgrade]
fn post_upgrade() {
    // In a production system, you would deserialize and restore the state
    start_interest_timer();
}

// Candid export
ic_cdk::export_candid!();