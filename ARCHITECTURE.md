# System Architecture & Flow Diagrams

## 📐 Detailed System Architecture

### Component Interaction Diagram

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                           ICP Collateral Protocol                               │
│                                                                                 │
│  ┌─────────────┐    ┌──────────────────────────────────────┐    ┌─────────────┐ │
│  │   Users     │    │            Core Backend              │    │  Frontend   │ │
│  │             │    │         (Rust Canister)             │    │   (Vite)    │ │
│  │ ┌─────────┐ │    │                                      │    │             │ │
│  │ │Lenders  │◄┼────┼──┐  ┌─────────────────────────────┐  │    │ ┌─────────┐ │ │
│  │ └─────────┘ │    │  │  │      State Management       │  │    │ │   UI    │ │ │
│  │             │    │  │  │                             │  │    │ │Components│ │ │
│  │ ┌─────────┐ │    │  │  │ ┌─────────┬─────────────────┐ │  │    │ └─────────┘ │ │
│  │ │Borrowers│◄┼────┼──┼──┤ │Accounts │ Pools & Tokens  │ │  │    │             │ │
│  │ └─────────┘ │    │  │  │ └─────────┴─────────────────┘ │  │    │ ┌─────────┐ │ │
│  │             │    │  │  └─────────────────────────────┘  │    │ │Candid UI│ │ │
│  │ ┌─────────┐ │    │  │                                   │    │ └─────────┘ │ │
│  │ │Liquidat.│◄┼────┼──┼──┐ ┌─────────────────────────────┐ │    │             │ │
│  │ └─────────┘ │    │  │  │ │     Business Logic          │ │    └─────────────┘ │
│  └─────────────┘    │  │  │ │                             │ │                    │
│                     │  │  │ │ ┌──────────┬──────────────┐ │ │    ┌─────────────┐ │
│  ┌─────────────┐    │  │  │ │ │Lending   │Liquidation   │ │ │    │Mock Tokens  │ │
│  │             │    │  │  │ │ │Engine    │Engine        │ │ │    │             │ │
│  │ Mock Token  │    │  │  │ │ └──────────┴──────────────┘ │ │    │ ┌─────────┐ │ │
│  │ Canisters   │◄───┼──┼──┤ │                             │ │    │ │  USDC   │ │ │
│  │             │    │  │  │ │ ┌──────────┬──────────────┐ │ │    │ │ Canister│ │ │
│  │ ┌─────────┐ │    │  │  │ │ │Interest  │Token Lock    │ │ │    │ └─────────┘ │ │
│  │ │ Faucet  │ │    │  │  │ │ │Rate Model│Mechanism     │ │ │    │             │ │
│  │ │Functions│ │    │  │  │ │ └──────────┴──────────────┘ │ │    │ ┌─────────┐ │ │
│  │ └─────────┘ │    │  │  │ └─────────────────────────────┘ │    │ │  WETH   │ │ │
│  │             │    │  │  └─────────────────────────────────┘    │ │ Canister│ │ │
│  │ ┌─────────┐ │    │  │                                         │ └─────────┘ │ │
│  │ │ERC20-   │ │    │  │  ┌─────────────────────────────────┐    │             │ │
│  │ │like API │ │    │  └──┤       Query Functions          │    │ ┌─────────┐ │ │
│  │ └─────────┘ │    │     │                                 │    │ │  WBTC   │ │ │
│  └─────────────┘    │     │ ┌────────────┬────────────────┐ │    │ │ Canister│ │ │
│                     │     │ │Account Info│Pool Statistics │ │    │ └─────────┘ │ │
│                     │     │ └────────────┴────────────────┘ │    └─────────────┘ │
│                     │     │                                 │                    │
│                     │     │ ┌────────────┬────────────────┐ │                    │
│                     │     │ │Health      │Token Info      │ │                    │
│                     │     │ │Factor      │& Prices        │ │                    │
│                     │     │ └────────────┴────────────────┘ │                    │
│                     │     └─────────────────────────────────┘                    │
│                     └──────────────────────────────────────────────────────────────│
└─────────────────────────────────────────────────────────────────────────────────┘
```

## 🔄 Detailed Flow Diagrams

### 1. Complete Lending Cycle

```mermaid
graph TD
    subgraph "Initialization Phase"
        A[Deploy Canisters] --> B[Initialize Token Configs]
        B --> C[Set Price Oracles]
        C --> D[Configure Interest Rate Model]
    end
    
    subgraph "Liquidity Supply Phase"
        E[User Gets USDC] --> F[Supply to Pool]
        F --> G[Receive LP Tokens]
        G --> H[Earn Interest]
    end
    
    subgraph "Collateral & Borrowing Phase"
        I[User Gets WETH/WBTC] --> J[Deposit as Collateral]
        J --> K[Calculate Borrowing Power]
        K --> L[Borrow USDC]
        L --> M[Accrue Interest]
    end
    
    subgraph "Risk Management Phase"
        N[Monitor Health Factor] --> O{Health < 100%?}
        O -->|Yes| P[Liquidation Triggered]
        O -->|No| Q[Position Safe]
        P --> R[Liquidator Repays Debt]
        R --> S[Liquidator Gets Collateral + Bonus]
    end
    
    subgraph "Exit Phase"
        T[User Repays Debt] --> U[Withdraw Collateral]
        V[Liquidity Provider] --> W[Withdraw USDC + Interest]
    end
    
    D --> E
    D --> I
    H --> M
    M --> N
    Q --> T
    S --> T
    U --> V
