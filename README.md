# solana-compliance-hook
Compliance Hook demo, compliant payment processor on Solana now needs Transfer Hook logic for KYC allow lists, blacklists, transfer fees, and compliance gating

### repo architecture 
```
solana-compliance-hook/

├── programs/

│ └── solana-compliance-hook/

│ ├── Cargo.toml

│ └── src/

│ ├── lib.rs

│ ├── error.rs

│ ├── instructions/

│ │ ├── mod.rs

│ │ ├── set_mode.rs

│ │ ├── manage_list.rs

│ │ └── execute.rs

│ └── state/

│ ├── mod.rs

│ ├── compliance_config.rs

│ └── compliance_list.rs

├── tests/

│ ├── compliance.test.ts

│ └── utils.ts

├── Anchor.toml

├── Cargo.toml (workspace)

├── package.json

├── tsconfig.json

├── .gitignore

└── README.md (placeholder)                       # Will be rebuilt in Week 2

```
