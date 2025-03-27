import * as anchor from "@coral-xyz/anchor";
import BN from "bn.js";
import * as web3 from "@solana/web3.js";
import {
  Connection,
  Keypair,
  SystemProgram,
  Transaction,
  clusterApiUrl,
  sendAndConfirmTransaction,
  PublicKey,
} from "@solana/web3.js";
import type { Ballot } from "../target/types/ballot";

// Configure the client to use the local cluster
anchor.setProvider(anchor.AnchorProvider.env());

const program = anchor.workspace.Ballot as anchor.Program<Ballot>;


const wallet = pg.wallet;
const connection = new Connection(clusterApiUrl("devnet"), "confirmed");
const program = program;

const [pda0, bump0] = PublicKey.findProgramAddressSync(
  [Buffer.from("TEST"), Buffer.from("0")],
  program.programId
);
const [pda1, bump1] = PublicKey.findProgramAddressSync(
  [Buffer.from("TEST"), Buffer.from("1")],
  program.programId
);
const [pda2, bump2] = PublicKey.findProgramAddressSync(
  [Buffer.from("TEST"), Buffer.from("2")],
  program.programId
);
const [pda3, bump3] = PublicKey.findProgramAddressSync(
  [Buffer.from("TEST"), Buffer.from("3")],
  program.programId
);

const init_instruction_0 = await program.methods
  .init("TEST", "0", ["Spring", "Yarn", "Combat"])
  .accounts({
    payer: wallet.publicKey,
    ballotBox: pda0,
  })
  .instruction();
const init_instruction_1 = await program.methods
  .init("TEST", "1", ["Spring", "Yarn", "Combat"])
  .accounts({
    payer: wallet.publicKey,
    ballotBox: pda1,
  })
  .instruction();
const init_instruction_2 = await program.methods
  .init("TEST", "2", ["Spring", "Yarn", "Combat"])
  .accounts({
    payer: wallet.publicKey,
    ballotBox: pda2,
  })
  .instruction();
const init_instruction_3 = await program.methods
  .init("TEST", "3", ["Spring", "Yarn", "Combat"])
  .accounts({
    payer: wallet.publicKey,
    ballotBox: pda3,
  })
  .instruction();

// add both instruction to one transaction
const transaction = new Transaction()
  .add(init_instruction_0)
  .add(init_instruction_1)
  .add(init_instruction_2)
  .add(init_instruction_3);

// send transaction
// await sendAndConfirmTransaction(connection, transaction, [wallet.keypair]);

// const ballot_boxes = await program.account.ballotBox.all();
const ballot_boxes = await connection.getMultipleAccountsInfo([
  pda0,
  pda1,
  pda2,
  pda3,
]);

console.log("begin: ", ballot_boxes[1].data.toString());
// 打印所有 ballotBox 账户的信息
// console.log("Found", ballot_boxes.length, "ballotBox accounts:");
// ballot_boxes.forEach((ballotBox, index) => {
//   console.log(`\n--- BallotBox ${index + 1} ---`);
//   console.log("Account Address:", ballotBox.publicKey.toString());
//   console.log("Account Data:", ballotBox.account);
// });

const vote_instruction_0 = await program.methods
  .vote(new BN(0))
  .accounts({
    ballotBox: pda0,
  })
  .instruction();
const vote_instruction_1 = await program.methods
  .vote(new BN(0))
  .accounts({
    ballotBox: pda1,
  })
  .instruction();
const vote_instruction_2 = await program.methods
  .vote(new BN(1))
  .accounts({
    ballotBox: pda2,
  })
  .instruction();
const vote_instruction_3 = await program.methods
  .vote(new BN(2))
  .accounts({
    ballotBox: pda3,
  })
  .instruction();

// add both instruction to one transaction
const vote_transaction = new Transaction()
  .add(vote_instruction_0)
  .add(vote_instruction_1)
  .add(vote_instruction_2)
  .add(vote_instruction_3);

// send transaction
await sendAndConfirmTransaction(connection, vote_transaction, [wallet.keypair]);

const ballot_boxes_after = await connection.getMultipleAccountsInfo([
  pda0,
  pda1,
  pda2,
  pda3,
]);

console.log("after: ", ballot_boxes_after[0].data);
