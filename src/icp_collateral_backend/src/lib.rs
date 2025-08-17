use ic_cdk::{caller, query, update};
use std::cell::RefCell;
use std::collections::HashMap;
use candid::{CandidType, Deserialize, Principal}; // FIX: Impor CandidType dan Deserialize

// Tipe alias untuk keterbacaan
type TokenAmount = u64;

// --- KONSTANTA UNTUK MODEL SUKU BUNGA ---
const BASE_RATE_BPS: u64 = 200;
const UTILIZATION_MULTIPLIER_BPS: u64 = 1500;
const COLLATERAL_FACTOR_PERCENT: u64 = 75;

// --- STRUKTUR DATA ---

#[derive(CandidType, Deserialize, Clone, Debug)] // FIX: Tambahkan CandidType dan Deserialize
struct Account {
    principal: Principal,
    collateral_balance: TokenAmount,
    debt_balance: TokenAmount,
}

impl Default for Account {
    fn default() -> Self {
        Account {
            principal: Principal::anonymous(),
            collateral_balance: 0,
            debt_balance: 0,
        }
    }
}

#[derive(CandidType, Deserialize, Default, Debug, Clone)] // FIX: Tambahkan CandidType dan Deserialize
struct Pool {
    total_liquidity: TokenAmount,
    total_borrowed: TokenAmount,
}

#[derive(Default)]
struct State {
    accounts: HashMap<Principal, Account>,
    pool: Pool,
}

// --- STATE MANAGEMENT ---

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State::default());
}

// === FUNGSI UTAMA (PUBLIC ENDPOINTS) ===

#[update]
fn deposit(amount: TokenAmount) {
    if amount == 0 {
        panic!("Jumlah setoran tidak boleh nol");
    }
    let caller = caller();
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        let account = state.accounts.entry(caller).or_insert_with(|| Account {
            principal: caller,
            ..Default::default()
        });

        account.collateral_balance += amount;
        state.pool.total_liquidity += amount;
    });
}

#[update]
fn withdraw(amount: TokenAmount) {
    if amount == 0 {
        panic!("Jumlah penarikan tidak boleh nol");
    }
    let caller = caller();
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        let account = state.accounts.get_mut(&caller).expect("Akun tidak ditemukan");

        if amount > account.collateral_balance {
            panic!("Saldo agunan tidak mencukupi");
        }

        let remaining_collateral = account.collateral_balance - amount;
        let max_borrow_power_after = (remaining_collateral * COLLATERAL_FACTOR_PERCENT) / 100;
        
        if account.debt_balance > max_borrow_power_after {
            panic!("Penarikan tidak diizinkan, akan menyebabkan likuidasi");
        }

        account.collateral_balance -= amount;
        state.pool.total_liquidity -= amount;
    });
}

#[update]
fn borrow(amount: TokenAmount) {
    if amount == 0 {
        panic!("Jumlah pinjaman tidak boleh nol");
    }
    let caller = caller();
    STATE.with(|state| {
        let mut state_ref = state.borrow_mut();
        
        let available_liquidity = state_ref.pool.total_liquidity - state_ref.pool.total_borrowed;
        if amount > available_liquidity {
            panic!("Likuiditas di pool tidak mencukupi");
        }
        
        let account = state_ref.accounts.get_mut(&caller).expect("Akun tidak ditemukan, setor agunan terlebih dahulu");

        let max_borrow_power = (account.collateral_balance * COLLATERAL_FACTOR_PERCENT) / 100;
        let new_debt = account.debt_balance + amount;
        
        if new_debt > max_borrow_power {
            panic!("Jumlah pinjaman melebihi batas agunan Anda");
        }

        account.debt_balance += amount;
        state_ref.pool.total_borrowed += amount;
    });
}


#[update]
fn repay(amount: TokenAmount) {
    if amount == 0 {
        panic!("Jumlah pengembalian tidak boleh nol");
    }
    let caller = caller();
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        let account = state.accounts.get_mut(&caller).expect("Akun tidak ditemukan");

        let repay_amount = amount.min(account.debt_balance);

        account.debt_balance -= repay_amount;
        state.pool.total_borrowed -= repay_amount;
    });
}


// === FUNGSI QUERY (READ-ONLY) ===

#[query]
fn get_account_info() -> Option<Account> {
    let caller = caller();
    STATE.with(|state| {
        state.borrow().accounts.get(&caller).cloned()
    })
}

#[query]
fn get_pool_info() -> (Pool, u64) {
    STATE.with(|state| {
        let state_ref = state.borrow();
        let pool_info = state_ref.pool.clone();
        
        let utilization_rate_bps = if state_ref.pool.total_liquidity == 0 {
            0
        } else {
            (state_ref.pool.total_borrowed * 10000) / state_ref.pool.total_liquidity
        };
        
        let interest_rate = BASE_RATE_BPS + (utilization_rate_bps * UTILIZATION_MULTIPLIER_BPS / 10000);
        
        (pool_info, interest_rate)
    })
}

// Candid export, pastikan ini ada di akhir file
ic_cdk::export_candid!();