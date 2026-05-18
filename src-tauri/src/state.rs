use std::sync::Mutex;

#[derive(Default)]
pub struct SessionState {
    pub uid: Option<u64>,
    pub room_id: Option<String>,
    pub csrf: Option<String>,
    pub is_live: bool,
    pub current_area_id: Option<u64>,
    pub current_area_names: Vec<String>,
}

pub struct AppState {
    pub config: Mutex<crate::services::config_store::ConfigStore>,
    pub session: Mutex<SessionState>,
    pub api: tokio::sync::Mutex<crate::services::bili_api::BiliApi>,
}
