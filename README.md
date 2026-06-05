# solana-compliance-hook
Compliance Hook demo, compliant payment processor on Solana now needs Transfer Hook logic for KYC allow lists, blacklists, transfer fees, and compliance gating

### repo architecture 
```
solana-compliance-hook/
│
├── programs/
│   └── solana-compliance-hook/       # The only program – the Transfer Hook
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs                # Main program – will be refactored for 3 modes
│           ├── instructions/
│           │   ├── mod.rs
│           │   ├── set_mode.rs       # NEW – instruction to set mode + config
│           │   └── execute.rs        # TRANSFORMED – execute logic branches by mode
│           ├── state/
│           │   ├── mod.rs
│           │   └── compliance_config.rs  # NEW – config account with mode enum
│           └── error.rs
│
├── libraries/
│   └── extra-account-meta-list/      # Keep ONLY if this is a custom helper crate
│       ├── Cargo.toml                # Otherwise delete; we'll use spl-transfer-hook crate
│       └── src/lib.rs                # Keep for extra metas resolution utilities
│
├── tests/
│   ├── compliance.test.ts            # Anchor test suite covering all branches
│   ├── utils.ts                      # Helpers: create mint, set up hook, fund accounts
│   └── bankrun.ts                    # (optional) bankrun setup
│
├── Anchor.toml                       # Updated with program IDs, test config
├── Cargo.toml                        # Workspace root – only programs + libraries members
├── package.json                      # For running anchor tests (if TS)
├── tsconfig.json                     # TS config for tests
├── .gitignore
└── README.md                         # Will be rebuilt in Week 2

```
