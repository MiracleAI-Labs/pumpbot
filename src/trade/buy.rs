use anyhow::anyhow;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    compute_budget::ComputeBudgetInstruction, native_token::sol_to_lamports, pubkey::Pubkey, signature::{Keypair, Signature}, signer::Signer, system_instruction, transaction::Transaction
};
use spl_associated_token_account::{
    get_associated_token_address,
    instruction::create_associated_token_account,
};
use std::time::Instant;

use crate::{constants::{self, trade::JITO_TIP_AMOUNT}, instruction};

use super::common::{calculate_with_slippage_buy, get_bonding_curve_account, get_global_account, PriorityFee};

pub async fn build_buy_transaction(
    rpc: &RpcClient,
    payer: &Keypair,
    mint: &Pubkey,
    amount_sol: u64,
    slippage_basis_points: Option<u64>,
    priority_fee: Option<PriorityFee>,
) -> Result<Transaction, anyhow::Error> {
    if amount_sol == 0 {
        return Err(anyhow!("Amount cannot be zero"));
    }

    let global_account = get_global_account(rpc).await?;
    let bonding_curve_account = get_bonding_curve_account(rpc, mint).await?;
    let buy_amount = bonding_curve_account
        .get_buy_price(amount_sol)
        .map_err(|e| anyhow!(e))?;
    let buy_amount_with_slippage = calculate_with_slippage_buy(amount_sol, slippage_basis_points.unwrap_or(0));

    let mut instructions = Vec::new();
    if let Some(fee) = priority_fee {
        if let Some(limit) = fee.limit {
            instructions.push(ComputeBudgetInstruction::set_compute_unit_limit(limit));
        }
        if let Some(price) = fee.price {
            instructions.push(ComputeBudgetInstruction::set_compute_unit_price(price));
        }
    }

    let ata = get_associated_token_address(&payer.pubkey(), mint);
    if rpc.get_account(&ata).is_err() {
        instructions.push(create_associated_token_account(
            &payer.pubkey(),
            &payer.pubkey(),
            mint,
            &constants::accounts::TOKEN_PROGRAM,
        ));
    }

    instructions.push(instruction::buy(
        payer,
        mint,
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
        &[payer],
        recent_blockhash,
    );

    Ok(transaction)
}

pub async fn buy(
    rpc: &RpcClient,
    payer: &Keypair,
    mint: &Pubkey,
    amount_sol: u64,
    slippage_basis_points: Option<u64>,
    priority_fee: Option<PriorityFee>,
) -> Result<Signature, anyhow::Error> {
    if amount_sol == 0 {
        return Err(anyhow!("Amount cannot be zero"));
    }

    let global_account = get_global_account(rpc).await?;
    let bonding_curve_account = get_bonding_curve_account(rpc, mint).await?;
    let buy_amount = bonding_curve_account
        .get_buy_price(amount_sol)
        .map_err(|e| anyhow!(e))?;
    let buy_amount_with_slippage = calculate_with_slippage_buy(amount_sol, slippage_basis_points.unwrap_or(0));

    let mut instructions = Vec::new();
    if let Some(fee) = priority_fee {
        if let Some(limit) = fee.limit {
            instructions.push(ComputeBudgetInstruction::set_compute_unit_limit(limit));
        }
        if let Some(price) = fee.price {
            instructions.push(ComputeBudgetInstruction::set_compute_unit_price(price));
        }
    }

    let ata = get_associated_token_address(&payer.pubkey(), mint);
    if rpc.get_account(&ata).is_err() {
        instructions.push(create_associated_token_account(
            &payer.pubkey(),
            &payer.pubkey(),
            mint,
            &constants::accounts::TOKEN_PROGRAM,
        ));
    }

    instructions.push(instruction::buy(
        payer,
        mint,
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
        &[payer],
        recent_blockhash,
    );

    let signature = rpc.send_transaction(&transaction)?;
    Ok(signature)
}

/// Buy tokens using Jito
pub async fn buy_with_jito(
    rpc: &RpcClient,
    payer: &Keypair,
    jito_client: &crate::jito::JitoClient,
    mint: &Pubkey,
    buy_token_amount: u64,
    max_sol_cost: u64,
    slippage_basis_points: Option<u64>,
    jito_fee: Option<f64>,
) -> Result<String, anyhow::Error> {
    if buy_token_amount == 0 || max_sol_cost == 0 {
        return Err(anyhow!("Amount cannot be zero"));
    }

    let start_time = Instant::now();

    let global_account = get_global_account(rpc).await?;
    let buy_amount_with_slippage = calculate_with_slippage_buy(max_sol_cost, slippage_basis_points.unwrap_or(0));

    let mut instructions = Vec::new();
    let tip_account = jito_client.get_tip_account().await.map_err(|e| anyhow!(e))?;
    let ata = get_associated_token_address(&payer.pubkey(), mint);
    if rpc.get_account(&ata).is_err() {
        instructions.push(create_associated_token_account(
            &payer.pubkey(),
            &payer.pubkey(),
            mint,
            &constants::accounts::TOKEN_PROGRAM,
        ));
    }

    instructions.push(instruction::buy(
        payer,
        mint,
        &global_account.fee_recipient,
        instruction::Buy {
            _amount: buy_token_amount,
            _max_sol_cost: buy_amount_with_slippage,
        },
    ));

    let jito_fee = jito_fee.unwrap_or(JITO_TIP_AMOUNT);
    instructions.push(
        system_instruction::transfer(
            &payer.pubkey(),
            &tip_account,
            sol_to_lamports(jito_fee),
        ),
    );

    let recent_blockhash = rpc.get_latest_blockhash()?;
    let transaction = Transaction::new_signed_with_payer(
        &instructions,
        Some(&payer.pubkey()),
        &[payer],
        recent_blockhash,
    );

    let signature = jito_client.send_transaction(&transaction).await?;
    println!("Total Jito buy operation time: {:?}ms", start_time.elapsed().as_millis());

    Ok(signature)
}
