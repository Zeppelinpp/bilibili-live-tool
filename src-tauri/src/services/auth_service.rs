use crate::models::config::UserConfig;
use crate::models::user::{LoginResult, QrCodeData};
use crate::services::bili_api::BiliApi;
use anyhow::Result;

pub struct AuthService;

impl AuthService {
    pub async fn get_login_qrcode(api: &BiliApi) -> Result<QrCodeData> {
        api.get_passport_qrcode().await
    }

    pub async fn poll_login_status(api: &BiliApi, key: &str) -> Result<LoginResult> {
        let (code, _message, cookies) = api.poll_passport_qrcode(key).await?;
        if code == 0 {
            // Build a temporary user config from cookies
            let cookie_str = cookies
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join("; ");
            let csrf = cookies.get("bili_jct").cloned().unwrap_or_default();
            let uid = cookies
                .get("DedeUserID")
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(0);
            let user = UserConfig {
                uid,
                uname: String::new(),
                face: String::new(),
                cookie: cookie_str,
                room_id: String::new(),
                csrf,
                last_title: String::new(),
                last_area_id: 0,
                last_area_name: vec![],
                level: 0,
                follower: 0,
                following: 0,
                dynamic_count: 0,
            };
            Ok(LoginResult {
                code,
                uid: Some(uid),
                user: Some(user),
            })
        } else {
            Ok(LoginResult {
                code,
                uid: None,
                user: None,
            })
        }
    }
}
