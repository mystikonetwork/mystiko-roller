use crate::common::{LiteDataChainDataLoader, LiteDataLoaderConfigOptions, RollerResult};
use crate::context::RollerContext;
use crate::loader::ChainDataLoader;
use anyhow::Result;
use async_trait::async_trait;
use mystiko_dataloader::loader::{DataLoader, FromConfig, LoadOption};
use std::sync::Arc;
use typed_builder::TypedBuilder;

#[derive(TypedBuilder)]
#[builder(field_defaults(setter(into)))]
pub struct RollerChainDataLoader {
    loader: Arc<LiteDataChainDataLoader>,
}

#[async_trait]
impl ChainDataLoader for RollerChainDataLoader {
    async fn load(&self) -> RollerResult<()> {
        let load_options = LoadOption::default();
        self.loader.load(load_options).await?;
        Ok(())
    }
}

impl RollerChainDataLoader {
    pub async fn from_config(context: Arc<RollerContext>) -> Result<Self> {
        let options = LiteDataLoaderConfigOptions::builder()
            .config(context.config.loader.config.clone())
            .mystiko_config(context.mystiko_config.clone())
            .providers(context.providers.clone())
            .handler(context.handler.clone())
            .chain_id(context.config.chain_id)
            .build();
        let loader = LiteDataChainDataLoader::from_config(&options).await?;
        Ok(Self::builder().loader(loader).build())
    }
}
