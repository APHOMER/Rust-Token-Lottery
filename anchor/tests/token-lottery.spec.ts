import * as anchor from '@coral-xyz/anchor';
import { Program } from '@coral-xyz/anchor';
import { TokenLottery } from '../target/types/token_lottery';
import { describe, it } from 'node:test';
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";

describe('tokenlottery', () => {
  const provider = anchor.AnchorProvider.env();
  // Configure the client to use the local cluster.
  anchor.setProvider(provider);
  const wallet = provider.wallet as anchor.wallet;

  const program = anchor.workspace.TokenLottery as Program<TokenLottery>;

  async function buyTicket() {
    const buyTicketTx = await program.methods.buyTicket()
      .accounts({
        tokenProgram: TOKEN_PROGRAM_ID
      })
      .instruction();

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

  });
});


