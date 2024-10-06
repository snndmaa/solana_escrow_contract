import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Escrow } from "../target/types/escrow";

describe("escrow", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Escrow as Program<Escrow>;

  // Define test accounts
  const employer = anchor.web3.Keypair.generate();
  const worker = anchor.web3.Keypair.generate();
  const escrowAccount = anchor.web3.Keypair.generate();

  console.log("employer:", employer.publicKey.toString())
  console.log("worker:", worker.publicKey.toString())
  console.log("escrowAccount:", escrowAccount.publicKey.toString())

  const job = anchor.web3.Keypair.generate();

  const jobId = "job1";
  const title = "Software Engineer";
  const pay = new anchor.BN(1000);

  before(async () => {
    // Airdrop SOL to employer and worker accounts for transaction fees
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(employer.publicKey, 2 * anchor.web3.LAMPORTS_PER_SOL)
    );

    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(worker.publicKey, 2 * anchor.web3.LAMPORTS_PER_SOL)
    );

    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(job.publicKey, 2 * anchor.web3.LAMPORTS_PER_SOL)
    );

    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(escrowAccount.publicKey, 2 * anchor.web3.LAMPORTS_PER_SOL)
    );

    // job = anchor.web3.Keypair.generate(); // New job keypair for each test
  });

  it("Creates a new job", async () => {

    await program.methods
      .createJob(jobId, title, pay)
      .accounts({
        job: job.publicKey,
        employer: employer.publicKey,
        worker: worker.publicKey,
        escrowAccount: escrowAccount.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([employer, worker, job, escrowAccount])
      .rpc();

    // Fetch the job account and verify data
    const jobAccount = await program.account.job.fetch(job.publicKey);
    const escrowBalance = await provider.connection.getBalance(escrowAccount.publicKey);

    // assert.equal(jobAccount.id, jobId);
    // assert.equal(jobAccount.title, title);
    // assert.equal(jobAccount.pay.toString(), pay.toString());
    // assert.equal(jobAccount.employer.toString(), employer.publicKey.toString());
    // assert.equal(jobAccount.worker.toString(), worker.publicKey.toString());
    console.log(jobAccount.status, escrowBalance);
  });

  it("worker approves job", async () => {
    await program.methods
    .approveJobWorker()
    .accounts({
      job: job.publicKey,
      worker: worker.publicKey,
      escrowAccount: escrowAccount.publicKey,
      systemProgram: anchor.web3.SystemProgram.programId,
    })
    .signers([worker])
    .rpc();

    // Fetch the job account and verify its status
    const jobAccount = await program.account.job.fetch(job.publicKey);
    console.log(jobAccount.status);
    // assert.isTrue(jobAccount.workerApproved);
    // assert.isTrue(jobAccount.employerApproved);
  });

  it("employer approves the job and completes payment", async () => {
    // Call employer approval
    await program.methods
      .approveJobEmployer()
      .accounts({
        job: job.publicKey,
        employer: employer.publicKey,
        worker: worker.publicKey,
        escrowAccount: escrowAccount.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([employer, escrowAccount])
      .rpc();

    // Fetch job state after employer approval
    const jobAccount = await program.account.job.fetch(job.publicKey);

    // Assert both approvals are registered
    // assert.ok(jobAccount.workerApproved);
    // assert.ok(jobAccount.employerApproved);

    // Assert job is marked as completed
    console.log(jobAccount.worker);

    // Check worker balance after payment (assuming payment has been completed)
    const workerBalance = await provider.connection.getBalance(worker.publicKey);
    console.log('workerBalance:', workerBalance);

    console.log(jobAccount.status)
  });

});
