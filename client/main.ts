import { PublicKey, Keypair, SystemProgram, LAMPORTS_PER_SOL, TransactionMessage, VersionedTransaction, Connection } from "@solana/web3.js";

const payer = Keypair.fromSecretKey(Uint8Array.from([]))
const connection= new Connection("https://api.devnet.solana.com","confirmed");

//call this once after initializing the hook
//this function creates a counter that counts CPIs made into FRRNG program
//each number in counter is used as a seed to derive a unique pda to pass into FPRNG program
async function create_hook_counter() {

  const hook_program = new PublicKey("54GNE9AuT5juYGVbYBTMakgo1ACgf65sZaCq32AVSHSj");

  const newAccount = Keypair.generate()

  const ix = SystemProgram.createAccount({
    fromPubkey:payer.publicKey,
    newAccountPubkey:newAccount.publicKey,
    space:8,
    lamports:LAMPORTS_PER_SOL*0.01,
    programId:hook_program
  })

  const message = new TransactionMessage({
    instructions: [ix],
      payerKey: payer.publicKey,
      recentBlockhash : (await connection.getLatestBlockhash()).blockhash
    }).compileToV0Message();

    const tx = new VersionedTransaction(message);
    tx.sign([payer,newAccount]);

    console.log(newAccount.publicKey.toBase58())

  const sig = await connection.sendTransaction(tx);


}

//users who want to call FPRNG on a token tranfer need to add lamports to this account for keeping calling the rng
async function get_user_account() {

  const mint = new PublicKey("f4xD9KagBKJfJM8f8WFj6pz2E2YJUoYLggac1Kg7Cc5");


  const hook_program = new PublicKey("54GNE9AuT5juYGVbYBTMakgo1ACgf65sZaCq32AVSHSj");
  const current_feed = PublicKey.findProgramAddressSync([mint.toBytes(),payer.publicKey.toBytes()],hook_program);

  console.log(current_feed[0].toBase58())

}

