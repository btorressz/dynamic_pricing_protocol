// No imports needed: web3, anchor, pg and more are globally available
describe("Dynamic Pricing Protocol", () => {
  // Create Keypairs for testing
  const priceDataKp = new web3.Keypair(); // Price data account
  const liquidityProviderKp = new web3.Keypair(); // Liquidity provider account

  it("Initialize Price Data", async () => {
    const initialPrice = new BN(100); // Set initial price to 100

    // Send transaction to initialize price data
    const txHash = await pg.program.methods
      .initializePriceData(initialPrice)
      .accounts({
        priceData: priceDataKp.publicKey,
        user: pg.wallet.publicKey, // Use the current wallet
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([priceDataKp])
      .rpc();

    console.log(`Transaction hash: ${txHash}`);

    // Confirm the transaction was successful
    await pg.connection.confirmTransaction(txHash);

    // Fetch the price data from the blockchain
    const priceData = await pg.program.account.priceData.fetch(priceDataKp.publicKey);

    console.log("Initialized Price Data:", priceData);

    // Verify that the price is correctly initialized
    assert(priceData.price.eq(initialPrice), "Price should be initialized to 100");
    assert.equal(priceData.supply, 100, "Supply should be initialized to 100");
    assert.equal(priceData.totalLiquidity, 0, "Total liquidity should be initialized to 0");
  });

  it("Update Price Manually", async () => {
    const newPrice = new BN(150); // Update price to 150

    // Send transaction to update price
    const txHash = await pg.program.methods
      .updatePrice(newPrice)
      .accounts({
        priceData: priceDataKp.publicKey,
        user: pg.wallet.publicKey, // Authorized user
      })
      .rpc();

    console.log(`Transaction hash: ${txHash}`);

    // Confirm the transaction
    await pg.connection.confirmTransaction(txHash);

    // Fetch the updated price data
    const priceData = await pg.program.account.priceData.fetch(priceDataKp.publicKey);

    console.log("Updated Price Data:", priceData);

    // Verify the updated price
    assert(priceData.price.eq(newPrice), "Price should be updated to 150");
  });

  it("Contribute Liquidity", async () => {
    const liquidityAmount = new BN(500); // Amount of liquidity to contribute

    // Send transaction to contribute liquidity
    const txHash = await pg.program.methods
      .contributeLiquidity(liquidityAmount)
      .accounts({
        liquidityProvider: liquidityProviderKp.publicKey,
        priceData: priceDataKp.publicKey,
        user: pg.wallet.publicKey, // Signer providing liquidity
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([liquidityProviderKp])
      .rpc();

    console.log(`Transaction hash: ${txHash}`);

    // Confirm the transaction
    await pg.connection.confirmTransaction(txHash);

    // Fetch the liquidity provider data
    const liquidityProvider = await pg.program.account.liquidityProvider.fetch(liquidityProviderKp.publicKey);
    const priceData = await pg.program.account.priceData.fetch(priceDataKp.publicKey);

    console.log("Liquidity Provider Data:", liquidityProvider);
    console.log("Price Data after Liquidity Contribution:", priceData);

    // Verify that the liquidity was correctly added
    assert(liquidityProvider.liquidity.eq(liquidityAmount), "Liquidity should be 500");
    assert.equal(priceData.totalLiquidity, 500, "Total liquidity should be updated to 500");
  });

  it("Vest Rewards", async () => {
    const rewardAmount = new BN(100); // Amount of rewards to vest

    // Send transaction to vest rewards
    const txHash = await pg.program.methods
      .vestRewards(rewardAmount)
      .accounts({
        liquidityProvider: liquidityProviderKp.publicKey,
        user: pg.wallet.publicKey,
      })
      .rpc();

    console.log(`Transaction hash: ${txHash}`);

    // Confirm the transaction
    await pg.connection.confirmTransaction(txHash);

    // Fetch the updated liquidity provider data
    const liquidityProvider = await pg.program.account.liquidityProvider.fetch(liquidityProviderKp.publicKey);

    console.log("Liquidity Provider Data after Vesting:", liquidityProvider);

    // Verify that rewards were vested correctly
    assert(liquidityProvider.rewards.eq(new BN(0)), "Rewards should be vested");
  });

  // Additional tests can be added here to test other features like:
  // - Fetching price from Oracle
  // - Adjusting price based on supply/demand
  // - Distributing revenue to liquidity providers
  // - Dynamic fees
});
