use async_trait::async_trait;
use ethers_core::types::U256;
use mockall::mock;
use mystiko_server_utils::token_price::{PriceMiddleware, PriceMiddlewareResult};

mock! {
    #[derive(Debug)]
    pub RollerTokenPrice{}

    #[async_trait]
    impl PriceMiddleware for RollerTokenPrice{
        async fn price(&self, symbol: &str) -> PriceMiddlewareResult<f64>;
        async fn swap(
            &self,
            asset_a: &str,
            decimal_a: u32,
            amount_a: U256,
            asset_b: &str,
            decimal_b: u32,
        ) -> PriceMiddlewareResult<U256>;
    }

}
