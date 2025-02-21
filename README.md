# pumpbot Rust SDK

A comprehensive Rust SDK for seamless interaction with the PumpFun Solana program. This SDK provides a robust set of tools and interfaces to integrate PumpFun functionality into your applications.

## Usage
Compared to pumpfun-sdk(https://github.com/MiracleAI-Labs/pumpfun-sdk), it adds the ability to create token and batch buy with multiple wallets, all bundled in a single Jito transaction. This ensures that all your wallet purchase transactions are prioritized over any snipers, trading bots, or regular users.

```rust
// Create a new PumpFun client
let rpc_url: &str = "https://api.mainnet-beta.solana.com";
let jito_url: &str = "https://mainnet.block-engine.jito.wtf/api/v1/bundles";

let pumpfun = PumpFun::new(
    rpc_url.to_string(),
    Some(CommitmentConfig::processed()),
    Some(jito_url.to_string()),
);

// Mint keypair
let mint: Keypair = Keypair::new();

// Token metadata
let metadata: CreateTokenMetadata = CreateTokenMetadata {
    name: "Lorem ipsum".to_string(),
    symbol: "LIP".to_string(),
    description: "Lorem ipsum dolor, sit amet consectetur adipisicing elit. Quam, nisi.".to_string(),
    file: "/path/to/image.png".to_string(),
    twitter: None,
    telegram: None,
    website: Some("https://example.com".to_string()),
};

// ${ipfs_api_key} for https://pinata.cloud 
let ipfs_metadata = ipfs::create_token_metadata(metadata, "${ipfs_api_key}").await?;

// random buy amount
let buy_amount_min = sol_to_lamports(0.1);
let buy_amount_max = sol_to_lamports(0.5);
let mut rng = rand::rng();
let amount_sols: Vec<u64> = payers.iter()
    .map(|_| sol_to_lamports(rng.random_range(buy_amount_min..=buy_amount_max)))
    .collect();

let payers: Vec<Keypair> = vec![]; // payers for buy
let payers_ref: Vec<&Keypair> = payers.iter().collect();

// jito fee
let jito_fee = 0.001;

// create and buy with multiple wallets in a single Jito transaction
pumpfun.create_and_buy_list_with_jito(payers_ref, &mint, ipfs_metadata, amount_sols, None, Some(jito_fee)).await?;

// buy with jito
pumpfun.buy_with_jito(payer, &mint, amount_sol, None, Some(jito_fee)).await?;

// sell with jito
pumpfun.sell_with_jito(payer, &mint, amount_token, None, Some(jito_fee)).await?;

// sell by percent with jito
pumpfun.sell_by_percent_with_jito(payer, &mint, percent, None, Some(jito_fee)).await?;

```