```

### 2. Interest Rate Calculation Flow

```mermaid
graph TD
    A[Pool State Change] --> B[Calculate New Utilization]
    B --> C[Utilization = Borrowed/Liquidity]
    C --> D[Base Rate = 2%]
    D --> E[Variable Rate = Utilization × 15%]
    E --> F[New Rate = Base + Variable]
    F --> G[Update Pool State]
    G --> H[Compound Interest for Borrowers]
    H --> I[Distribute to Lenders]
    
    subgraph "Examples"
        J[20% Util → 5% APY]
        K[50% Util → 9.5% APY]
        L[90% Util → 15.5% APY]
    end
    
    F --> J
    F --> K
    F --> L
```

### 3. Health Factor Calculation & Liquidation

```mermaid
graph TD
    A[User Position] --> B[Get Collateral Values]
    B --> C[Apply Token Prices]
    C --> D[Calculate Total Collateral USD]
    D --> E[Apply Liquidation Thresholds]
    E --> F[Get Total Debt USD]
    F --> G[Health = Collateral×Threshold/Debt×100]
    
    G --> H{Health Factor}
    H -->|≥ 150%| I[Very Safe - Green]
    H -->|100-149%| J[Safe - Yellow] 
    H -->|80-99%| K[At Risk - Orange]
    H -->|< 80%| L[Liquidatable - Red]
    
    L --> M[Liquidation Process]
    M --> N[Calculate Max Repay = 50% of Debt]
    N --> O[Calculate Collateral to Seize]
    O --> P[Seize Amount = Repay × Price × 1.05]
    P --> Q[Transfer Collateral to Liquidator]
    Q --> R[Reduce User Debt]
    R --> S[Update Pool State]
```

### 4. Token Lock Mechanism

```mermaid
graph TD
    A[User Wants to Lock] --> B[Select Token & Amount]
    B --> C[Choose Lock Duration]
    C --> D{Duration Category}
    
    D -->|1-30 days| E[Bonus: 1% APY]
    D -->|31-90 days| F[Bonus: 2% APY]
    D -->|91-180 days| G[Bonus: 3% APY]
    D -->|181-365 days| H[Bonus: 5% APY]
    D -->|>365 days| I[Bonus: 10% APY]
    
    E --> J[Create Lock Position]
    F --> J
    G --> J
    H --> J
    I --> J
    
    J --> K[Set Unlock Timestamp]
    K --> L[Prevent Withdrawals]
    L --> M[Accrue Bonus Rewards]
    M --> N{Lock Expired?}
    N -->|No| M
    N -->|Yes| O[Allow Withdrawal]
    O --> P[Claim Bonus]
    P --> Q[Position Unlocked]
