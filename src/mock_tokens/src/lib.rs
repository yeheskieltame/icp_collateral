use ic_cdk::{caller, query, update, init};
use std::cell::RefCell;
use std::collections::HashMap;
use candid::{CandidType, Deserialize, Principal};
use serde::Serialize;

// === TYPE DEFINITIONS ===
type TokenAmount = u128;

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct TokenMetadata {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub total_supply: TokenAmount,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct TransferArgs {
    pub to: Principal,
    pub amount: TokenAmount,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct TransferFromArgs {
    pub from: Principal,
    pub to: Principal,
    pub amount: TokenAmount,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct ApproveArgs {
    pub spender: Principal,
    pub amount: TokenAmount,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum TransferError {
    InsufficientBalance,
    InsufficientAllowance,
    Unauthorized,
}

type TransferResult = Result<TokenAmount, TransferError>;

// === STATE MANAGEMENT ===
#[derive(Default)]
struct State {
    metadata: Option<TokenMetadata>,
    balances: HashMap<Principal, TokenAmount>,
    allowances: HashMap<Principal, HashMap<Principal, TokenAmount>>,
    owner: Option<Principal>,
}

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State::default());
}

// === INITIALIZATION ===
#[init]
fn init(name: String, symbol: String, decimals: u8, initial_supply: TokenAmount) {
    let caller = caller();
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        
        state.metadata = Some(TokenMetadata {
            name,
            symbol,
            decimals,
            total_supply: initial_supply,
        });
        
        state.owner = Some(caller);
        state.balances.insert(caller, initial_supply);
    });
}

// === PUBLIC FUNCTIONS ===

#[query]
fn name() -> String {
    STATE.with(|state| {
        state.borrow().metadata.as_ref()
            .map(|m| m.name.clone())
            .unwrap_or_default()
    })
}

#[query]
fn symbol() -> String {
    STATE.with(|state| {
        state.borrow().metadata.as_ref()
            .map(|m| m.symbol.clone())
            .unwrap_or_default()
    })
}

#[query]
fn decimals() -> u8 {
    STATE.with(|state| {
        state.borrow().metadata.as_ref()
            .map(|m| m.decimals)
            .unwrap_or_default()
    })
}

#[query]
fn total_supply() -> TokenAmount {
    STATE.with(|state| {
        state.borrow().metadata.as_ref()
            .map(|m| m.total_supply)
            .unwrap_or_default()
    })
}

#[query]
fn balance_of(account: Principal) -> TokenAmount {
    STATE.with(|state| {
        *state.borrow().balances.get(&account).unwrap_or(&0)
    })
}

#[query]
fn allowance(owner: Principal, spender: Principal) -> TokenAmount {
    STATE.with(|state| {
        state.borrow()
            .allowances
            .get(&owner)
            .and_then(|allowances| allowances.get(&spender))
            .copied()
            .unwrap_or(0)
    })
}

#[update]
fn transfer(args: TransferArgs) -> TransferResult {
    let caller = caller();
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        
        let from_balance = *state.balances.get(&caller).unwrap_or(&0);
        if from_balance < args.amount {
            return Err(TransferError::InsufficientBalance);
        }
        
        // Update balances
        state.balances.insert(caller, from_balance - args.amount);
        let to_balance = *state.balances.get(&args.to).unwrap_or(&0);
        state.balances.insert(args.to, to_balance + args.amount);
        
        Ok(args.amount)
    })
}

#[update]
fn transfer_from(args: TransferFromArgs) -> TransferResult {
    let caller = caller();
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        
        // Check allowance
        let allowed = state.allowances
            .get(&args.from)
            .and_then(|allowances| allowances.get(&caller))
            .copied()
            .unwrap_or(0);
        
        if allowed < args.amount {
            return Err(TransferError::InsufficientAllowance);
        }
        
        // Check balance
        let from_balance = *state.balances.get(&args.from).unwrap_or(&0);
        if from_balance < args.amount {
            return Err(TransferError::InsufficientBalance);
        }
        
        // Update balances
        state.balances.insert(args.from, from_balance - args.amount);
        let to_balance = *state.balances.get(&args.to).unwrap_or(&0);
        state.balances.insert(args.to, to_balance + args.amount);
        
        // Update allowance
        state.allowances
            .entry(args.from)
            .or_insert_with(HashMap::new)
            .insert(caller, allowed - args.amount);
        
        Ok(args.amount)
    })
}

#[update]
fn approve(args: ApproveArgs) -> TransferResult {
    let caller = caller();
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        
        state.allowances
            .entry(caller)
            .or_insert_with(HashMap::new)
            .insert(args.spender, args.amount);
        
        Ok(args.amount)
    })
}

// === MINT FUNCTION FOR TESTING ===
#[update]
fn mint(to: Principal, amount: TokenAmount) -> TransferResult {
    let caller = caller();
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        
        // Only owner can mint (or anyone for testing purposes)
        // In production, you might want to restrict this
        
        let current_balance = *state.balances.get(&to).unwrap_or(&0);
        state.balances.insert(to, current_balance + amount);
        
        // Update total supply
        if let Some(ref mut metadata) = state.metadata {
            metadata.total_supply += amount;
        }
        
        Ok(amount)
    })
}

// === FAUCET FUNCTION FOR TESTING ===
#[update]
fn faucet() -> TransferResult {
    let caller = caller();
    let faucet_amount = 1000 * 10_u128.pow(decimals() as u32); // 1000 tokens
    
    mint(caller, faucet_amount)
}

// === QUERY FUNCTIONS ===
#[query]
fn get_metadata() -> Option<TokenMetadata> {
    STATE.with(|state| {
        state.borrow().metadata.clone()
    })
}

#[query]
fn get_all_balances() -> Vec<(Principal, TokenAmount)> {
    STATE.with(|state| {
        state.borrow().balances.iter()
            .map(|(k, v)| (*k, *v))
            .collect()
    })
}

// Candid export
ic_cdk::export_candid!();
