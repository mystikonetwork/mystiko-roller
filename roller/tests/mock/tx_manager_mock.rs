use async_trait::async_trait;
use ethers_core::types::{TransactionReceipt, TxHash, U256};
use ethers_providers::{JsonRpcClient, Provider};
use mockall::mock;
use mystiko_server_utils::tx_manager::{TransactionData, TransactionMiddleware, TransactionMiddlewareResult};

mock! {
    #[derive(Debug)]
    pub RollerTxManager<P>
    where
    P: JsonRpcClient,
    {}

    #[async_trait]
    impl<P> TransactionMiddleware<P> for RollerTxManager<P>
        where
    P: JsonRpcClient,
    {
    fn tx_eip1559(&self) -> bool;
           async fn gas_price(&self, provider: &Provider<P>) -> TransactionMiddlewareResult<U256>;
            async fn estimate_gas(
                &self,
                data: &TransactionData,
                provider: &Provider<P>,
            ) -> TransactionMiddlewareResult<U256>;
            async fn send(&self, data: &TransactionData, provider: &Provider<P>) -> TransactionMiddlewareResult<TxHash>;
            async fn confirm(
                &self,
                tx_hash: &TxHash,
                provider: &Provider<P>,
            ) -> TransactionMiddlewareResult<TransactionReceipt>;

    }

}
