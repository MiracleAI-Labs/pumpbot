pub mod accounts;
pub mod constants;
pub mod error;
pub mod instruction;
pub mod jito;
pub mod common;
pub mod ipfs;
pub mod trade;

use anyhow::anyhow;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
    signature::{Keypair, Signature},
};

use common::{logs_events::PumpfunEvent, logs_subscribe};
use common::logs_subscribe::SubscriptionHandle;
use ipfs::TokenMetadataIPFS;

use std::sync::Arc;
use crate::jito::JitoClient;
use crate::trade::common::PriorityFee;

pub struct PumpFun {
    pub rpc: RpcClient,
    pub payer: Arc<Keypair>,
    pub jito_client: Option<JitoClient>,
}

impl Clone for PumpFun {
    fn clone(&self) -> Self {
        Self {
            rpc: RpcClient::new_with_commitment(
                self.rpc.url().to_string(),
                self.rpc.commitment()
            ),
            payer: self.payer.clone(),
            jito_client: self.jito_client.clone(),
        }
    }
}

impl PumpFun {
    #[inline]
    pub fn new(
        rpc_url: String,
        commitment: Option<CommitmentConfig>,
        payer: Arc<Keypair>,
        jito_url: Option<String>,
    ) -> Self {
        let rpc = RpcClient::new_with_commitment(
            rpc_url,
            commitment.unwrap_or(CommitmentConfig::processed())
        );   

        let jito_client = jito_url.map(|url| JitoClient::new(&url, None));

        Self {
            rpc,
            payer,
            jito_client,
        }
    }

    /// Create a new token
    pub async fn create(
        &self,
        mint: &Keypair,
        ipfs: TokenMetadataIPFS,
        priority_fee: Option<PriorityFee>,
    ) -> Result<Signature, anyhow::Error> {
        trade::create::create(
            &self.rpc,
            &self.payer,
            mint,
            ipfs,
            priority_fee,
        ).await 
    }

    pub async fn create_and_buy(
        &self,
        mint: &Keypair,
        ipfs: TokenMetadataIPFS,
        amount_sol: u64,
        slippage_basis_points: Option<u64>,
        priority_fee: Option<PriorityFee>,
    ) -> Result<Signature, anyhow::Error> {
        trade::create::create_and_buy(
            &self.rpc,
            &self.payer,
            mint,
            ipfs,
            amount_sol,
            slippage_basis_points,
            priority_fee,
        ).await
    }

    /// Buy tokens
    pub async fn buy(
        &self,
        mint: &Pubkey,
        amount_sol: u64,
        slippage_basis_points: Option<u64>,
        priority_fee: Option<PriorityFee>,
    ) -> Result<Signature, anyhow::Error> {
        trade::buy::buy(
            &self.rpc,
            &self.payer,
            mint,
            amount_sol,
            slippage_basis_points,
            priority_fee,
        ).await
    }

    /// Buy tokens using Jito
    pub async fn buy_with_jito(
        &self,
        mint: &Pubkey,
        buy_token_amount: u64,
        max_sol_cost: u64,
        slippage_basis_points: Option<u64>,
        jito_fee: Option<f64>,
    ) -> Result<String, anyhow::Error> {
        trade::buy::buy_with_jito(
            &self.rpc,
            &self.payer,
            self.jito_client.as_ref().unwrap(),
            mint,
            buy_token_amount,
            max_sol_cost,
            slippage_basis_points,
            jito_fee,
        ).await
    }

    /// Sell tokens
    pub async fn sell(
        &self,
        mint: &Pubkey,
        amount_token: Option<u64>,
        slippage_basis_points: Option<u64>,
        priority_fee: Option<PriorityFee>,
    ) -> Result<(), anyhow::Error> {
        trade::sell::sell(
            &self.rpc,
            &self.payer,
            mint,
            amount_token,
            slippage_basis_points,
            priority_fee,
        ).await
    }

    /// Sell tokens by percentage
    pub async fn sell_by_percent(
        &self,
        mint: &Pubkey,
        percent: u64,
        slippage_basis_points: Option<u64>,
        priority_fee: Option<PriorityFee>,
    ) -> Result<(), anyhow::Error> {
        trade::sell::sell_by_percent(
            &self.rpc,
            &self.payer,
            mint,
            percent,
            slippage_basis_points,
            priority_fee,
        ).await
    }

    pub async fn sell_by_percent_with_jito(
        &self,
        mint: &Pubkey,
        percent: u64,
        slippage_basis_points: Option<u64>,
        jito_fee: Option<f64>,
    ) -> Result<String, anyhow::Error> {
        trade::sell::sell_by_percent_with_jito(
            &self.rpc,
            &self.payer,
            self.jito_client.as_ref().unwrap(),
            mint,
            percent,
            slippage_basis_points,
            jito_fee,
            ).await
    }

    /// Sell tokens using Jito
    pub async fn sell_with_jito(
        &self,
        mint: &Pubkey,
        amount_token: Option<u64>,
        slippage_basis_points: Option<u64>,
        jito_fee: Option<f64>,
    ) -> Result<String, anyhow::Error> {
        let jito_client = self.jito_client.as_ref()
            .ok_or_else(|| anyhow!("Jito client not found"))?;

        trade::sell::sell_with_jito(
            &self.rpc,
            &self.payer,
            jito_client,
            mint,
            amount_token,
            slippage_basis_points,
            jito_fee,
        ).await
    }

    #[inline]
    pub async fn tokens_subscription<F>(
        &self,
        ws_url: &str,
        commitment: CommitmentConfig,
        callback: F,
        bot_wallet: Option<Pubkey>,
    ) -> Result<SubscriptionHandle, Box<dyn std::error::Error>>
    where
        F: Fn(PumpfunEvent) + Send + Sync + 'static,
    {
        logs_subscribe::tokens_subscription(ws_url, commitment, callback, bot_wallet).await
    }

    #[inline]
    pub async fn stop_subscription(&self, subscription_handle: SubscriptionHandle) {
        subscription_handle.shutdown().await;
    }
}
