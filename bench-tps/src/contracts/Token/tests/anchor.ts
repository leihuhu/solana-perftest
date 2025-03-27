import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { SimpleTransfer } from "../target/types/simple_transfer";
import { expect } from "chai";
import { PublicKey } from "@solana/web3.js";

describe("simple-transfer", () => {
  // 配置 Anchor 提供者
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  // 获取程序实例
  const program = anchor.workspace.SimpleTransfer as Program<SimpleTransfer>;
  
  // 测试账户
  const payer = anchor.web3.Keypair.generate();
  const user1 = anchor.web3.Keypair.generate();
  const user2 = anchor.web3.Keypair.generate();
  
  // PDA 账户
  let user1PDA: PublicKey;
  let user2PDA: PublicKey;
  let user1Bump: number;
  let user2Bump: number;

  // 在所有测试前运行一次
  before(async () => {
    // 为测试账户空投一些 SOL
    const airdropSignature = await provider.connection.requestAirdrop(
      payer.publicKey,
      2 * anchor.web3.LAMPORTS_PER_SOL
    );
    
    await provider.connection.confirmTransaction(airdropSignature);
    
    // 计算 PDA
    const [pda1, bump1] = await PublicKey.findProgramAddress(
      [Buffer.from("balance"), payer.publicKey.toBuffer()],
      program.programId
    );
    user1PDA = pda1;
    user1Bump = bump1;
    
    const [pda2, bump2] = await PublicKey.findProgramAddress(
      [Buffer.from("balance"), user1.publicKey.toBuffer()],
      program.programId
    );
    user2PDA = pda2;
    user2Bump = bump2;
  });

  it("初始化用户账户", async () => {
    // 初始化第一个用户账户
    await program.methods
      .initUser(new anchor.BN(1000))
      .accounts({
        userAccount: user1PDA,
        payer: payer.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([payer])
      .rpc();

    // 验证账户数据
    const userAccount1 = await program.account.userBalance.fetch(user1PDA);
    expect(userAccount1.owner.toString()).to.equal(payer.publicKey.toString());
    expect(userAccount1.balance.toNumber()).to.equal(1000);

    // 初始化第二个用户账户
    await program.methods
      .initUser(new anchor.BN(500))
      .accounts({
        userAccount: user2PDA,
        payer: user1.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([user1])
      .rpc();

    // 验证账户数据
    const userAccount2 = await program.account.userBalance.fetch(user2PDA);
    expect(userAccount2.owner.toString()).to.equal(user1.publicKey.toString());
    expect(userAccount2.balance.toNumber()).to.equal(500);
  });

  it("转账成功", async () => {
    // 执行转账
    await program.methods
      .transfer(new anchor.BN(300))
      .accounts({
        from: user1PDA,
        to: user2PDA,
        owner: payer.publicKey,
      })
      .signers([payer])
      .rpc();

    // 验证转账后的余额
    const userAccount1 = await program.account.userBalance.fetch(user1PDA);
    const userAccount2 = await program.account.userBalance.fetch(user2PDA);
    
    expect(userAccount1.balance.toNumber()).to.equal(700); // 1000 - 300
    expect(userAccount2.balance.toNumber()).to.equal(800); // 500 + 300
  });

  it("余额不足时转账失败", async () => {
    try {
      // 尝试转账超过余额的金额
      await program.methods
        .transfer(new anchor.BN(1000))
        .accounts({
          from: user1PDA,
          to: user2PDA,
          owner: payer.publicKey,
        })
        .signers([payer])
        .rpc();
      
      // 如果执行到这里，说明没有抛出错误，测试应该失败
      expect.fail("应该因为余额不足而失败");
    } catch (error) {
      // 验证错误是否为余额不足
      expect(error.error.errorMessage).to.include("Insufficient Balance");
    }
  });

  it("非所有者转账失败", async () => {
    try {
      // 使用错误的所有者尝试转账
      await program.methods
        .transfer(new anchor.BN(100))
        .accounts({
          from: user1PDA,
          to: user2PDA,
          owner: user1.publicKey, // 错误的所有者
        })
        .signers([user1])
        .rpc();
      
      // 如果执行到这里，说明没有抛出错误，测试应该失败
      expect.fail("应该因为所有者不匹配而失败");
    } catch (error) {
      // 验证错误是否为所有者不匹配
      expect(error.error.errorMessage).to.include("Owner Mismatch");
    }
  });
});