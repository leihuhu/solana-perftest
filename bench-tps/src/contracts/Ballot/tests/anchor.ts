import BN from "bn.js";
import * as web3 from "@solana/web3.js";
import * as anchor from "@coral-xyz/anchor";
import * as anchor from "@coral-xyz/anchor";
import { Keypair, PublicKey } from "@solana/web3.js";
import type { Ballot } from "../target/types/ballot";

describe("ballot test", () => {
  // Configure the client to use the local cluster
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.Ballot as anchor.Program<Ballot>;
  
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const payer = provider.wallet as anchor.Wallet;

  const program = program;

  // Generate a new keypair for the counter account
  const [pda, bump] = PublicKey.findProgramAddressSync(
    [Buffer.from("TEST"), Buffer.from("0")],
    program.programId
  );

  it("Initialize Counter", async () => {
    await program.methods
      .init("TEST", "0", ["Spring", "Yarn", "Combat"])
      .accounts({
        ballotBox: pda,
        payer: payer.publicKey,
      })
      .signers([payer.payer])
      .rpc();

    const ballotBox = await program.account.ballotBox.fetch(pda);

    console.log("current votes count is ", ballotBox.votes.toString());
  });

  it("Vote Yarn", async () => {
    await program.methods.vote(new BN(1)).accounts({ ballotBox: pda }).rpc();

    const currentVotes = await program.account.ballotBox.fetch(pda);
    console.log(
      "current votes is ",
      currentVotes.proposals.toString(),
      ", ",
      currentVotes.votes.toString()
    );
  });

  it("Vote Spring", async () => {
    await program.methods.vote(new BN(0)).accounts({ ballotBox: pda }).rpc();

    const currentVotes = await program.account.ballotBox.fetch(pda);
    console.log(
      "current votes is ",
      currentVotes.proposals.toString(),
      ", ",
      currentVotes.votes.toString()
    );
  });
  it("Vote Combat", async () => {
    await program.methods.vote(new BN(2)).accounts({ ballotBox: pda }).rpc();

    const currentVotes = await program.account.ballotBox.fetch(pda);
    console.log(
      "current votes is ",
      currentVotes.proposals.toString(),
      ", ",
      currentVotes.votes.toString()
    );
  });
});
