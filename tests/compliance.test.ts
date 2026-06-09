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