```

## 💾 Data Structure Diagrams

### State Management Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                       Global State                              │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────────────────────────────────────────────────────┐ │
│  │                   Accounts HashMap                          │ │
│  │                                                             │ │
│  │  Principal → Account {                                      │ │
│  │    ├─ principal: Principal                                  │ │
│  │    ├─ collateral_positions: HashMap<TokenType, Amount>      │ │
│  │    ├─ debt_positions: HashMap<TokenType, Amount>           │ │
│  │    ├─ locked_until: Option<Timestamp>                      │ │
│  │    └─ last_interest_update: Timestamp                      │ │
│  │  }                                                          │ │
│  └─────────────────────────────────────────────────────────────┘ │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────────┐ │
│  │                    Pools HashMap                            │ │
│  │                                                             │ │
│  │  TokenType → LiquidityPool {                               │ │
│  │    ├─ token_type: TokenType                                │ │
│  │    ├─ total_liquidity: Amount                              │ │
│  │    ├─ total_borrowed: Amount                               │ │
│  │    ├─ interest_rate_bps: u64                               │ │
│  │    ├─ utilization_rate_bps: u64                            │ │
│  │    ├─ last_update: Timestamp                               │ │
│  │    └─ cumulative_interest_index: Amount                    │ │
│  │  }                                                          │ │
│  └─────────────────────────────────────────────────────────────┘ │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────────┐ │
│  │                  Token Info HashMap                         │ │
│  │                                                             │ │
│  │  TokenType → TokenInfo {                                   │ │
│  │    ├─ symbol: String                                       │ │
│  │    ├─ decimals: u8                                         │ │
│  │    ├─ price_usd: Amount                                    │ │
│  │    ├─ is_collateral: bool                                  │ │
│  │    ├─ collateral_factor: u64                               │ │
│  │    ├─ liquidation_threshold: u64                           │ │
│  │    └─ liquidation_bonus: u64                               │ │
│  │  }                                                          │ │
│  └─────────────────────────────────────────────────────────────┘ │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────────┐ │
│  │                Lock Positions HashMap                       │ │
│  │                                                             │ │
│  │  Principal → Vec<LockInfo> {                               │ │
│  │    ├─ amount: Amount                                       │ │
│  │    ├─ token_type: TokenType                                │ │
│  │    ├─ lock_duration: u64                                   │ │
│  │    ├─ unlock_time: Timestamp                               │ │
│  │    └─ bonus_rate: u64                                      │ │
│  │  }                                                          │ │
│  └─────────────────────────────────────────────────────────────┘ │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────────┐ │
│  │                   Admin Controls                            │ │
│  │                                                             │ │
│  │  ├─ admin: Option<Principal>                               │ │
│  │  ├─ is_paused: bool                                        │ │
│  │  └─ total_supply: HashMap<TokenType, Amount>               │ │
│  └─────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

## 🔧 Function Call Flows

### Borrow Function Call Flow

```mermaid
sequenceDiagram
    participant User
    participant Frontend
    participant Backend
    participant TokenCanister
    
    User->>Frontend: Request Borrow
    Frontend->>Backend: call borrow(TokenType, Amount)
    
    Note over Backend: Validation Phase
    Backend->>Backend: Check if paused
    Backend->>Backend: Validate token type (USDC only)
    Backend->>Backend: Check pool liquidity
    Backend->>Backend: Get user account
    
    Note over Backend: Risk Assessment
    Backend->>Backend: Calculate borrowing power
    Backend->>Backend: Calculate current debt
    Backend->>Backend: Check collateral sufficiency
    Backend->>Backend: Check if account locked
    
    Note over Backend: State Updates
    Backend->>Backend: Update debt position
    Backend->>Backend: Update pool state
    Backend->>Backend: Update interest rates
    
    Backend->>Frontend: Return success/error
    Frontend->>User: Display result
```

### Liquidation Function Call Flow

```mermaid
sequenceDiagram
    participant Liquidator
    participant Backend
    participant BorrowerAccount
    participant Pool
    
    Liquidator->>Backend: call liquidate(user, collateral_token, debt_token, amount)
    
    Note over Backend: Validation
    Backend->>Backend: Check if not self-liquidation
    Backend->>Backend: Calculate health factor
    Backend->>Backend: Verify health factor < 100%
    
    Note over Backend: Liquidation Calculation
    Backend->>Backend: Get token prices
    Backend->>Backend: Calculate max repay amount
    Backend->>Backend: Calculate collateral to seize
    Backend->>Backend: Add liquidation bonus (5%)
    
    Note over Backend: State Updates
    Backend->>BorrowerAccount: Reduce debt position
    Backend->>BorrowerAccount: Reduce collateral position
    Backend->>Backend: Credit liquidator account
    Backend->>Pool: Update pool state
    
    Backend->>Liquidator: Return liquidation result
```

## 📊 Performance Metrics

### Key Performance Indicators

```
┌─────────────────────────────────────────────────────────────┐
│                     System Metrics                          │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Pool Utilization:     [████████░░] 80%                    │
│  Interest Rate:        12.0% APY                           │
│  Total Liquidity:      $2,500,000 USDC                     │
│  Total Borrowed:       $2,000,000 USDC                     │
│  Available Liquidity:  $500,000 USDC                       │
│                                                             │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │              Collateral Breakdown                       │ │
│  │                                                         │ │
│  │  WETH:  $1,800,000 (60%)  ██████████████████████████░░  │ │
│  │  WBTC:  $1,200,000 (40%)  ███████████████░░░░░░░░░░░░░░  │ │
│  │  Total: $3,000,000                                      │ │
│  └─────────────────────────────────────────────────────────┘ │
│                                                             │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │               Health Factor Distribution                 │ │
│  │                                                         │ │
│  │  Healthy (>150%):     120 positions ████████████████░░  │ │
│  │  Safe (100-150%):     45 positions  █████░░░░░░░░░░░░░░  │ │
│  │  At Risk (80-100%):   12 positions  ██░░░░░░░░░░░░░░░░░░  │ │
│  │  Liquidatable (<80%): 3 positions   ░░░░░░░░░░░░░░░░░░░░  │ │
│  └─────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

This comprehensive architecture documentation provides a complete visual understanding of the ICP Collateral Protocol system, including all component interactions, data flows, and state management structures.
