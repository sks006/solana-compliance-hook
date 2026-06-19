import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import type { SolanaComplianceHook } from "../target/types/solana_compliance_hook";
import { 
  PublicKey, 
  Keypair, 
  SystemProgram, 
  Transaction, 
  sendAndConfirmTransaction 
} from "@solana/web3.js";
import { expect } from "chai";

describe("solana-compliance-hook", () => {
  // Configure the client to use the local cluster provider
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.SolanaComplianceHook as Program<SolanaComplianceHook>;
  const wallet = provider.wallet as anchor.Wallet;

  // Structural Invariants: Define cryptographic state identities
  const mockMint = Keypair.generate();
  let complianceConfigPda: PublicKey;
  let complianceListPda: PublicKey;
  let extraMetasPda: PublicKey;

  // Enum variant maps matching compliance_mode.rs layout
  const ComplianceMode = {
    None: { none: {} },
    AllowList: { allowList: {} },
    BlackList: { blackList: {} },
    Both: { both: {} },
  };

  const ListType = {
    Allow: { allow: {} },
    Block: { block: {} },
  };

  before(async () => {
    // 🗲 Rule: Derive PDAs strictly based on the SEED_PREFIX fields declared in Rust
    [complianceConfigPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("compliance-config"), mockMint.publicKey.toBuffer()],
      program.programId
    );

    [complianceListPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("compliance-list"), mockMint.publicKey.toBuffer()],
      program.programId
    );

    [extraMetasPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("extra-account-metas"), mockMint.publicKey.toBuffer()],
      program.programId
    );
  });

  describe("⚙️ Configuration State Machine", () => {
    it("Initializes the compliance mode rules and sets target variables", async () => {
      const targetFeeBps = 500; // 5% fee profile
      const feeCollector = Keypair.generate().publicKey;

      // Type Signatures required for instruction route parameter mapping
      await program.methods
        .setMode(ComplianceMode.AllowList, targetFeeBps, feeCollector)
        .accounts({
          config: complianceConfigPda,
          mint: mockMint.publicKey,
          authority: wallet.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .rpc();

      // Fetch state back from the ledger to evaluate binary consistency
      const configAccount = await program.account.complianceConfig.fetch(complianceConfigPda);
      
      expect(configAccount.authority.toBase58()).to.equal(wallet.publicKey.toBase58());
      expect(configAccount.feeBasisPoints).to.equal(targetFeeBps);
      expect(configAccount.feeRecipient.toBase58()).to.equal(feeCollector.toBase58());
      expect(configAccount.mode).to.have.property("allowList");
    });
  });

  describe("📜 Compliance Vector Management", () => {
    const targetUser = Keypair.generate().publicKey;

    it("Appends a strategic wallet address directly to the Allow List vector", async () => {
      await program.methods
        .addToList(ListType.Allow, targetUser)
        .accounts({
          config: complianceConfigPda,
          complianceList: complianceListPda,
          mint: mockMint.publicKey,
          authority: wallet.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .rpc();

      const listAccount = await program.account.complianceList.fetch(complianceListPda);
      
      const parsedAddresses = listAccount.allowList.map(pubkey => pubkey.toBase58());
      expect(parsedAddresses).to.include(targetUser.toBase58());
    });

    it("Removes a previously validated public key from the tracking vector", async () => {
      await program.methods
        .removeFromList(ListType.Allow, targetUser)
        .accounts({
          config: complianceConfigPda,
          complianceList: complianceListPda,
          mint: mockMint.publicKey,
          authority: wallet.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .rpc();

      const listAccount = await program.account.complianceList.fetch(complianceListPda);
      const parsedAddresses = listAccount.allowList.map(pubkey => pubkey.toBase58());
      expect(parsedAddresses).to.not.include(targetUser.toBase58());
    });
  });

  describe("🧬 Extra Accounts Metadata Registry Initialization", () => {
    it("Pre-allocates data storage structures to configure the transfer hook list layout", async () => {
      // Execute transaction logic to satisfy initialize_extra_metas constraints
      const tx = await program.methods
        .initializeExtraMetas()
        .accounts({
          payer: wallet.publicKey,
          extraMetasAccount: extraMetasPda,
          mint: mockMint.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .rpc();

      expect(tx).to.be.a("string");
    });
  });
});








// //=================================================================

// import * as anchor from "@coral-xyz/anchor";
// import { Program } from "@coral-xyz/anchor";
// import type { SolanaComplianceHook } from "../target/types/solana_compliance_hook";
// import { 
//   PublicKey, 
//   Keypair, 
//   SystemProgram, 
//   Transaction,
//   sendAndConfirmTransaction
// } from "@solana/web3.js";
// import { 
//   ExtensionType, 
//   TOKEN_2022_PROGRAM_ID, 
//   getMintLen, 
//   createInitializeMintInstruction,
//   createInitializeTransferHookInstruction,
//   createInitializeTransferFeeConfigInstruction,
//   createAssociatedTokenAccountInstruction,
//   getAssociatedTokenAddressSync,
//   createMintToInstruction,
//   createTransferCheckedWithTransferHookInstruction
// } from "@solana/spl-token";
// import { expect } from "chai";

// describe("solana-compliance-hook", () => {
//   const provider = anchor.AnchorProvider.env();
//   anchor.setProvider(provider);

//   const program = anchor.workspace.SolanaComplianceHook as Program<SolanaComplianceHook>;
//   const wallet = provider.wallet as anchor.Wallet;

//   // Structural Invariants: Cryptographic identities
//   const mockMint = Keypair.generate();
//   let complianceConfigPda: PublicKey;
//   let complianceListPda: PublicKey;
//   let extraMetasPda: PublicKey;

//   // Strict Anchor IDL Serialization Mappings
//   const ComplianceMode = {
//     allowList: { allowList: {} },
//     blackList: { blackList: {} },
//     both: { both: {} },
//   };

//   const ListType = {
//     allow: { allow: {} },
//     block: { block: {} },
//   };

//   // Test Accounts for simulated retail card transfer loops
//   const alice = Keypair.generate(); 
//   const bob = Keypair.generate();   
//   let aliceAta: PublicKey;
//   let bobAta: PublicKey;

//   before(async () => {
//     // 1. Derive structural PDA mappings based on program-pinned prefixes
//     [complianceConfigPda] = PublicKey.findProgramAddressSync(
//       [Buffer.from("compliance-config"), mockMint.publicKey.toBuffer()],
//       program.programId
//     );

//     [complianceListPda] = PublicKey.findProgramAddressSync(
//       [Buffer.from("compliance-list"), mockMint.publicKey.toBuffer()],
//       program.programId
//     );

//     [extraMetasPda] = PublicKey.findProgramAddressSync(
//       [Buffer.from("extra-account-metas"), mockMint.publicKey.toBuffer()],
//       program.programId
//     );

//     aliceAta = getAssociatedTokenAddressSync(mockMint.publicKey, alice.publicKey, false, TOKEN_2022_PROGRAM_ID);
//     bobAta = getAssociatedTokenAddressSync(mockMint.publicKey, bob.publicKey, false, TOKEN_2022_PROGRAM_ID);

//     // 2. Provision native lamport balances for fee transaction signing gas requirements
//     const airdropTx = new Transaction().add(
//       SystemProgram.transfer({
//         fromPubkey: wallet.publicKey,
//         toPubkey: alice.publicKey,
//         lamports: 500_000_000, 
//       })
//     );
//     await provider.sendAndConfirm(airdropTx);

//     // 3. Atomically deploy the parent Token-2022 Mint with embedded compliance hooks
//     const mintLen = getMintLen([ExtensionType.TransferHook, ExtensionType.TransferFeeConfig]);
//     const lamports = await provider.connection.getMinimumBalanceForRentExemption(mintLen);

//     const setupTx = new Transaction().add(
//       SystemProgram.createAccount({
//         fromPubkey: wallet.publicKey,
//         newAccountPubkey: mockMint.publicKey,
//         space: mintLen,
//         lamports,
//         programId: TOKEN_2022_PROGRAM_ID,
//       }),
//       createInitializeMintInstruction(
//         mockMint.publicKey,
//         6,
//         wallet.publicKey,
//         null,
//         TOKEN_2022_PROGRAM_ID
//       ),
//       createInitializeTransferHookInstruction(
//         mockMint.publicKey,
//         wallet.publicKey,
//         program.programId,
//         TOKEN_2022_PROGRAM_ID
//       ),
//       createInitializeTransferFeeConfigInstruction(
//         mockMint.publicKey,
//         wallet.publicKey,
//         wallet.publicKey,
//         500,
//         BigInt(1_000_000),
//         TOKEN_2022_PROGRAM_ID
//       ),
//       createAssociatedTokenAccountInstruction(
//         wallet.publicKey,
//         aliceAta,
//         alice.publicKey,
//         mockMint.publicKey,
//         TOKEN_2022_PROGRAM_ID
//       ),
//       createAssociatedTokenAccountInstruction(
//         wallet.publicKey,
//         bobAta,
//         bob.publicKey,
//         mockMint.publicKey,
//         TOKEN_2022_PROGRAM_ID
//       ),
//       createMintToInstruction(
//         mockMint.publicKey,
//         aliceAta,
//         wallet.publicKey,
//         1_000_000,
//         [],
//         TOKEN_2022_PROGRAM_ID
//       )
//     );
//     await provider.sendAndConfirm(setupTx, [mockMint]);
//   });

//   describe("⚙️ Configuration State Machine", () => {
//     it("Initializes the compliance mode rules and sets target variables", async () => {
//       const targetFeeBps = 500; 
//       const feeCollector = Keypair.generate().publicKey;

//       await program.methods
//         .setMode(ComplianceMode.allowList, targetFeeBps, feeCollector)
//         .accounts({
//           mint: mockMint.publicKey,
//         })
//         .rpc();

//       const configAccount = await program.account.complianceConfig.fetch(complianceConfigPda);
      
//       expect(configAccount.authority.toBase58()).to.equal(wallet.publicKey.toBase58());
//       expect(configAccount.feeBasisPoints).to.equal(targetFeeBps);
//       expect(configAccount.feeRecipient.toBase58()).to.equal(feeCollector.toBase58());
//       expect(configAccount.mode).to.have.property("allowList");
//     });
//   });

//   describe("📜 Compliance Vector Management", () => {
//     it("Appends Alice directly to the Allow List vector", async () => {
//       await program.methods
//         .addToList(ListType.allow, alice.publicKey)
//         .accounts({
//           mint: mockMint.publicKey,
//         })
//         .rpc();

//       const listAccount = await program.account.complianceList.fetch(complianceListPda);
//       const parsedAddresses = listAccount.allowList.map(pubkey => pubkey.toBase58());
//       expect(parsedAddresses).to.include(alice.publicKey.toBase58());
//     });
//   });

//   describe("🧬 Extra Accounts Metadata Registry Initialization", () => {
//     it("Pre-allocates data storage structures to configure the transfer hook list layout", async () => {
//       const tx = await program.methods
//         .initializeExtraMetas()
//         .accounts({
//           payer: wallet.publicKey,
//           extraMetasAccount: extraMetasPda,
//           mint: mockMint.publicKey,
//         })
//         .rpc();

//       expect(tx).to.be.a("string");
//     });
//   });

//   describe("🛡️ Synchronous Compliance Hook Verification Execution", () => {
//     it("Fails a transfer synchronously if the recipient is missing from the KYC Allowlist", async () => {
//       // Invariant Check: Bob has not been allowlisted yet. Runtime execution must revert.
//       const transferInstruction = await createTransferCheckedWithTransferHookInstruction(
//         provider.connection,
//         aliceAta,
//         mockMint.publicKey,
//         bobAta,
//         alice.publicKey,
//         BigInt(100_000),
//         6,
//         [],
//         "confirmed",
//         TOKEN_2022_PROGRAM_ID
//       );

//       const tx = new Transaction().add(transferInstruction);

//       try {
//         await sendAndConfirmTransaction(provider.connection, tx, [alice], { commitment: "confirmed" });
//         expect.fail("The compliance hook allowed an unverified wallet to receive assets.");
//       } catch (err: any) {
//         // Safe check for the logs inside the nested CPI failure array
//         const logHistory = err.logs ? err.logs.join("\n") : "";
//         expect(logHistory).to.include("NotAllowlisted");
//       }
//     });

//     it("Successfully moves balances once the destination address is whitelisted", async () => {
//       // Mutate ledger state to append Bob to the verified collection
//       await program.methods
//         .addToList(ListType.allow, bob.publicKey)
//         .accounts({
//           mint: mockMint.publicKey,
//         })
//         .rpc();

//       const transferInstruction = await createTransferCheckedWithTransferHookInstruction(
//         provider.connection,
//         aliceAta,
//         mockMint.publicKey,
//         bobAta,
//         alice.publicKey,
//         BigInt(100_000),
//         6,
//         [],
//         "confirmed",
//         TOKEN_2022_PROGRAM_ID
//       );

//       const tx = new Transaction().add(transferInstruction);
//       const signature = await sendAndConfirmTransaction(provider.connection, tx, [alice]);
      
//       expect(signature).to.be.a("string");
//     });
//   });
// });