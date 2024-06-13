use async_trait::async_trait;
use hyper::Body;
use mime::Mime;
use mystiko_status_server::Status;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use typed_builder::TypedBuilder;

#[derive(Debug, TypedBuilder)]
pub struct RollerStatusGetter {
    pub status: Arc<RollerStatusWrapper>,
}

#[derive(Debug, TypedBuilder)]
pub struct RollerStatusWrapper {
    pub status: RwLock<RollerStatus>,
}

#[derive(Debug, Clone, TypedBuilder, Serialize, Deserialize)]
pub struct RollerStatus {
    pub action: RollerStatusAction,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum RollerStatusAction {
    Idle,
    Loading,
    Rollup,
}

#[async_trait]
impl Status for RollerStatusGetter {
    async fn status(&self) -> anyhow::Result<(mime::Mime, hyper::Body)> {
        let status = self.status.get_status().await;
        let body = Body::from(serde_json::to_string(&status)?);
        let mime = mime::APPLICATION_JSON;
        Ok((mime, body))
    }
}

#[async_trait]
impl Status for Box<RollerStatusGetter> {
    async fn status(&self) -> anyhow::Result<(Mime, Body)> {
        self.status().await
    }
}

impl RollerStatusWrapper {
    pub async fn new() -> RollerStatusWrapper {
        let status = RollerStatus::builder().action(RollerStatusAction::Idle).build();
        RollerStatusWrapper {
            status: RwLock::new(status),
        }
    }

    pub async fn get_status(&self) -> RollerStatus {
        self.status.read().await.clone()
    }

    pub async fn set_action(&self, action: RollerStatusAction) {
        let mut status = self.status.write().await;
        status.action = action;
    }
}
