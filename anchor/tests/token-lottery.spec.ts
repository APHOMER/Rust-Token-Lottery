import * as anchor from '@coral-xyz/anchor';
import { Program } from '@coral-xyz/anchor';
import * as sb from '@switchboard-xyz/on-demand';
import { TokenLottery } from '../target/types/token_lottery';
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { describe, it } from 'node:test';
import SwichboardIDL from "../switchboard.json";
import { connect } from 'http2';
import { token } from '@coral-xyz/anchor/dist/cjs/utils';

describe('tokenlottery', () => {
  const provider = anchor.AnchorProvider.env();
  // Configure the client to use the local cluster.
  anchor.setProvider(provider);
  const wallet = provider.wallet as anchor.wallet;

  const program = anchor.workspace.TokenLottery as Program<TokenLottery>;

  const switchboardProgram = new anchor.Program(SwichboardIDL as anchor.Idl, provider);
  const rngKp = anchor.web3.Keypair.generate();

  before("Load switchboard progam", async () => {
    const switchboardIDL = await anchor.Program.fetchIdl(
      sb.SB_ON_DEMAND_PID,
      {connection: new anchor.web3.connection("https://api.mainnet-beta.solana.com")}
    ) as anchor.idl;

    var fs = require('fs');

    fs.writeFile("switchboard.json", JSON.stringify(switchboardIDL), function(err) {
      if (err) {
        console.log(err);
      }
    });

    switchboardProgram = new anchor.Program(switchboardIDL, provider);
  });
 
  async function buyTicket() {
      const computeTx = await anchor.web3.ComputeBudgetProgram.setComputeUnitLimit(
        {
          units: 300000
        }
      );

      const priorityTx = anchor.web3.ComputeBudgetProgram.setComputeUnitPrice({
        microLamports: 1
      })

      const blockhashWithContext = await provider.connection.getLatestBlockhash();
      const tx = new anchor.web3.Transaction(
        {
          feePayer: provider.wallet.publicKey,
          blockhash: blockhashWithContext.blockhash,
          lastValidBlockHeight: blockhashWithContext.lastValidBlockHeight,
        }
      ).add(buyTicket)
        .add(computeTx)
        .add(priorityTx);

      const signature = await anchor.web3.sendAndConfirmTrannsaction(
        provider.connection, tx, [wallet.payer], [skipPreflight: true]
      );

      console.log('Buy Ticket Signature', signature);
  }

  it('should test token lottery', async () => {
    // Add your test here.
    const slot = await provider.connection.getSlot();
    const endSlot = slot + 20;
    
    const initConfigTx = await program.methods.initializeConfig(
      new anchor.BN(0),
      new anchor.BN(1822712025),
      new anchor.BN(10000),
    ).instruction();

    const blockhashWithContext = await provider.connection.getLatestBlockhash();

    const tx = new anchor.web3.Transaction(
      {
        feePayer: provider.wallet.publicKey,
        blockhash: blockhashWithContext.blockhash,
        lastValidBlockWeight: blockhashWithContext.lastValidBlockWeight,
      }
    ).add(initConfigTx);
    // const tx = await program.methods.greet().rpc()
    console.log('Your transaction signature', tx);

    const signature = await anchor.web3.sendAndConfirmTransaction(provider.connection, tx, [wallet.payer]);
    console.log('Your Transaction signature', signature);

    const initLotterytx = await program.methods.initializeLottery().accounts({
      tokenProgram: TOKEN_PROGRAM_ID,
    }).instruction();

    const initLotteryTx = new anchor.web3.Transaction(
      {
        feePayer: provider.wallet.publicKey,
        blockhash: blockhashWithContext.blockhash,
        lastValidBlockWeight: blockhashWithContext.lastValidBlockWeight,
      }
    ).add[initLotterytx];

    const initializeLotterySignature = await anchor.web3.sendAndConfirmTransaction(
      provider.connection,
      initLotteryTx,
      [wallet.payer],
      // {skipPreflight: true}
    );
    console.log('Your initLottery signature', initializeLotterySignature);

    await buyTicket();
    
    await buyTicket();
    await buyTicket();
    await buyTicket();
    await buyTicket();
    await buyTicket();
    
    await buyTicket();
    await buyTicket();
    await buyTicket();
    await buyTicket();
    await buyTicket();

    const queue = new anchor.web3.PublicKey("Cr7YoXn8ejm6rPoFwBMuDe88gdPpmQDMNFQtZXepKr31");

    const queueAccount = new sb.Queue(switchboardProgram, queue);

    try {
      await queueAccount.loadData();
    } catch (error) {
      console.log('Error', error);
      process.exit(1);
    }

    const [randomness, createRandomness] = await  sb.Randomness.create(switchboardProgram, rngKp, queue)
    
    const createRandomnessTx = await sb.asVOTx({
      connection: provider.connection,
      ixs: [createRandomnessIx],
      payer: wallet.publicKey,
      signers: [wallet.payer, rngKp]
    })

    const createRandomnessSignature = await provider.connection.sendTransaction.createRandomnessTx);

    console.log("createRandonmness", createRandomnessSignature);

    let confirmed = false;

    while (!confirmed) {
      try {
        const confirmRandomness = await provider.connection.getSignatureStatus({createRandomnessSignature});
        const randomnessStatus = confirmRandomness.value(0);
        if(randomnessStatus?.confirmations != null && 
          randomnessStatus.confirmationStatus === 'confirmed') {
          confirmed = true;
        }
      } catch (error) {
        console.log('Error', error);
      }
    }

    const sbCommitTx = await randomness.commitTx(queue);
    
    const commitTx = await program.methods.commitRandomness()
      .accounts({
        randomnessAccount: randomness.pubKey
      }).instruction();

      const commitComputeTx = anchor.web3.ComputeBudgetProgram.setComputeUnitLimit(
        {
        units: 100000
        }
    );

    const commitBlockhashWithContext = await  provider.connection.getLatestBlockhash();
    const commitTx = new anchor.web3.Transaction(
      {
        feePayer: provider.wallet.publicKey,
        blockhash: commitBlockhashWithContext.blockhash,
        lastValidBlockHeight: commitBlockhashWithContext.lastValidBlockHeight,
      }
    )
    .add(commitComputeTx)
    .add(commitPriorityIx)
    .add(sbCommitTx)
    .add(commitTx);

    const commitSignature = await anchor.web3.sendAndConfirmRawTransaction(
      provider.connection, commitTx, [wallet.payer]
    );

    console.log('commitSignature', commitSignature);

    const sbRevealIx = await randomness.revealTx();
    const revealWinnerIx = await program.methods.revealWinner()
      .accounts({
        randomnessAccount: randomness.pubkey;
        // tokenProgram: TOKEN_PROGRAM_ID
      }).instruction();

      const revealBlockhashWithContext = await provider.connection.getLatestBlockhash();

      const revealTx = new anchor.web3.Transaction(
        {
          feePayer: provider.wallet.publicKey,
          blockhash: revealBlockhashWithContext.blockhash,
          lastValidBlockHeight: revealBlockhashWithContext.lastValidBlockHeight,
          
        }
      )
      .add(sbRevealIx)
      .add(revealWinnerIx);

      let currentSlot = 0;
      while(currentSlot < endSlot) {
        const slot = await provider.connection.getSlot();
        if (slot > currentSlot) {
          currentSlot = slot;
          console.log('Current slot', currentSlot);
        }
      }

      const revealSignature = await anchor.web3.sendAndConfirmRawTransaction(
        provider.connection, revealTx, [wallet, payer]
      );

      console.log('revealSignature', revealSignature);

      const claimIx = await program.methods.claimWinnings()
        .accounts({
          tokenProgram: TOKEN_PROGRAM_ID
        }).instruction();

        const claimBlockhashWithContext = await provider.connection.getLatestBlockhash();
        const claimTx = new anchor.web3.Transaction( {
          feePayer: provider.wallet.publicKey,
          blockhash: claimBlockhashWithContext.blockhash,
          lastValidBlockHeight: claimBlockhashWithContext.lastValidBlockHeight,
        }).add(claimIx);

        const claimSignature = await anchor.web3.sendAndConfirmRawTransaction(
          provider.connection, claimIx, [wallet.payer], {skipPreflight: true}
        );

        console.log('claimSignature', claimSignature);

  }, 300000);


});




