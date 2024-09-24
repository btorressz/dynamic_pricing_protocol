// Client
console.log("My address:", pg.wallet.publicKey.toString());

// Check the wallet balance
const balance = await pg.connection.getBalance(pg.wallet.publicKey);
console.log(`My balance: ${balance / web3.LAMPORTS_PER_SOL} SOL`);

// Keypairs for the price data and liquidity provider
const priceDataKp = new web3.Keypair();
const liquidityProviderKp = new web3.Keypair();

// Initialize Price Data
async function initializePriceData() {
  const initialPrice = new BN(100); // Initial price to set

  // Send transaction to initialize the price data
  const txHash = await pg.program.methods
    .initializePriceData(initialPrice)
    .accounts({
      priceData: priceDataKp.publicKey,
      user: pg.wallet.publicKey, // The user's public key (payer)
      systemProgram: web3.SystemProgram.programId,
    })
    .signers([priceDataKp]) // Sign the transaction with the price data account
    .rpc();

  console.log(`Price Data Initialized. Transaction hash: ${txHash}`);
}

// Update Price
async function updatePrice(newPrice) {
  const updatedPrice = new BN(newPrice); // New price to update

  const txHash = await pg.program.methods
    .updatePrice(updatedPrice)
    .accounts({
      priceData: priceDataKp.publicKey,
      user: pg.wallet.publicKey, // Authorized user
    })
    .rpc();

  console.log(`Price Updated. Transaction hash: ${txHash}`);
}

// Contribute Liquidity
async function contributeLiquidity(amount) {
  const liquidityAmount = new BN(amount); // Liquidity to contribute

  const txHash = await pg.program.methods
    .contributeLiquidity(liquidityAmount)
    .accounts({
      liquidityProvider: liquidityProviderKp.publicKey,
      priceData: priceDataKp.publicKey,
      user: pg.wallet.publicKey, // Signer contributing liquidity
      systemProgram: web3.SystemProgram.programId,
    })
    .signers([liquidityProviderKp]) // Sign with the liquidity provider account
    .rpc();

  console.log(`Liquidity Contributed. Transaction hash: ${txHash}`);
}

// Fetch Price Data
async function fetchPriceData() {
  const priceData = await pg.program.account.priceData.fetch(priceDataKp.publicKey);
  console.log("Price Data:", priceData);
}

// Fetch Liquidity Provider Data
async function fetchLiquidityProviderData() {
  const liquidityProviderData = await pg.program.account.liquidityProvider.fetch(liquidityProviderKp.publicKey);
  console.log("Liquidity Provider Data:", liquidityProviderData);
}

// Main flow of the client

(async () => {
  console.log("Initializing price data...");
  await initializePriceData(); // Initialize price data

  console.log("Updating price...");
  await updatePrice(150); // Update the price to 150

  console.log("Contributing liquidity...");
  await contributeLiquidity(500); // Contribute 500 liquidity

  console.log("Fetching price data...");
  await fetchPriceData(); // Fetch and print price data

  console.log("Fetching liquidity provider data...");
  await fetchLiquidityProviderData(); // Fetch and print liquidity provider data
})();
