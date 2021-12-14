## Implementation automated market maker(Uniswap) in a Solana on-chain program
Full description of the [task_description.pdf](./task_description.pdf)

***
### Diagram
[Accounts ownership diagrams](./acc_ownership.drawio)

***
_Topics:_
- Basics about Solana programming model
- Serialization and deserialization instruction_data and state in Rust using `borsh`
- Functional tests for on-chain Solana programs
- Program Derived Addresses
- Create accounts inside on-chain programs
- Invoke system instructions inside on-chain programs
- Cross-Program Invocations
- Privilege extension
- SPL Token
- SPL Associated Token Account Program
- Solana common security bugs
- Integer arithmetics in Rust
***

### Instruction
```rust
pub enum AmmInstruction {
    /// Initialization of an automated market maker.
    /// Creating and initializing PDA smart contract accounts.
    /// Saving the initial value of the contract tokens X, Y.
    /// X * Y = K
    ///
    /// Accounts expected by this instruction:
    /// 0. `[signer]` - user SPL token X owner
    /// 1. `[signer]` - user SPL token Y owner
    /// 2. `[signer, writable]` - user payer for creating PDA X, Y accounts
    /// 3. `[writable]` - from user SPL token X holder
    /// 4. `[writable]` - from user SPL token Y holder
    /// 5. `[]` - minter SPL token X
    /// 6. `[]` - minter SPL token Y
    /// 7. `[writable]` - contract(PDA) SPL token X holder
    /// 8. `[writable]` - contract(PDA) SPL token Y holder
    /// 9. `[]` - contract(PDA) SPL token X owner
    /// 10. `[]` - contract(PDA) SPL token Y owner
    /// 11. `[writable]` - contract(PDA) Vault
    /// 12. `[]` - Rent sysvar
    /// 13. `[]` - System program
    /// 14. `[]` - SPL Token program
    /// 15. `[]` - SPL associated token account program
    ///
    InitMarket { amount_x: u64, amount_y: u64 },

    /// Swap token with market.
    /// The user add token X(or Y) to contract.
    /// Contract return token Y(or X).
    /// dY = Y - K / (X + dX) / dX = X - K / (Y + dY)
    ///
    /// Accounts expected by this instruction:
    /// 0. `[signer]` - user SPL token owner
    /// 1. `[writable]` - from user SPL token X holder
    /// 2. `[writable]` - from user SPL token Y holder
    /// 3. `[]` - minter SPL token X
    /// 4. `[]` - minter SPL token Y
    /// 5. `[writable]` - contract(PDA) SPL token X holder
    /// 6. `[writable]` - contract(PDA) SPL token Y holder
    /// 7. `[]` - contract(PDA) SPL token X owner
    /// 8. `[]` - contract(PDA) SPL token Y owner
    /// 9. `[writable]` - contract(PDA) Vault
    /// 10. `[]` - SPL token program
    ///
    Swap {
        amount: u64,
        minter_pk: Pubkey,
    },
}
```

###State
```rust
/// Vault of balances of X, Y tokens of the market.
/// Unique for every different X, Y tokens. 
/// Needed because an attacker can add tokens in PDA of
/// a Solana on-chain program for violate the ratio X * Y = K
#[derive(BorshSerialize, BorshDeserialize, Debug, PartialEq)]
pub struct Vault {
    pub token_x_amount: u64,
    pub token_y_amount: u64
}
```

### Tests
Functional tests cover the main cases, the work of a smart contract.
Adding new tests is trivial, but covering all the cases will take too much time.

***
### Usage:
```
$ cargo build-bpf
$ cargo test-bpf
```

### Links:
- https://docs.solana.com/developing/programming-model/overview
- https://borsh.io
- https://docs.solana.com/developing/programming-model/calling-between-programs
- https://spl.solana.com/token
- https://spl.solana.com/associated-token-account
- https://docs.rs/spl-token/3.1.1/spl_token/index.html
- https://docs.rs/spl-associated-token-account/1.0.2/spl_associated_token_account/
- https://blog.neodyme.io/posts/solana_common_pitfalls
- https://husobee.github.io/money/float/2016/09/23/never-use-floats-for-currency.html
- https://medium.com/coinmonks/understanding-arithmetic-overflow-underflows-in-rust-and-solana-smart-contracts-9f3c9802dc45
- https://docs.rs/spl-math/latest/spl_math/