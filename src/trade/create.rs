use anyhow::anyhow;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    signature::{Keypair, Signature}, signer::Signer, transaction::Transaction
};
use spl_associated_token_account::{
    get_associated_token_address,
    instruction::create_associated_token_account,
};

use crate::{constants, instruction, ipfs::TokenMetadataIPFS, jito::JitoClient};

use super::{buy::build_buy_transaction, common::{create_priority_fee_instructions, get_buy_amount_with_slippage, get_global_account, PriorityFee}};

/// Create a new token
pub async fn create(
    rpc: &RpcClient,
    payer: &Keypair,
    mint: &Keypair,
    ipfs: TokenMetadataIPFS,
    priority_fee: Option<PriorityFee>,
) -> Result<Signature, anyhow::Error> {
    let mut instructions = create_priority_fee_instructions(priority_fee);

    instructions.push(instruction::create(
        payer,
        mint,
        instruction::Create {
            _name: ipfs.metadata.name,
            _symbol: ipfs.metadata.symbol,
            _uri: ipfs.metadata_uri,
        },
    ));

    let recent_blockhash = rpc.get_latest_blockhash()?;
    let transaction = Transaction::new_signed_with_payer(
        &instructions,
        Some(&payer.pubkey()),
        &[payer, mint],
        recent_blockhash,
    );

    let signature = rpc.send_and_confirm_transaction(&transaction)?;

    Ok(signature)
}

/// Create and buy tokens in one transaction
pub async fn create_and_buy(
    rpc: &RpcClient,
    payer: &Keypair,
    mint: &Keypair,
    ipfs: TokenMetadataIPFS,
    amount_sol: u64,
    slippage_basis_points: Option<u64>,
    priority_fee: Option<PriorityFee>,
) -> Result<Signature, anyhow::Error> {
    if amount_sol == 0 {
        return Err(anyhow!("Amount cannot be zero"));
    }

    let global_account = get_global_account(rpc).await?;
    let buy_amount = global_account.get_initial_buy_price(amount_sol);
    let buy_amount_with_slippage =
        get_buy_amount_with_slippage(amount_sol, slippage_basis_points);

    let mut instructions = create_priority_fee_instructions(priority_fee);

    instructions.push(instruction::create(
        payer,
        mint,
        instruction::Create {
            _name: ipfs.metadata.name,
            _symbol: ipfs.metadata.symbol,
            _uri: ipfs.metadata_uri,
        },
    ));

    let ata = get_associated_token_address(&payer.pubkey(), &mint.pubkey());
    if rpc.get_account(&ata).is_err() {
        instructions.push(create_associated_token_account(
            &payer.pubkey(),
            &payer.pubkey(),
            &mint.pubkey(),
            &constants::accounts::TOKEN_PROGRAM,
        ));
    }

    instructions.push(instruction::buy(
        payer,
        &mint.pubkey(),
        &global_account.fee_recipient,
        instruction::Buy {
            _amount: buy_amount,
            _max_sol_cost: buy_amount_with_slippage,
        },
    ));

    let recent_blockhash = rpc.get_latest_blockhash()?;
    let transaction = Transaction::new_signed_with_payer(
        &instructions,
        Some(&payer.pubkey()),
        &[payer, mint],
        recent_blockhash,
    );

    let signature = rpc.send_and_confirm_transaction(&transaction)?;

    Ok(signature)
}

pub async fn create_and_buy_with_jito(
    jito_client: &JitoClient,
    rpc: &RpcClient,
    payers: Vec<&Keypair>,
    mint: &Keypair,
    ipfs: TokenMetadataIPFS,
    amount_sols: Vec<u64>,
) -> Result<(), anyhow::Error> {
    if amount_sols.is_empty() {
        return Err(anyhow!("Amount cannot be zero"));
    }

    let mut transactions = Vec::new();
    let transaction = build_create_and_buy_transaction(rpc, payers[0], mint, ipfs, amount_sols[0], None, None).await?;
    transactions.push(transaction);
    
    for (i, payer) in payers.iter().skip(1).enumerate() {
        let buy_transaction = build_buy_transaction(rpc, payer, &mint.pubkey(), amount_sols[i], None, None).await?;
        transactions.push(buy_transaction);
    }

    jito_client.send_transactions(&transactions).await?;
    
    Ok(())
}

pub async fn build_create_and_buy_transaction(
    rpc: &RpcClient,
    payer: &Keypair,
    mint: &Keypair,
    ipfs: TokenMetadataIPFS,
    amount_sol: u64,
    slippage_basis_points: Option<u64>,
    priority_fee: Option<PriorityFee>,
) -> Result<Transaction, anyhow::Error> {
    if amount_sol == 0 {
        return Err(anyhow!("Amount cannot be zero"));
    }

    let global_account = get_global_account(rpc).await?;
    let buy_amount = global_account.get_initial_buy_price(amount_sol);
    let buy_amount_with_slippage =
        get_buy_amount_with_slippage(amount_sol, slippage_basis_points);

    let mut instructions = create_priority_fee_instructions(priority_fee);

    instructions.push(instruction::create(
        payer,
        mint,
        instruction::Create {
            _name: ipfs.metadata.name,
            _symbol: ipfs.metadata.symbol,
            _uri: ipfs.metadata_uri,
        },
    ));

    let ata = get_associated_token_address(&payer.pubkey(), &mint.pubkey());
    if rpc.get_account(&ata).is_err() {
        instructions.push(create_associated_token_account(
            &payer.pubkey(),
            &payer.pubkey(),
            &mint.pubkey(),
            &constants::accounts::TOKEN_PROGRAM,
        ));
    }

    instructions.push(instruction::buy(
        payer,
        &mint.pubkey(),
        &global_account.fee_recipient,
        instruction::Buy {
            _amount: buy_amount,
            _max_sol_cost: buy_amount_with_slippage,
        },
    ));

    let recent_blockhash = rpc.get_latest_blockhash()?;
    let transaction = Transaction::new_signed_with_payer(
        &instructions,
        Some(&payer.pubkey()),
        &[payer, mint],
        recent_blockhash,
    );

    Ok(transaction)
}
