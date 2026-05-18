use crate::models::live::{PartitionMap, StreamCodeData, StreamProtocol};
use crate::services::bili_api::BiliApi;
use crate::services::config_store::ConfigStore;
use crate::state::SessionState;
use anyhow::Result;

pub struct LiveService {
    partition_map: PartitionMap,
}

impl LiveService {
    pub fn new() -> Self {
        Self {
            partition_map: PartitionMap::new(),
        }
    }

    pub async fn refresh_partitions(&mut self, api: &BiliApi) -> Result<()> {
        let res = api.get_area_list().await?;
        if res["code"].as_i64().unwrap_or(-1) != 0 {
            return Err(anyhow::anyhow!("获取分区列表失败"));
        }
        let data = res["data"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("分区数据格式错误"))?;
        self.partition_map.clear();
        for area in data {
            let name = area["name"].as_str().unwrap_or("").to_string();
            let mut sub_map = std::collections::HashMap::new();
            if let Some(list) = area["list"].as_array() {
                for sub in list {
                    let sub_name = sub["name"].as_str().unwrap_or("").to_string();
                    let id = sub["id"].as_u64().unwrap_or(0);
                    sub_map.insert(sub_name, id);
                }
            }
            self.partition_map.insert(name, sub_map);
        }
        Ok(())
    }

    pub fn get_partitions(&self) -> PartitionMap {
        self.partition_map.clone()
    }

    pub fn get_area_id(&self, p_name: &str, s_name: &str) -> Option<u64> {
        self.partition_map.get(p_name)?.get(s_name).copied()
    }

    pub async fn start_live(
        &mut self,
        api: &BiliApi,
        session: &mut SessionState,
        config: &mut ConfigStore,
        p_name: Option<String>,
        s_name: Option<String>,
    ) -> Result<StreamCodeData> {
        let room_id = session
            .room_id
            .clone()
            .ok_or_else(|| anyhow::anyhow!("未登录"))?;
        let room_id_num = room_id.parse::<u64>()?;
        let csrf = session
            .csrf
            .clone()
            .ok_or_else(|| anyhow::anyhow!("未获取CSRF"))?;

        let area_id = if let (Some(p), Some(s)) = (p_name, s_name) {
            self.get_area_id(&p, &s).unwrap_or(235)
        } else {
            session.current_area_id.unwrap_or(235)
        };

        let res = api.start_live(room_id_num, area_id, &csrf).await?;
        let code = res["code"].as_i64().unwrap_or(-1);

        if code == 60024 || code == 60043 {
            return Err(anyhow::anyhow!("需要人脸验证"));
        }
        if code != 0 {
            let msg = res["message"].as_str().unwrap_or("开播失败").to_string();
            return Err(anyhow::anyhow!(msg));
        }

        session.is_live = true;
        session.current_area_id = Some(area_id);

        let data = &res["data"];
        let rtmp = &data["rtmp"];
        let protocols = data["protocols"].as_array().cloned().unwrap_or_default();

        let rtmp1 = StreamProtocol {
            addr: rtmp["addr"].as_str().unwrap_or("").to_string(),
            code: rtmp["code"].as_str().unwrap_or("").to_string(),
        };

        let mut rtmp2 = StreamProtocol::default();
        let mut srt = StreamProtocol::default();
        for p in protocols {
            if p["protocol"].as_str() == Some("rtmp") && rtmp2.addr.is_empty() {
                rtmp2.addr = p["addr"].as_str().unwrap_or("").to_string();
                rtmp2.code = p["code"].as_str().unwrap_or("").to_string();
            }
            if p["protocol"].as_str() == Some("srt") && srt.addr.is_empty() {
                srt.addr = p["addr"].as_str().unwrap_or("").to_string();
                srt.code = p["code"].as_str().unwrap_or("").to_string();
            }
        }

        if let Some(uid) = session.uid {
            let uid_str = uid.to_string();
            if let Some(user) = config.data_mut().users.get_mut(&uid_str) {
                user.last_area_id = area_id;
            }
            config.save()?;
        }

        Ok(StreamCodeData { rtmp1, rtmp2, srt })
    }

    pub async fn stop_live(&mut self, api: &BiliApi, session: &mut SessionState) -> Result<()> {
        let room_id = session
            .room_id
            .clone()
            .ok_or_else(|| anyhow::anyhow!("未登录"))?;
        let room_id_num = room_id.parse::<u64>()?;
        let csrf = session
            .csrf
            .clone()
            .ok_or_else(|| anyhow::anyhow!("未获取CSRF"))?;

        let res = api.stop_live(room_id_num, &csrf).await?;
        if res["code"].as_i64().unwrap_or(-1) != 0 {
            return Err(anyhow::anyhow!("停播失败"));
        }
        session.is_live = false;
        Ok(())
    }

    pub async fn update_title(
        api: &BiliApi,
        session: &SessionState,
        config: &mut ConfigStore,
        title: &str,
    ) -> Result<()> {
        let room_id = session
            .room_id
            .clone()
            .ok_or_else(|| anyhow::anyhow!("未登录"))?;
        let room_id_num = room_id.parse::<u64>()?;
        let csrf = session
            .csrf
            .clone()
            .ok_or_else(|| anyhow::anyhow!("未获取CSRF"))?;

        let res = api.update_title(room_id_num, title, &csrf).await?;
        if res["code"].as_i64().unwrap_or(-1) != 0 {
            return Err(anyhow::anyhow!("更新标题失败"));
        }

        if let Some(uid) = session.uid {
            let uid_str = uid.to_string();
            if let Some(user) = config.data_mut().users.get_mut(&uid_str) {
                user.last_title = title.to_string();
            }
            config.save()?;
        }
        Ok(())
    }

    pub async fn update_area(
        &mut self,
        api: &BiliApi,
        session: &mut SessionState,
        config: &mut ConfigStore,
        p_name: &str,
        s_name: &str,
    ) -> Result<()> {
        let area_id = self
            .get_area_id(p_name, s_name)
            .ok_or_else(|| anyhow::anyhow!("无效分区"))?;
        let room_id = session
            .room_id
            .clone()
            .ok_or_else(|| anyhow::anyhow!("未登录"))?;
        let room_id_num = room_id.parse::<u64>()?;
        let csrf = session
            .csrf
            .clone()
            .ok_or_else(|| anyhow::anyhow!("未获取CSRF"))?;

        let res = api.update_area(room_id_num, area_id, &csrf).await?;
        if res["code"].as_i64().unwrap_or(-1) != 0 {
            return Err(anyhow::anyhow!("更新分区失败"));
        }

        session.current_area_id = Some(area_id);
        session.current_area_names = vec![p_name.to_string(), s_name.to_string()];

        if let Some(uid) = session.uid {
            let uid_str = uid.to_string();
            if let Some(user) = config.data_mut().users.get_mut(&uid_str) {
                user.last_area_id = area_id;
                user.last_area_name = vec![p_name.to_string(), s_name.to_string()];
            }
            config.save()?;
        }
        Ok(())
    }
}
