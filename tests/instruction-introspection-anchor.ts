import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { InstructionIntrospectionAnchor } from "../target/types/instruction_introspection_anchor";
import {PublicKey, SystemProgram, SYSVAR_INSTRUCTIONS_PUBKEY} from "@solana/web3.js";
import { ASSOCIATED_TOKEN_PROGRAM_ID, createMint, getOrCreateAssociatedTokenAccount, mintTo, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { publicKey } from "@coral-xyz/anchor/dist/cjs/utils";
import { Transaction } from "@solana/web3.js";
import { createTransferCheckedInstruction } from "@solana/spl-token";

let provider = anchor.AnchorProvider.env();
anchor.setProvider(provider);

const program = anchor.workspace.instructionIntrospectionAnchor as Program<InstructionIntrospectionAnchor>;

let mint: PublicKey;
let fromAccount: PublicKey;
let toAccount: PublicKey;
let instructionSummaryAccount: PublicKey;
let securityAnalysisAccount: PublicKey;
let instrosepctionResultAccount: PublicKey;

before(async () => {
  mint = await createMint (
    provider.connection,
    provider.wallet.payer,
    provider.wallet.publicKey,
    null,
    6
  );

  let fromAtaAccount = await getOrCreateAssociatedTokenAccount(
    provider.connection,
    provider.wallet.payer,
    mint,
    provider.wallet.publicKey,
  );

  fromAccount = fromAtaAccount.address;

  let toAtaAccount = await getOrCreateAssociatedTokenAccount(
    provider.connection,
    provider.wallet.payer,
    mint,
    provider.wallet.publicKey,
  );

  toAccount = toAtaAccount.address;

  [instructionSummaryAccount] = PublicKey.findProgramAddressSync(
    [Buffer.from("instruction_summary"), provider.wallet.publicKey.toBuffer()],
    program.programId
  );

  [securityAnalysisAccount] = PublicKey.findProgramAddressSync(
    [Buffer.from("security_analysis"), provider.wallet.publicKey.toBuffer()],
    program.programId
  );

  [instrosepctionResultAccount] = PublicKey.findProgramAddressSync(
    [Buffer.from("introspection_result"), provider.wallet.publicKey.toBuffer()],
    program.programId
  );
})

describe("instruction-introspection-anchor", () => {

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });

  it("Process Transfer Introspection", async () => {

    await mintTo(
      provider.connection,
      provider.wallet.payer,
      mint,
      fromAccount, 
      provider.wallet.publicKey,
      500
    );

    let amount = new anchor.BN(100);

    const tx = await program.methods.processTransferIntrospection(amount).accounts({
      sender: provider.wallet.publicKey,
      recipient: provider.wallet.publicKey,
      mint,
      from: fromAccount,
      to: toAccount,
      instructionSummaryAccount,
      securityAnalysisAccount,
      instrosepctionResultAccount,
      SYSVAR_INSTRUCTIONS_PUBKEY,
      systmeProgram: SystemProgram.programId,
      tokenProgram: TOKEN_PROGRAM_ID,
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
    }).signers([provider.wallet.payer]).rpc();

    console.log(`Transsction Signatire: ${tx}`);
  });

  it("Process Transfer Introspection with real SPL transfer", async () => {
  const amount = 100;

  // 1. SPL Token transfer
  const transferIx = createTransferCheckedInstruction(
    fromAccount,
    mint,
    toAccount,
    provider.wallet.publicKey, // authority
    amount,
    6, // decimals
    [],
    TOKEN_PROGRAM_ID
  );

  // 2. Your program introspection instruction
  const programIx = await program.methods
    .processTransferIntrospection(new anchor.BN(amount))
    .accounts({
      sender: provider.wallet.publicKey,
      recipient: provider.wallet.publicKey,
      mint,
      fromAccount,
      toAccount,
      instructionSummaryAccount,
      securityAnalysisAccount,
      instrosepctionResultAccount,
      instructions: SYSVAR_INSTRUCTIONS_PUBKEY,
    })
    .instruction();

  // Send both in one transaction
    const tx = new Transaction().add(transferIx, programIx);
    const sig = await provider.sendAndConfirm(tx);
    console.log("Tx:", sig);
  });
});
