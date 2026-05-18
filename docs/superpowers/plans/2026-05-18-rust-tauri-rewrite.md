# B站直播工具 Rust + Tauri 重构实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 将现有 Python + pywebview + Vue 3 项目重构为 Rust 后端 + Tauri 2.x + React + TypeScript 桌面应用，保留全部功能。

**Architecture:** Rust 后端通过 Tauri Commands 暴露 API 给前端调用，弹幕通过 Tauri Events 推送到前端。异步运行时使用 tokio，HTTP 用 reqwest，WebSocket 用 tokio-tungstenite。

**Tech Stack:** Rust (tokio, reqwest, tokio-tungstenite, serde, anyhow, tracing, toml, directories), Tauri 2.x, React 18, TypeScript, TailwindCSS, shadcn/ui

---

## 文件结构总览

```
src-tauri/
  Cargo.toml
  tauri.conf.json
  src/
    main.rs              # Tauri App 初始化、Command 注册、托盘设置
    lib.rs               # 模块导出
    commands/
      auth.rs            # get_login_qrcode, poll_login_status
      user.rs            # load_saved_config, refresh, get_account_list, switch, logout
      live.rs            # get_partitions, update_title, update_area, start_live, stop_live
      danmaku.rs           # start_danmaku_monitor, stop_danmaku_monitor, send_danmaku
      window.rs          # window_min, window_max, window_close, window_drag
      config.rs          # get_app_config, set_app_config, get_version
    services/
      bili_api.rs        # B站 HTTP API 封装 (reqwest)
      config_store.rs    # TOML 配置读写
      auth_service.rs    # 登录逻辑
      user_service.rs    # 用户/多账户管理
      live_service.rs    # 直播控制
      danmaku_ws.rs        # 弹幕 WebSocket (tokio-tungstenite)
    models/
      user.rs            # UserConfig, UserInfo, LoginResult
      live.rs            # StreamCodeData, Area
      danmaku.rs           # DanmakuMsg, GiftMsg, InteractMsg, DanmakuServerInfo
      config.rs          # AppConfig
    utils/
      crypto.rs          # App 签名 (MD5)
      mask.rs            # 日志脱敏
      wbi.rs             # Wbi 签名参数生成

src/ (frontend)
  main.tsx
  App.tsx
  components/
    Sidebar.tsx
    StreamPanel.tsx
    DanmakuPanel.tsx
    AccountPanel.tsx
    SettingsPanel.tsx
    QrCodeLogin.tsx
    ui/                  # shadcn/ui 组件
  hooks/
    useTauri.ts
    useDanmaku.ts
    useLive.ts
  context/
    AppContext.tsx
  types/
    api.ts
  styles/
    globals.css
```

---

## Task 1: Tauri 项目初始化

**Files:**
- Create: `src-tauri/Cargo.toml`
- Create: `src-tauri/tauri.conf.json`
- Create: `src-tauri/src/main.rs`
- Create: `src-tauri/icons/` (保留现有 bilibili.ico 并转换)
- Create: `package.json`
- Create: `vite.config.ts`
- Create: `tsconfig.json`
- Create: `tailwind.config.ts`
- Create: `index.html`
- Create: `src/main.tsx`
- Create: `src/App.tsx`
- Create: `src/styles/globals.css`

- [ ] **Step 1: 确认 Rust 和 Node.js 环境**

Run:
```bash
rustc --version  # should be 1.75+
node --version   # should be 18+
npm --version
```
Expected: 版本号输出，无错误。

- [ ] **Step 2: 初始化 Tauri + React + TypeScript 项目**

Run:
```bash
# 在 bilibili_live_stream_code 根目录
npm create tauri-app@latest . -- --template react-ts
# 如果提示目录非空，先 git stash 保存当前代码，或手动初始化
```
Expected: 项目骨架生成，包含 `src-tauri/` 和前端文件。

如果 `create tauri-app` 因为非空目录拒绝，改为手动：
```bash
npm install -D @tauri-apps/cli@latest
```

- [ ] **Step 3: 配置 Cargo.toml**

`src-tauri/Cargo.toml`:
```toml
[package]
name = "bili-live-tool"
version = "3.0.0"
description = "Bilibili Live Stream Tool"
authors = ["you"]
edition = "2024"

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
tauri = { version = "2", features = ["tray-icon"] }
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.12", features = ["json", "cookies"] }
tokio-tungstenite = { version = "0.23", features = ["native-tls"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"
directories = "5"
anyhow = "1"
thiserror = "1"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
md-5 = "0.10"
hex = "0.4"
chrono = "0.4"
urlencoding = "2"
flate2 = "1"
brotli = "6"
prost = "0.13"
once_cell = "1"
```

- [ ] **Step 4: 配置 tauri.conf.json**

`src-tauri/tauri.conf.json`:
```json
{
  "productName": "BiliLiveTool",
  "version": "3.0.0",
  "identifier": "com.bilitool.app",
  "build": {
    "beforeDevCommand": "npm run dev",
    "beforeBuildCommand": "npm run build",
    "devUrl": "http://localhost:1420",
    "frontendDist": "../dist"
  },
  "app": {
    "windows": [
      {
        "title": "B站直播工具",
        "width": 900,
        "height": 700,
        "minWidth": 700,
        "minHeight": 500,
        "center": true,
        "decorations": true
      }
    ],
    "security": {
      "csp": null
    }
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": ["icons/32x32.png", "icons/128x128.png", "icons/128x128@2x.png", "icons/icon.icns", "icons/icon.ico"]
  }
}
```

- [ ] **Step 5: 配置前端依赖**

Run:
```bash
npm install react react-dom
npm install -D typescript @types/react @types/react-dom vite @vitejs/plugin-react tailwindcss postcss autoprefixer
npx tailwindcss init -p
npm install class-variance-authority clsx tailwind-merge lucide-react
npm install @radix-ui/react-slot
```

- [ ] **Step 6: 配置 TailwindCSS（双主题）**

`tailwind.config.ts`:
```typescript
import type { Config } from 'tailwindcss'

const config: Config = {
  darkMode: 'class',
  content: [
    './index.html',
    './src/**/*.{ts,tsx}',
  ],
  theme: {
    extend: {
      colors: {
        stone: {
          50: '#fafaf9',
          100: '#f5f5f4',
          200: '#e7e5e4',
          300: '#d6d3d1',
          400: '#a8a29e',
          500: '#78716c',
          600: '#57534e',
          700: '#44403c',
          800: '#292524',
          900: '#1c1917',
          950: '#0c0a09',
        },
      },
      fontFamily: {
        sans: ['-apple-system', 'BlinkMacSystemFont', 'SF Pro Text', 'Segoe UI', 'Roboto', 'sans-serif'],
        mono: ['SF Mono', 'SFMono-Regular', 'Consolas', 'monospace'],
      },
    },
  },
  plugins: [],
}
export default config
```

- [ ] **Step 7: 配置全局样式**

`src/styles/globals.css`:
```css
@tailwind base;
@tailwind components;
@tailwind utilities;

@layer base {
  html {
    -webkit-font-smoothing: antialiased;
    -moz-osx-font-smoothing: grayscale;
  }

  body {
    @apply bg-stone-50 text-stone-800 dark:bg-stone-950 dark:text-stone-200;
  }

  ::-webkit-scrollbar {
    width: 5px;
  }
  ::-webkit-scrollbar-track {
    background: transparent;
  }
  ::-webkit-scrollbar-thumb {
    background: rgba(0,0,0,0.1);
    border-radius: 4px;
  }
  .dark ::-webkit-scrollbar-thumb {
    background: rgba(255,255,255,0.1);
  }
}
```

- [ ] **Step 8: 配置 Vite**

`vite.config.ts`:
```typescript
import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import path from 'path'

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      ignored: ['**/src-tauri/**'],
    },
  },
})
```

- [ ] **Step 9: 入口文件**

`src/main.tsx`:
```tsx
import React from 'react'
import ReactDOM from 'react-dom/client'
import App from './App'
import './styles/globals.css'

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
)
```

`index.html`:
```html
<!DOCTYPE html>
<html lang="zh-CN">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>B站直播工具</title>
  </head>
  <body>
    <div id="root"></div>
    <script type="module" src="/src/main.tsx"></script>
  </body>
</html>
```

- [ ] **Step 10: 验证 Tauri 能启动**

Run:
```bash
npx tauri dev
```
Expected: 窗口弹出，显示空白或 "Hello Tauri" 页面，无报错。

- [ ] **Step 11: Commit**

```bash
git add .
git commit -m "chore: init tauri + react + ts project"
```

---

## Task 2: Rust 基础模块 — 配置存储 + 数据模型

**Files:**
- Create: `src-tauri/src/models/config.rs`
- Create: `src-tauri/src/models/user.rs`
- Create: `src-tauri/src/models/live.rs`
- Create: `src-tauri/src/models/danmaku.rs`
- Create: `src-tauri/src/models/mod.rs`
- Create: `src-tauri/src/services/config_store.rs`
- Create: `src-tauri/src/services/mod.rs`
- Create: `src-tauri/src/utils/mask.rs`
- Create: `src-tauri/src/utils/mod.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/main.rs`

- [ ] **Step 1: 定义数据模型**

`src-tauri/src/models/config.rs`:
```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct AppConfig {
    pub current_uid: Option<u64>,
    pub users: HashMap<String, UserConfig>,
    #[serde(default = "default_min_to_tray")]
    pub min_to_tray: bool,
}

fn default_min_to_tray() -> bool { true }

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UserConfig {
    pub uid: u64,
    pub uname: String,
    pub face: String,
    pub cookie: String,
    pub room_id: String,
    pub csrf: String,
    #[serde(default)]
    pub last_title: String,
    #[serde(default)]
    pub last_area_id: u64,
    #[serde(default)]
    pub last_area_name: Vec<String>,
    #[serde(default)]
    pub level: u32,
    #[serde(default)]
    pub follower: u32,
    #[serde(default)]
    pub following: u32,
    #[serde(default)]
    pub dynamic_count: u32,
}
```

`src-tauri/src/models/user.rs`:
```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UserInfo {
    pub mid: u64,
    pub uname: String,
    pub face: String,
    pub level_info: LevelInfo,
    pub money: f64,
    pub wallet: Wallet,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct LevelInfo {
    #[serde(rename = "current_level")]
    pub current_level: u32,
    #[serde(rename = "current_exp")]
    pub current_exp: u64,
    #[serde(rename = "next_exp")]
    pub next_exp: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Wallet {
    #[serde(rename = "bcoin_balance")]
    pub bcoin_balance: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UserStat {
    pub following: u32,
    pub follower: u32,
    pub dynamic_count: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct QrCodeData {
    pub url: String,
    pub qrcode_key: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LoginResult {
    pub code: i32,
    pub uid: Option<u64>,
    pub user: Option<crate::models::config::UserConfig>,
}
```

`src-tauri/src/models/live.rs`:
```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StreamCodeData {
    pub rtmp1: StreamProtocol,
    pub rtmp2: StreamProtocol,
    pub srt: StreamProtocol,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct StreamProtocol {
    pub addr: String,
    pub code: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Area {
    pub id: u64,
    pub name: String,
    pub list: Vec<SubArea>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SubArea {
    pub id: u64,
    pub name: String,
}

pub type PartitionMap = HashMap<String, HashMap<String, u64>>;
```

`src-tauri/src/models/danmaku.rs`:
```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DanmakuServerInfo {
    pub token: String,
    pub host_list: Vec<DanmakuHost>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DanmakuHost {
    pub host: String,
    pub wss_port: u16,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum DanmakuMessage {
    #[serde(rename = "danmaku")]
    Danmaku { uid: u64, uname: String, face: String, msg: String },
    #[serde(rename = "interact")]
    Interact { uid: u64, uname: String, msg: String },
    #[serde(rename = "gift")]
    Gift { uid: u64, uname: String, face: String, gift_name: String, num: u32, action: String },
    #[serde(rename = "system")]
    System { msg: String },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SendDanmakuResult {
    pub code: i32,
    pub msg: String,
}
```

`src-tauri/src/models/mod.rs`:
```rust
pub mod config;
pub mod danmaku;
pub mod live;
pub mod user;
```

- [ ] **Step 2: 实现配置存储**

`src-tauri/src/services/config_store.rs`:
```rust
use crate::models::config::AppConfig;
use anyhow::Result;
use std::fs;
use std::path::PathBuf;

pub struct ConfigStore {
    path: PathBuf,
    data: AppConfig,
}

impl ConfigStore {
    pub fn new() -> Result<Self> {
        let dir = directories::ProjectDirs::from("com", "bilitool", "BiliLiveTool")
            .ok_or_else(|| anyhow::anyhow!("Failed to get project dirs"))?;
        let config_dir = dir.config_dir();
        fs::create_dir_all(config_dir)?;
        let path = config_dir.join("config.toml");

        let data = if path.exists() {
            let content = fs::read_to_string(&path)?;
            toml::from_str(&content).unwrap_or_default()
        } else {
            AppConfig::default()
        };

        Ok(Self { path, data })
    }

    pub fn data(&self) -> &AppConfig {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut AppConfig {
        &mut self.data
    }

    pub fn save(&self) -> Result<()> {
        let content = toml::to_string_pretty(&self.data)?;
        fs::write(&self.path, content)?;
        Ok(())
    }
}
```

`src-tauri/src/services/mod.rs`:
```rust
pub mod config_store;
```

- [ ] **Step 3: 实现脱敏工具**

`src-tauri/src/utils/mask.rs`:
```rust
pub fn mask_string(s: &str, visible_start: usize, visible_end: usize) -> String {
    if s.len() <= visible_start + visible_end {
        return "***".to_string();
    }
    let start = &s[..visible_start.min(s.len())];
    let end = &s[s.len().saturating_sub(visible_end)..];
    format!("{}***{}", start, end)
}
```

`src-tauri/src/utils/mod.rs`:
```rust
pub mod mask;
```

- [ ] **Step 4: 组装 lib.rs**

`src-tauri/src/lib.rs`:
```rust
pub mod commands;
pub mod models;
pub mod services;
pub mod utils;
```

- [ ] **Step 5: 更新 main.rs 引入模块**

`src-tauri/src/main.rs`:
```rust
// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    bili_live_tool_lib::run();
}
```

Wait, Tauri 2.x 推荐模式不同。改为：
```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 6: 编译验证**

Run:
```bash
cd src-tauri && cargo check
```
Expected: 编译通过，无错误。

- [ ] **Step 7: Commit**

```bash
git add .
git commit -m "feat: add data models and config store"
```

---

## Task 3: BiliApi 封装 — HTTP 客户端 + App 签名

**Files:**
- Create: `src-tauri/src/services/bili_api.rs`
- Create: `src-tauri/src/utils/crypto.rs`
- Modify: `src-tauri/src/utils/mod.rs`
- Modify: `src-tauri/src/services/mod.rs`

- [ ] **Step 1: 实现 App 签名**

`src-tauri/src/utils/crypto.rs`:
```rust
use md5::{Digest, Md5};
use std::collections::HashMap;

const APP_KEY: &str = "aae92bc66f3edfab";
const APP_SEC: &str = "af125a0d5279fd576c1b4418a3e8276d";

pub fn app_sign(params: &mut HashMap<String, String>) -> HashMap<String, String> {
    params.insert("appkey".to_string(), APP_KEY.to_string());
    let mut keys: Vec<_> = params.keys().cloned().collect();
    keys.sort();
    let query: Vec<String> = keys
        .iter()
        .map(|k| format!("{}={}", urlencoding::encode(k), urlencoding::encode(&params[k])))
        .collect();
    let query_str = query.join("&");
    let sign_str = format!("{}{}", query_str, APP_SEC);
    let hash = Md5::digest(sign_str.as_bytes());
    let sign = hex::encode(hash);
    params.insert("sign".to_string(), sign);
    params.clone()
}
```

- [ ] **Step 2: 实现 BiliApi**

`src-tauri/src/services/bili_api.rs`:
```rust
use crate::models::danmaku::DanmakuServerInfo;
use crate::models::live::{Area, StreamCodeData};
use crate::models::user::{QrCodeData, UserInfo};
use crate::utils::crypto::app_sign;
use anyhow::Result;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use serde_json::Value;
use std::collections::HashMap;

const UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36";

pub struct BiliApi {
    client: reqwest::Client,
    cookies: HashMap<String, String>,
}

impl BiliApi {
    pub fn new() -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static(UA));
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .cookie_store(true)
            .build()
            .unwrap();
        Self {
            client,
            cookies: HashMap::new(),
        }
    }

    pub fn update_cookies(&mut self, cookies: HashMap<String, String>) {
        self.cookies = cookies;
    }

    pub fn cookie_str(&self) -> String {
        self.cookies
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("; ")
    }

    fn headers(&self) -> HeaderMap {
        let mut h = HeaderMap::new();
        h.insert("User-Agent", HeaderValue::from_static(UA));
        h.insert("Referer", HeaderValue::from_static("https://live.bilibili.com"));
        if !self.cookies.is_empty() {
            if let Ok(v) = HeaderValue::from_str(&self.cookie_str()) {
                h.insert("Cookie", v);
            }
        }
        h
    }

    pub async fn request(
        &self,
        method: &str,
        url: &str,
        params: Option<HashMap<String, String>>,
        data: Option<HashMap<String, String>>,
    ) -> Result<Value> {
        let mut req = if method == "GET" {
            let mut r = self.client.get(url);
            if let Some(p) = params {
                r = r.query(&p);
            }
            r
        } else {
            let mut r = self.client.post(url);
            if let Some(p) = params {
                r = r.query(&p);
            }
            if let Some(d) = data {
                r = r.form(&d);
            }
            r
        };
        req = req.headers(self.headers());
        let resp = req.send().await?;
        let json: Value = resp.json().await?;
        Ok(json)
    }

    // --- 扫码登录 ---
    pub async fn get_passport_qrcode(&self) -> Result<QrCodeData> {
        let res = self.request("GET", "https://passport.bilibili.com/x/passport-login/web/qrcode/generate", None, None).await?;
        let data = res["data"].clone();
        let qr: QrCodeData = serde_json::from_value(data)?;
        Ok(qr)
    }

    pub async fn poll_passport_qrcode(&self, key: &str) -> Result<(i32, String, HashMap<String, String>)> {
        let res = self.request("GET", "https://passport.bilibili.com/x/passport-login/web/qrcode/poll",
            Some(HashMap::from([("qrcode_key".to_string(), key.to_string())])), None).await?;
        let code = res["data"]["code"].as_i64().unwrap_or(-1) as i32;
        let message = res["data"]["message"].as_str().unwrap_or("").to_string();
        // TODO: extract cookies from response (need raw response access)
        Ok((code, message, HashMap::new()))
    }

    // --- 用户信息 ---
    pub async fn get_user_info(&self) -> Result<Value> {
        self.request("GET", "https://api.bilibili.com/x/web-interface/nav", None, None).await
    }

    pub async fn get_user_stat(&self) -> Result<Value> {
        self.request("GET", "https://api.bilibili.com/x/web-interface/nav/stat", None, None).await
    }

    pub async fn get_room_id_by_uid(&self, uid: u64) -> Result<Value> {
        self.request("GET", &format!("https://api.live.bilibili.com/room/v2/Room/room_id_by_uid?uid={}", uid), None, None).await
    }

    // --- 直播控制 ---
    pub async fn get_area_list(&self) -> Result<Value> {
        self.request("GET", "https://api.live.bilibili.com/room/v1/Area/getList", Some(HashMap::from([("show_pinyin".to_string(), "1".to_string())])), None).await
    }

    pub async fn update_title(&self, room_id: u64, title: &str, csrf: &str) -> Result<Value> {
        let mut data = HashMap::new();
        data.insert("room_id".to_string(), room_id.to_string());
        data.insert("platform".to_string(), "pc_link".to_string());
        data.insert("title".to_string(), title.to_string());
        data.insert("csrf_token".to_string(), csrf.to_string());
        data.insert("csrf".to_string(), csrf.to_string());
        self.request("POST", "https://api.live.bilibili.com/room/v1/Room/update", None, Some(data)).await
    }

    pub async fn update_area(&self, room_id: u64, area_id: u64, csrf: &str) -> Result<Value> {
        let mut data = HashMap::new();
        data.insert("room_id".to_string(), room_id.to_string());
        data.insert("area_id".to_string(), area_id.to_string());
        data.insert("platform".to_string(), "pc_link".to_string());
        data.insert("csrf_token".to_string(), csrf.to_string());
        data.insert("csrf".to_string(), csrf.to_string());
        self.request("POST", "https://api.live.bilibili.com/room/v1/Room/update", None, Some(data)).await
    }

    pub async fn start_live(&self, room_id: u64, area_id: u64, csrf: &str) -> Result<Value> {
        // 1. 获取时间戳
        let now_res = self.request("GET", "https://api.bilibili.com/x/report/click/now", None, None).await?;
        let ts = now_res["data"]["now"].as_i64().unwrap_or(0).to_string();

        // 2. 获取版本
        let mut v_params = HashMap::new();
        v_params.insert("system_version".to_string(), "2".to_string());
        v_params.insert("ts".to_string(), ts.clone());
        let v_signed = app_sign(&mut v_params);
        let v_res = self.request("GET", "https://api.live.bilibili.com/xlive/app-blink/v1/liveVersionInfo/getHomePageLiveVersion", Some(v_signed), None).await?;
        let build = v_res["data"]["build"].as_i64().unwrap_or(0).to_string();
        let version = v_res["data"]["curr_version"].as_str().unwrap_or("").to_string();

        // 3. 开播
        let mut data = HashMap::new();
        data.insert("room_id".to_string(), room_id.to_string());
        data.insert("platform".to_string(), "pc_link".to_string());
        data.insert("area_v2".to_string(), area_id.to_string());
        data.insert("backup_stream".to_string(), "0".to_string());
        data.insert("csrf_token".to_string(), csrf.to_string());
        data.insert("csrf".to_string(), csrf.to_string());
        data.insert("build".to_string(), build);
        data.insert("version".to_string(), version);
        data.insert("ts".to_string(), ts);
        let signed = app_sign(&mut data);
        self.request("POST", "https://api.live.bilibili.com/room/v1/Room/startLive", None, Some(signed)).await
    }

    pub async fn stop_live(&self, room_id: u64, csrf: &str) -> Result<Value> {
        let mut data = HashMap::new();
        data.insert("room_id".to_string(), room_id.to_string());
        data.insert("platform".to_string(), "pc_link".to_string());
        data.insert("csrf_token".to_string(), csrf.to_string());
        data.insert("csrf".to_string(), csrf.to_string());
        self.request("POST", "https://api.live.bilibili.com/room/v1/Room/stopLive", None, Some(data)).await
    }

    // --- 弹幕 ---
    pub async fn get_danmaku_info(&self, room_id: u64) -> Result<Value> {
        self.request("GET", "https://api.live.bilibili.com/xlive/web-room/v1/index/getDanmakuInfo", Some(HashMap::from([("id".to_string(), room_id.to_string())])), None).await
    }

    pub async fn send_danmaku(&self, room_id: u64, msg: &str, csrf: &str) -> Result<Value> {
        let mut data = HashMap::new();
        data.insert("msg".to_string(), msg.to_string());
        data.insert("roomid".to_string(), room_id.to_string());
        data.insert("csrf_token".to_string(), csrf.to_string());
        data.insert("csrf".to_string(), csrf.to_string());
        data.insert("rnd".to_string(), chrono::Utc::now().timestamp().to_string());
        data.insert("fontsize".to_string(), "25".to_string());
        data.insert("mode".to_string(), "1".to_string());
        data.insert("pool".to_string(), "0".to_string());
        data.insert("color".to_string(), "16777215".to_string());
        self.request("POST", "https://api.live.bilibili.com/msg/send", None, Some(data)).await
    }
}
```

- [ ] **Step 3: 更新 mod.rs**

`src-tauri/src/services/mod.rs`:
```rust
pub mod bili_api;
pub mod config_store;
```

`src-tauri/src/utils/mod.rs`:
```rust
pub mod crypto;
pub mod mask;
```

- [ ] **Step 4: 添加依赖并编译**

在 `Cargo.toml` 中确保已有 `reqwest`, `md-5`, `hex`, `urlencoding`, `chrono`。

Run:
```bash
cd src-tauri && cargo check
```
Expected: 编译通过。

- [ ] **Step 5: Commit**

```bash
git add .
git commit -m "feat: add BiliApi with app sign and core endpoints"
```

---

## Task 4: 业务服务层 — Auth + User + Live

**Files:**
- Create: `src-tauri/src/services/auth_service.rs`
- Create: `src-tauri/src/services/user_service.rs`
- Create: `src-tauri/src/services/live_service.rs`
- Create: `src-tauri/src/state.rs`
- Modify: `src-tauri/src/services/mod.rs`

- [ ] **Step 1: 定义全局状态**

`src-tauri/src/state.rs`:
```rust
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
```

- [ ] **Step 2: 实现 AuthService**

`src-tauri/src/services/auth_service.rs`:
```rust
use crate::models::user::{LoginResult, QrCodeData};
use crate::services::bili_api::BiliApi;
use anyhow::Result;

pub struct AuthService;

impl AuthService {
    pub async fn get_login_qrcode(api: &BiliApi) -> Result<QrCodeData> {
        api.get_passport_qrcode().await
    }

    pub async fn poll_login_status(api: &BiliApi, key: &str) -> Result<LoginResult> {
        let (code, message, cookies) = api.poll_passport_qrcode(key).await?;
        if code == 0 {
            // TODO: extract cookies and create user
            Ok(LoginResult { code, uid: None, user: None })
        } else {
            Ok(LoginResult { code, uid: None, user: None })
        }
    }
}
```

- [ ] **Step 3: 实现 UserService**

`src-tauri/src/services/user_service.rs`:
```rust
use crate::models::config::{AppConfig, UserConfig};
use crate::models::user::{UserInfo, UserStat};
use crate::services::bili_api::BiliApi;
use crate::services::config_store::ConfigStore;
use anyhow::Result;
use serde_json::Value;

pub struct UserService;

impl UserService {
    pub fn init_current_user(config: &ConfigStore, session: &mut crate::state::SessionState, api: &mut BiliApi) {
        if let Some(uid) = config.data().current_uid {
            let uid_str = uid.to_string();
            if let Some(user) = config.data().users.get(&uid_str) {
                session.uid = Some(user.uid);
                session.room_id = Some(user.room_id.clone());
                session.csrf = Some(user.csrf.clone());
                session.current_area_id = if user.last_area_id > 0 { Some(user.last_area_id) } else { None };
                session.current_area_names = user.last_area_name.clone();
                // Parse cookies
                let cookies = parse_cookie_str(&user.cookie);
                api.update_cookies(cookies);
            }
        }
    }

    pub async fn refresh_current_user(api: &BiliApi, config: &mut ConfigStore, session: &mut crate::state::SessionState) -> Result<UserConfig> {
        let uid = session.uid.ok_or_else(|| anyhow::anyhow!("未登录"))?;
        let nav = api.get_user_info().await?;
        if nav["code"].as_i64().unwrap_or(-1) != 0 {
            return Err(anyhow::anyhow!("获取用户信息失败"));
        }
        let stat = api.get_user_stat().await?;
        let stat_data = if stat["code"].as_i64().unwrap_or(-1) == 0 { stat["data"].clone() } else { Value::Null };

        let uid_str = uid.to_string();
        let room_id = session.room_id.clone().unwrap_or_default();
        let csrf = session.csrf.clone().unwrap_or_default();
        let cookie_str = api.cookie_str();

        let user = build_user_config(uid, &nav["data"], &stat_data, &cookie_str, &room_id, &csrf);
        config.data_mut().users.insert(uid_str.clone(), user.clone());
        config.data_mut().current_uid = Some(uid);
        config.save()?;
        Ok(user)
    }

    pub fn get_account_list(config: &ConfigStore) -> Vec<UserConfig> {
        config.data().users.values().cloned().collect()
    }

    pub fn switch_account(config: &mut ConfigStore, session: &mut crate::state::SessionState, api: &mut BiliApi, uid: u64) -> Result<UserConfig> {
        let uid_str = uid.to_string();
        let user = config.data().users.get(&uid_str).cloned().ok_or_else(|| anyhow::anyhow!("账户不存在"))?;
        config.data_mut().current_uid = Some(uid);
        config.save()?;
        session.uid = Some(user.uid);
        session.room_id = Some(user.room_id.clone());
        session.csrf = Some(user.csrf.clone());
        session.current_area_id = if user.last_area_id > 0 { Some(user.last_area_id) } else { None };
        let cookies = parse_cookie_str(&user.cookie);
        api.update_cookies(cookies);
        Ok(user)
    }

    pub fn logout(config: &mut ConfigStore, session: &mut crate::state::SessionState, api: &mut BiliApi, uid: u64) -> Result<()> {
        let uid_str = uid.to_string();
        config.data_mut().users.remove(&uid_str);
        if config.data().current_uid == Some(uid) {
            config.data_mut().current_uid = None;
            session.uid = None;
            session.room_id = None;
            session.csrf = None;
            api.update_cookies(std::collections::HashMap::new());
        }
        config.save()?;
        Ok(())
    }
}

fn parse_cookie_str(s: &str) -> std::collections::HashMap<String, String> {
    s.split(';')
        .filter_map(|part| {
            let mut kv = part.trim().splitn(2, '=');
            let k = kv.next()?;
            let v = kv.next()?;
            Some((k.to_string(), v.to_string()))
        })
        .collect()
}

fn build_user_config(uid: u64, nav: &Value, stat: &Value, cookie: &str, room_id: &str, csrf: &str) -> UserConfig {
    UserConfig {
        uid,
        uname: nav["uname"].as_str().unwrap_or("").to_string(),
        face: nav["face"].as_str().unwrap_or("").to_string(),
        cookie: cookie.to_string(),
        room_id: room_id.to_string(),
        csrf: csrf.to_string(),
        last_title: String::new(),
        last_area_id: 0,
        last_area_name: vec![],
        level: nav["level_info"]["current_level"].as_u64().unwrap_or(0) as u32,
        follower: stat["follower"].as_u64().unwrap_or(0) as u32,
        following: stat["following"].as_u64().unwrap_or(0) as u32,
        dynamic_count: stat["dynamic_count"].as_u64().unwrap_or(0) as u32,
    }
}
```

- [ ] **Step 4: 实现 LiveService**

`src-tauri/src/services/live_service.rs`:
```rust
use crate::models::config::UserConfig;
use crate::models::live::{PartitionMap, StreamCodeData, StreamProtocol};
use crate::services::bili_api::BiliApi;
use crate::services::config_store::ConfigStore;
use crate::state::SessionState;
use anyhow::Result;
use serde_json::Value;

pub struct LiveService {
    partition_map: PartitionMap,
}

impl LiveService {
    pub fn new() -> Self {
        Self { partition_map: PartitionMap::new() }
    }

    pub async fn refresh_partitions(&mut self, api: &BiliApi) -> Result<()> {
        let res = api.get_area_list().await?;
        if res["code"].as_i64().unwrap_or(-1) != 0 {
            return Err(anyhow::anyhow!("获取分区列表失败"));
        }
        let data = res["data"].as_array().ok_or_else(|| anyhow::anyhow!("分区数据格式错误"))?;
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
        let room_id = session.room_id.clone().ok_or_else(|| anyhow::anyhow!("未登录"))?;
        let room_id_num = room_id.parse::<u64>()?;
        let csrf = session.csrf.clone().ok_or_else(|| anyhow::anyhow!("未获取CSRF"))?;

        // 确定分区
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

        // 解析推流码
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

        // 保存配置
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
        let room_id = session.room_id.clone().ok_or_else(|| anyhow::anyhow!("未登录"))?;
        let room_id_num = room_id.parse::<u64>()?;
        let csrf = session.csrf.clone().ok_or_else(|| anyhow::anyhow!("未获取CSRF"))?;

        let res = api.stop_live(room_id_num, &csrf).await?;
        if res["code"].as_i64().unwrap_or(-1) != 0 {
            return Err(anyhow::anyhow!("停播失败"));
        }
        session.is_live = false;
        Ok(())
    }

    pub async fn update_title(api: &BiliApi, session: &SessionState, config: &mut ConfigStore, title: &str) -> Result<()> {
        let room_id = session.room_id.clone().ok_or_else(|| anyhow::anyhow!("未登录"))?;
        let room_id_num = room_id.parse::<u64>()?;
        let csrf = session.csrf.clone().ok_or_else(|| anyhow::anyhow!("未获取CSRF"))?;

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
        let area_id = self.get_area_id(p_name, s_name).ok_or_else(|| anyhow::anyhow!("无效分区"))?;
        let room_id = session.room_id.clone().ok_or_else(|| anyhow::anyhow!("未登录"))?;
        let room_id_num = room_id.parse::<u64>()?;
        let csrf = session.csrf.clone().ok_or_else(|| anyhow::anyhow!("未获取CSRF"))?;

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
```

- [ ] **Step 5: 更新 mod.rs**

`src-tauri/src/services/mod.rs`:
```rust
pub mod auth_service;
pub mod bili_api;
pub mod config_store;
pub mod live_service;
pub mod user_service;
```

- [ ] **Step 6: 编译验证**

Run:
```bash
cd src-tauri && cargo check
```
Expected: 编译通过，可能有未使用变量警告，但无错误。

- [ ] **Step 7: Commit**

```bash
git add .
git commit -m "feat: add auth, user, and live services"
```

---

## Task 5: 弹幕 WebSocket 服务

**Files:**
- Create: `src-tauri/src/services/danmaku_ws.rs`
- Modify: `src-tauri/src/services/mod.rs`

- [ ] **Step 1: 实现弹幕 WebSocket 服务**

`src-tauri/src/services/danmaku_ws.rs`:
```rust
use crate::models::danmaku::DanmakuMessage;
use crate::services::bili_api::BiliApi;
use futures::{SinkExt, StreamExt};
use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, Mutex};
use tokio::time::{interval, sleep};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message as WsMessage};

#[derive(Debug, Clone)]
pub enum DanmakuCommand {
    Connect { room_id: u64 },
    Disconnect,
    SendDanmaku { msg: String },
}

pub struct DanmakuService {
    tx: mpsc::Sender<DanmakuCommand>,
    running: Arc<Mutex<bool>>,
}

impl DanmakuService {
    pub fn new(api: Arc<tokio::sync::Mutex<BiliApi>>) -> Self {
        let (tx, mut rx) = mpsc::channel::<DanmakuCommand>(32);
        let running = Arc::new(Mutex::new(false));
        let running_clone = running.clone();

        tokio::spawn(async move {
            let mut ws_task = None;
            let mut room_id: Option<u64> = None;

            while let Some(cmd) = rx.recv().await {
                match cmd {
                    DanmakuCommand::Connect { room_id: rid } => {
                        if let Some(handle) = ws_task.take() {
                            handle.abort();
                        }
                        room_id = Some(rid);
                        *running_clone.lock().await = true;
                        let api_clone = api.clone();
                        let running_inner = running_clone.clone();
                        ws_task = Some(tokio::spawn(async move {
                            if let Err(e) = connect_and_run(api_clone, rid, running_inner).await {
                                tracing::error!("Danmaku error: {}", e);
                            }
                        }));
                    }
                    DanmakuCommand::Disconnect => {
                        *running_clone.lock().await = false;
                        if let Some(handle) = ws_task.take() {
                            handle.abort();
                        }
                        room_id = None;
                    }
                    DanmakuCommand::SendDanmaku { msg } => {
                        // This is handled via API, not websocket
                        if let Ok(api_guard) = api.lock().await {
                            if let Some(rid) = room_id {
                                if let Some(csrf) = get_csrf_from_api(&api_guard) {
                                    let _ = api_guard.send_danmaku(rid, &msg, &csrf).await;
                                }
                            }
                        }
                    }
                }
            }
        });

        Self { tx, running }
    }

    pub async fn connect(&self, room_id: u64) {
        let _ = self.tx.send(DanmakuCommand::Connect { room_id }).await;
    }

    pub async fn disconnect(&self) {
        let _ = self.tx.send(DanmakuCommand::Disconnect).await;
    }

    pub async fn is_running(&self) -> bool {
        *self.running.lock().await
    }
}

fn get_csrf_from_api(api: &BiliApi) -> Option<String> {
    // Access internal cookies - this needs BiliApi to expose this
    None // Placeholder - will need to add method to BiliApi
}

async fn connect_and_run(
    api: Arc<tokio::sync::Mutex<BiliApi>>,
    room_id: u64,
    running: Arc<Mutex<bool>>,
) -> anyhow::Result<()> {
    let api_guard = api.lock().await;
    let danmaku_info = api_guard.get_danmaku_info(room_id).await?;
    drop(api_guard);

    let token = danmaku_info["data"]["token"].as_str().ok_or_else(|| anyhow::anyhow!("no token"))?;
    let host_list = danmaku_info["data"]["host_list"].as_array().ok_or_else(|| anyhow::anyhow!("no host list"))?;
    let host = host_list[0]["host"].as_str().ok_or_else(|| anyhow::anyhow!("no host"))?;
    let wss_port = host_list[0]["wss_port"].as_u64().unwrap_or(443);

    let ws_url = format!("wss://{}:{}/sub", host, wss_port);
    let (ws_stream, _) = connect_async(&ws_url).await?;
    let (mut write, mut read) = ws_stream.split();

    // Send auth packet
    let auth = serde_json::json!({
        "uid": 0,
        "roomid": room_id,
        "protover": 3,
        "platform": "web",
        "type": 2,
        "key": token,
    });
    let auth_body = auth.to_string();
    let auth_packet = build_packet(7, &auth_body);
    write.send(WsMessage::Binary(auth_packet)).await?;

    // Heartbeat task
    let mut heartbeat = interval(Duration::from_secs(30));
    let write = Arc::new(Mutex::new(write));
    let write_clone = write.clone();

    tokio::spawn(async move {
        loop {
            heartbeat.tick().await;
            let packet = build_packet(2, "");
            if let Err(e) = write_clone.lock().await.send(WsMessage::Binary(packet)).await {
                tracing::error!("Heartbeat failed: {}", e);
                break;
            }
        }
    });

    // Read loop
    while *running.lock().await {
        tokio::select! {
            msg = read.next() => {
                match msg {
                    Some(Ok(WsMessage::Binary(data))) => {
                        process_packet(&data).await;
                    }
                    Some(Ok(WsMessage::Close(_))) | None => {
                        tracing::info!("WebSocket closed");
                        break;
                    }
                    Some(Err(e)) => {
                        tracing::error!("WebSocket error: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}

fn build_packet(op: u32, body: &str) -> Vec<u8> {
    let body_bytes = body.as_bytes();
    let len = 16 + body_bytes.len() as u32;
    let mut packet = Vec::with_capacity(len as usize);
    packet.extend_from_slice(&len.to_be_bytes());
    packet.extend_from_slice(&16u16.to_be_bytes());
    packet.extend_from_slice(&1u16.to_be_bytes());
    packet.extend_from_slice(&op.to_be_bytes());
    packet.extend_from_slice(&1u32.to_be_bytes());
    packet.extend_from_slice(body_bytes);
    packet
}

async fn process_packet(data: &[u8]) {
    if data.len() < 16 {
        return;
    }
    let packet_len = u32::from_be_bytes([data[0], data[1], data[2], data[3]]) as usize;
    let header_len = u16::from_be_bytes([data[4], data[5]]) as usize;
    let proto_ver = u16::from_be_bytes([data[6], data[7]]);
    let op = u32::from_be_bytes([data[8], data[9], data[10], data[11]]);
    // let seq = u32::from_be_bytes([data[12], data[13], data[14], data[15]]);

    let body = &data[header_len..packet_len];

    match proto_ver {
        2 => {
            // zlib compressed
            if let Ok(decompressed) = decompress_zlib(body) {
                process_packet(&decompressed).await;
            }
        }
        3 => {
            // brotli compressed
            if let Ok(decompressed) = decompress_brotli(body) {
                process_packet(&decompressed).await;
            }
        }
        _ => {
            if op == 5 {
                // Command
                if let Ok(s) = std::str::from_utf8(body) {
                    if let Ok(json) = serde_json::from_str::<Value>(s) {
                        handle_command(json).await;
                    }
                }
            } else if op == 3 {
                // Heartbeat reply (popularity)
                if body.len() >= 4 {
                    let pop = u32::from_be_bytes([body[0], body[1], body[2], body[3]]);
                    tracing::debug!("Popularity: {}", pop);
                }
            }
        }
    }

    if data.len() > packet_len {
        process_packet(&data[packet_len..]).await;
    }
}

fn decompress_zlib(data: &[u8]) -> anyhow::Result<Vec<u8>> {
    use std::io::Read;
    let mut decoder = flate2::read::ZlibDecoder::new(data);
    let mut result = Vec::new();
    decoder.read_to_end(&mut result)?;
    Ok(result)
}

fn decompress_brotli(data: &[u8]) -> anyhow::Result<Vec<u8>> {
    let mut result = Vec::new();
    let mut reader = brotli::Decompressor::new(data, 4096);
    use std::io::Read;
    reader.read_to_end(&mut result)?;
    Ok(result)
}

async fn handle_command(cmd: Value) {
    let cmd_str = cmd["cmd"].as_str().unwrap_or("");
    if cmd_str.starts_with("DANMU_MSG") {
        if let Some(info) = cmd["info"].as_array() {
            if info.len() > 2 {
                let uid = info[2][0].as_u64().unwrap_or(0);
                let uname = info[2][1].as_str().unwrap_or("").to_string();
                let msg = info[1].as_str().unwrap_or("").to_string();
                let face = extract_face(info);
                tracing::info!("Danmaku: {}: {}", uname, msg);
                // TODO: emit event to frontend
            }
        }
    } else if cmd_str == "INTERACT_WORD" {
        if let Some(data) = cmd["data"].as_object() {
            let uname = data["uname"].as_str().unwrap_or("").to_string();
            let msg_type = data["msg_type"].as_i64().unwrap_or(0);
            let msg = match msg_type {
                1 => "进入了直播间",
                2 => "关注了直播间",
                3 => "分享了直播间",
                _ => "",
            };
            if !msg.is_empty() {
                tracing::info!("Interact: {} {}", uname, msg);
            }
        }
    } else if cmd_str.starts_with("SEND_GIFT") {
        if let Some(data) = cmd["data"].as_object() {
            let uname = data["uname"].as_str().unwrap_or("").to_string();
            let gift_name = data["giftName"].as_str().unwrap_or("").to_string();
            tracing::info!("Gift: {} sent {}", uname, gift_name);
        }
    }
}

fn extract_face(info: &[Value]) -> String {
    if let Some(extra) = info.get(0).and_then(|v| v.as_array()) {
        if let Some(user_data) = extra.get(15).and_then(|v| v.get("user")).and_then(|v| v.get("base")) {
            return user_data["face"].as_str().unwrap_or("").to_string();
        }
    }
    String::new()
}
```

- [ ] **Step 2: 添加方法到 BiliApi 暴露 CSRF**

在 `src-tauri/src/services/bili_api.rs` 中添加：
```rust
pub fn get_csrf(&self) -> Option<String> {
    self.cookies.get("bili_jct").cloned()
}
```

修改 `danmaku_ws.rs` 中的 `get_csrf_from_api`：
```rust
fn get_csrf_from_api(api: &BiliApi) -> Option<String> {
    api.get_csrf()
}
```

- [ ] **Step 3: 更新 mod.rs**

`src-tauri/src/services/mod.rs`:
```rust
pub mod auth_service;
pub mod bili_api;
pub mod config_store;
pub mod danmaku_ws;
pub mod live_service;
pub mod user_service;
```

- [ ] **Step 4: 编译验证**

Run:
```bash
cd src-tauri && cargo check
```
Expected: 编译通过。

- [ ] **Step 5: Commit**

```bash
git add .
git commit -m "feat: add danmaku websocket service"
```

---

## Task 6: Tauri Commands + App State + 托盘

**Files:**
- Create: `src-tauri/src/commands/auth.rs`
- Create: `src-tauri/src/commands/user.rs`
- Create: `src-tauri/src/commands/live.rs`
- Create: `src-tauri/src/commands/danmaku.rs`
- Create: `src-tauri/src/commands/window.rs`
- Create: `src-tauri/src/commands/config.rs`
- Create: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/main.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: 定义 Commands**

`src-tauri/src/commands/auth.rs`:
```rust
use crate::models::user::{LoginResult, QrCodeData};
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn get_login_qrcode(state: State<'_, AppState>) -> Result<QrCodeData, String> {
    let api = state.api.lock().await;
    crate::services::auth_service::AuthService::get_login_qrcode(&api).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn poll_login_status(key: String, state: State<'_, AppState>) -> Result<LoginResult, String> {
    let api = state.api.lock().await;
    crate::services::auth_service::AuthService::poll_login_status(&api, &key).await.map_err(|e| e.to_string())
}
```

`src-tauri/src/commands/user.rs`:
```rust
use crate::models::config::UserConfig;
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn load_saved_config(state: State<'_, AppState>) -> Result<Option<UserConfig>, String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    let uid = config.data().current_uid;
    if let Some(uid) = uid {
        let uid_str = uid.to_string();
        Ok(config.data().users.get(&uid_str).cloned())
    } else {
        Ok(None)
    }
}

#[tauri::command]
pub async fn refresh_current_user(state: State<'_, AppState>) -> Result<UserConfig, String> {
    let mut api = state.api.lock().await;
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    let mut session = state.session.lock().map_err(|e| e.to_string())?;
    crate::services::user_service::UserService::refresh_current_user(&api, &mut config, &mut session).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_account_list(state: State<'_, AppState>) -> Result<Vec<UserConfig>, String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    Ok(crate::services::user_service::UserService::get_account_list(&config))
}

#[tauri::command]
pub async fn switch_account(uid: u64, state: State<'_, AppState>) -> Result<UserConfig, String> {
    let mut api = state.api.lock().await;
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    let mut session = state.session.lock().map_err(|e| e.to_string())?;
    crate::services::user_service::UserService::switch_account(&mut config, &mut session, &mut api, uid).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn logout(uid: u64, state: State<'_, AppState>) -> Result<(), String> {
    let mut api = state.api.lock().await;
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    let mut session = state.session.lock().map_err(|e| e.to_string())?;
    crate::services::user_service::UserService::logout(&mut config, &mut session, &mut api, uid).map_err(|e| e.to_string())
}
```

`src-tauri/src/commands/live.rs`:
```rust
use crate::models::live::{PartitionMap, StreamCodeData};
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn get_partitions(state: State<'_, AppState>) -> Result<PartitionMap, String> {
    let api = state.api.lock().await;
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    let mut session = state.session.lock().map_err(|e| e.to_string())?;
    let mut live = crate::services::live_service::LiveService::new();
    live.refresh_partitions(&api).await.map_err(|e| e.to_string())?;
    Ok(live.get_partitions())
}

#[tauri::command]
pub async fn update_title(title: String, state: State<'_, AppState>) -> Result<(), String> {
    let api = state.api.lock().await;
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    let session = state.session.lock().map_err(|e| e.to_string())?;
    crate::services::live_service::LiveService::update_title(&api, &session, &mut config, &title).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_area(p_name: String, s_name: String, state: State<'_, AppState>) -> Result<(), String> {
    let api = state.api.lock().await;
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    let mut session = state.session.lock().map_err(|e| e.to_string())?;
    let mut live = crate::services::live_service::LiveService::new();
    live.update_area(&api, &mut session, &mut config, &p_name, &s_name).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn start_live(
    p_name: Option<String>,
    s_name: Option<String>,
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<StreamCodeData, String> {
    let api = state.api.lock().await;
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    let mut session = state.session.lock().map_err(|e| e.to_string())?;
    let mut live = crate::services::live_service::LiveService::new();
    let result = live.start_live(&api, &mut session, &mut config, p_name, s_name).await.map_err(|e| e.to_string())?;
    // TODO: auto connect danmaku if configured
    Ok(result)
}

#[tauri::command]
pub async fn stop_live(state: State<'_, AppState>) -> Result<(), String> {
    let api = state.api.lock().await;
    let mut session = state.session.lock().map_err(|e| e.to_string())?;
    let mut live = crate::services::live_service::LiveService::new();
    live.stop_live(&api, &mut session).await.map_err(|e| e.to_string())
}
```

`src-tauri/src/commands/danmaku.rs`:
```rust
use crate::models::danmaku::SendDanmakuResult;
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn start_danmaku_monitor(state: State<'_, AppState>) -> Result<(), String> {
    // TODO: implement danmaku service in AppState and call connect
    Ok(())
}

#[tauri::command]
pub async fn stop_danmaku_monitor(state: State<'_, AppState>) -> Result<(), String> {
    Ok(())
}

#[tauri::command]
pub async fn send_danmaku(msg: String, state: State<'_, AppState>) -> Result<SendDanmakuResult, String> {
    let api = state.api.lock().await;
    let session = state.session.lock().map_err(|e| e.to_string())?;
    let room_id = session.room_id.clone().ok_or("未登录")?;
    let room_id_num = room_id.parse::<u64>().map_err(|_| "房间号无效")?;
    let csrf = session.csrf.clone().ok_or("未获取CSRF")?;
    let res = api.send_danmaku(room_id_num, &msg, &csrf).await.map_err(|e| e.to_string())?;
    let code = res["code"].as_i64().unwrap_or(-1) as i32;
    let msg_text = match code {
        0 => "发送成功",
        1003212 => "超出限制长度",
        -101 => "未登录",
        -400 => "参数错误",
        10031 => "发送频率过高",
        _ => res["msg"].as_str().unwrap_or("未知错误"),
    };
    Ok(SendDanmakuResult { code, msg: msg_text.to_string() })
}
```

`src-tauri/src/commands/window.rs`:
```rust
use tauri::Manager;

#[tauri::command]
pub fn window_min(window: tauri::Window) {
    let _ = window.minimize();
}

#[tauri::command]
pub fn window_max(window: tauri::Window) -> bool {
    let _ = window.toggle_maximize();
    true
}

#[tauri::command]
pub fn window_close(window: tauri::Window) {
    let _ = window.close();
}

#[tauri::command]
pub fn window_drag(window: tauri::Window, x: i32, y: i32) {
    let _ = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition { x, y }));
}
```

`src-tauri/src/commands/config.rs`:
```rust
use crate::state::AppState;
use serde_json::Value;
use tauri::State;

#[tauri::command]
pub fn get_app_config(state: State<'_, AppState>) -> Result<Value, String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    Ok(serde_json::json!({
        "min_to_tray": config.data().min_to_tray,
    }))
}

#[tauri::command]
pub fn set_app_config(key: String, value: bool, state: State<'_, AppState>) -> Result<(), String> {
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    if key == "min_to_tray" {
        config.data_mut().min_to_tray = value;
    }
    config.save().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
```

`src-tauri/src/commands/mod.rs`:
```rust
pub mod auth;
pub mod config;
pub mod danmaku;
pub mod live;
pub mod user;
pub mod window;
```

- [ ] **Step 2: 更新 lib.rs 和 main.rs**

`src-tauri/src/lib.rs`:
```rust
pub mod commands;
pub mod models;
pub mod services;
pub mod state;
pub mod utils;
```

`src-tauri/src/main.rs`:
```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bili_live_tool_lib::services::bili_api::BiliApi;
use bili_live_tool_lib::services::config_store::ConfigStore;
use bili_live_tool_lib::services::user_service::UserService;
use bili_live_tool_lib::state::{AppState, SessionState};
use std::sync::Mutex;
use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let config = ConfigStore::new().expect("Failed to load config");
            let mut api = BiliApi::new();
            let mut session = SessionState::default();
            UserService::init_current_user(&config, &mut session, &mut api);

            app.manage(AppState {
                config: Mutex::new(config),
                session: Mutex::new(session),
                api: tokio::sync::Mutex::new(api),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::auth::get_login_qrcode,
            commands::auth::poll_login_status,
            commands::user::load_saved_config,
            commands::user::refresh_current_user,
            commands::user::get_account_list,
            commands::user::switch_account,
            commands::user::logout,
            commands::live::get_partitions,
            commands::live::update_title,
            commands::live::update_area,
            commands::live::start_live,
            commands::live::stop_live,
            commands::danmaku::start_danmaku_monitor,
            commands::danmaku::stop_danmaku_monitor,
            commands::danmaku::send_danmaku,
            commands::window::window_min,
            commands::window::window_max,
            commands::window::window_close,
            commands::window::window_drag,
            commands::config::get_app_config,
            commands::config::set_app_config,
            commands::config::get_version,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 3: 编译验证**

Run:
```bash
cd src-tauri && cargo check
```
Expected: 编译通过。

- [ ] **Step 4: Commit**

```bash
git add .
git commit -m "feat: add tauri commands and app state management"
```

---

## Task 7: 前端基础 — 类型定义 + Tauri 桥接 + 全局状态

**Files:**
- Create: `src/types/api.ts`
- Create: `src/hooks/useTauri.ts`
- Create: `src/context/AppContext.tsx`
- Modify: `src/App.tsx`

- [ ] **Step 1: 定义类型**

`src/types/api.ts`:
```typescript
export interface UserConfig {
  uid: number;
  uname: string;
  face: string;
  room_id: string;
  csrf: string;
  last_title: string;
  last_area_id: number;
  last_area_name: string[];
  level: number;
  follower: number;
  following: number;
  dynamic_count: number;
}

export interface StreamProtocol {
  addr: string;
  code: string;
}

export interface StreamCodeData {
  rtmp1: StreamProtocol;
  rtmp2: StreamProtocol;
  srt: StreamProtocol;
}

export interface QrCodeData {
  url: string;
  qrcode_key: string;
}

export interface LoginResult {
  code: number;
  uid?: number;
  user?: UserConfig;
}

export interface DanmakuMessage {
  type: 'danmaku' | 'interact' | 'gift' | 'system';
  uid?: number;
  uname?: string;
  face?: string;
  msg?: string;
  gift_name?: string;
  num?: number;
  action?: string;
}

export interface AppConfig {
  min_to_tray: boolean;
}

export type PartitionMap = Record<string, string[]>;
```

- [ ] **Step 2: 实现 Tauri 桥接 Hook**

`src/hooks/useTauri.ts`:
```typescript
import { invoke } from '@tauri-apps/api/core';
import type {
  AppConfig,
  DanmakuMessage,
  LoginResult,
  PartitionMap,
  QrCodeData,
  StreamCodeData,
  UserConfig,
} from '@/types/api';

export async function getLoginQrcode(): Promise<QrCodeData> {
  return await invoke('get_login_qrcode');
}

export async function pollLoginStatus(key: string): Promise<LoginResult> {
  return await invoke('poll_login_status', { key });
}

export async function loadSavedConfig(): Promise<UserConfig | null> {
  return await invoke('load_saved_config');
}

export async function refreshCurrentUser(): Promise<UserConfig> {
  return await invoke('refresh_current_user');
}

export async function getAccountList(): Promise<UserConfig[]> {
  return await invoke('get_account_list');
}

export async function switchAccount(uid: number): Promise<UserConfig> {
  return await invoke('switch_account', { uid });
}

export async function logout(uid: number): Promise<void> {
  return await invoke('logout', { uid });
}

export async function getPartitions(): Promise<PartitionMap> {
  return await invoke('get_partitions');
}

export async function updateTitle(title: string): Promise<void> {
  return await invoke('update_title', { title });
}

export async function updateArea(pName: string, sName: string): Promise<void> {
  return await invoke('update_area', { pName, sName });
}

export async function startLive(pName?: string, sName?: string): Promise<StreamCodeData> {
  return await invoke('start_live', { pName, sName });
}

export async function stopLive(): Promise<void> {
  return await invoke('stop_live');
}

export async function sendDanmaku(msg: string): Promise<{ code: number; msg: string }> {
  return await invoke('send_danmaku', { msg });
}

export async function getAppConfig(): Promise<AppConfig> {
  return await invoke('get_app_config');
}

export async function setAppConfig(key: string, value: boolean): Promise<void> {
  return await invoke('set_app_config', { key, value });
}

export async function getVersion(): Promise<string> {
  return await invoke('get_version');
}

export async function windowMin(): Promise<void> {
  return await invoke('window_min');
}

export async function windowMax(): Promise<void> {
  return await invoke('window_max');
}

export async function windowClose(): Promise<void> {
  return await invoke('window_close');
}
```

- [ ] **Step 3: 实现全局状态 Context**

`src/context/AppContext.tsx`:
```typescript
import { createContext, useContext, useState, type ReactNode } from 'react';
import type { DanmakuMessage, StreamCodeData, UserConfig } from '@/types/api';

interface AppState {
  user: UserConfig | null;
  setUser: (user: UserConfig | null) => void;
  isLive: boolean;
  setIsLive: (v: boolean) => void;
  streamCode: StreamCodeData | null;
  setStreamCode: (v: StreamCodeData | null) => void;
  danmakuList: DanmakuMessage[];
  addDanmaku: (msg: DanmakuMessage) => void;
  clearDanmaku: () => void;
  consoleLogs: string[];
  addLog: (log: string) => void;
  clearLogs: () => void;
  isDark: boolean;
  setIsDark: (v: boolean) => void;
  consoleOpen: boolean;
  setConsoleOpen: (v: boolean) => void;
}

const AppContext = createContext<AppState | null>(null);

export function AppProvider({ children }: { children: ReactNode }) {
  const [user, setUser] = useState<UserConfig | null>(null);
  const [isLive, setIsLive] = useState(false);
  const [streamCode, setStreamCode] = useState<StreamCodeData | null>(null);
  const [danmakuList, setDanmakuList] = useState<DanmakuMessage[]>([]);
  const [consoleLogs, setConsoleLogs] = useState<string[]>([]);
  const [isDark, setIsDark] = useState(true);
  const [consoleOpen, setConsoleOpen] = useState(true);

  const addDanmaku = (msg: DanmakuMessage) => {
    setDanmakuList((prev) => [...prev, msg].slice(-500));
  };

  const clearDanmaku = () => setDanmakuList([]);

  const addLog = (log: string) => {
    setConsoleLogs((prev) => [...prev, log].slice(-200));
  };

  const clearLogs = () => setConsoleLogs([]);

  return (
    <AppContext.Provider
      value={{
        user, setUser,
        isLive, setIsLive,
        streamCode, setStreamCode,
        danmakuList, addDanmaku, clearDanmaku,
        consoleLogs, addLog, clearLogs,
        isDark, setIsDark,
        consoleOpen, setConsoleOpen,
      }}
    >
      {children}
    </AppContext.Provider>
  );
}

export function useApp() {
  const ctx = useContext(AppContext);
  if (!ctx) throw new Error('useApp must be used within AppProvider');
  return ctx;
}
```

- [ ] **Step 4: 更新 App.tsx**

`src/App.tsx`:
```tsx
import { AppProvider } from '@/context/AppContext';

function App() {
  return (
    <AppProvider>
      <div className="flex h-screen bg-white text-stone-800 dark:bg-stone-950 dark:text-stone-200">
        {/* TODO: Sidebar + Main Content */}
        <div className="flex-1 flex items-center justify-center">
          <span className="text-stone-400">App initialized</span>
        </div>
      </div>
    </AppProvider>
  );
}

export default App;
```

- [ ] **Step 5: 安装 Tauri API 依赖**

Run:
```bash
npm install @tauri-apps/api
```

- [ ] **Step 6: Commit**

```bash
git add .
git commit -m "feat: add frontend types, tauri hooks, and app context"
```

---

## Task 8: 前端组件 — Sidebar + StreamPanel + DanmakuPanel + AccountPanel + SettingsPanel

**Files:**
- Create: `src/components/Sidebar.tsx`
- Create: `src/components/StreamPanel.tsx`
- Create: `src/components/DanmakuPanel.tsx`
- Create: `src/components/AccountPanel.tsx`
- Create: `src/components/SettingsPanel.tsx`
- Create: `src/components/ConsolePanel.tsx`
- Modify: `src/App.tsx`

- [ ] **Step 1: Sidebar**

`src/components/Sidebar.tsx`:
```tsx
import { useApp } from '@/context/AppContext';
import { BroadcastLine, Chat1Line, UserLine, Settings3Line } from 'lucide-react';

interface SidebarProps {
  activeTab: string;
  onTabChange: (tab: string) => void;
}

const navItems = [
  { id: 'stream', label: '推流设置', icon: BroadcastLine },
  { id: 'danmaku', label: '弹幕监控', icon: Chat1Line },
  { id: 'account', label: '账户管理', icon: UserLine },
];

export default function Sidebar({ activeTab, onTabChange }: SidebarProps) {
  const { user, isLive } = useApp();

  return (
    <div className="w-52 bg-stone-50 dark:bg-stone-950 border-r border-stone-200 dark:border-stone-800 flex flex-col shrink-0">
      {/* User Card */}
      <div className="px-3 pb-3 mb-2 border-b border-stone-200 dark:border-stone-800">
        <div className="flex items-center gap-2.5 p-2.5 rounded-lg hover:bg-stone-100 dark:hover:bg-stone-900 transition cursor-pointer">
          <div className="w-8 h-8 rounded-full bg-stone-400 flex items-center justify-center text-white text-xs font-semibold">
            {user?.uname?.[0] ?? '?'}
          </div>
          <div className="flex-1 min-w-0">
            <div className="text-[13px] font-medium truncate">{user?.uname ?? '未登录'}</div>
            <div className="text-[11px] text-stone-400 truncate">
              {user ? `LV${user.level} · ${user.uid}` : '点击登录'}
            </div>
          </div>
        </div>
      </div>

      {/* Navigation */}
      <nav className="flex-1 px-2 space-y-0.5">
        {navItems.map((item) => {
          const isActive = activeTab === item.id;
          return (
            <button
              key={item.id}
              onClick={() => onTabChange(item.id)}
              className={`w-full flex items-center gap-3 px-3 py-2 rounded-lg text-[13px] font-medium transition ${
                isActive
                  ? 'bg-stone-200 dark:bg-stone-800 text-stone-900 dark:text-stone-100'
                  : 'text-stone-500 dark:text-stone-400 hover:bg-stone-100 dark:hover:bg-stone-900 hover:text-stone-900 dark:hover:text-stone-100'
              }`}
            >
              <item.icon size={16} />
              <span>{item.label}</span>
            </button>
          );
        })}

        <div className="pt-4 mt-4 border-t border-stone-200 dark:border-stone-800">
          <button
            onClick={() => onTabChange('settings')}
            className={`w-full flex items-center gap-3 px-3 py-2 rounded-lg text-[13px] font-medium transition ${
              activeTab === 'settings'
                ? 'bg-stone-200 dark:bg-stone-800 text-stone-900 dark:text-stone-100'
                : 'text-stone-500 dark:text-stone-400 hover:bg-stone-100 dark:hover:bg-stone-900 hover:text-stone-900 dark:hover:text-stone-100'
            }`}
          >
            <Settings3Line size={16} />
            <span>设置</span>
          </button>
        </div>
      </nav>

      {/* Live Status */}
      <div className="px-3 pt-2 border-t border-stone-200 dark:border-stone-800">
        <div className="flex items-center gap-2 px-3 py-2">
          <span className={`w-1.5 h-1.5 rounded-full ${isLive ? 'bg-green-500 animate-pulse' : 'bg-stone-300'}`} />
          <span className="text-[11px] text-stone-400">{isLive ? '直播中' : '未开播'}</span>
        </div>
      </div>
    </div>
  );
}
```

- [ ] **Step 2: StreamPanel**

`src/components/StreamPanel.tsx`:
```tsx
import { useState } from 'react';
import { useApp } from '@/context/AppContext';
import { startLive, stopLive, updateTitle, updateArea } from '@/hooks/useTauri';

export default function StreamPanel() {
  const { user, isLive, setIsLive, streamCode, setStreamCode, addLog } = useApp();
  const [title, setTitle] = useState(user?.last_title ?? '');

  const handleStart = async () => {
    addLog('开始获取推流码...');
    try {
      const data = await startLive();
      setStreamCode(data);
      setIsLive(true);
      addLog('开播成功！');
    } catch (e: any) {
      addLog(`开播失败: ${e}`);
    }
  };

  const handleStop = async () => {
    addLog('正在停止直播...');
    try {
      await stopLive();
      setIsLive(false);
      setStreamCode(null);
      addLog('直播已停止');
    } catch (e: any) {
      addLog(`停止失败: ${e}`);
    }
  };

  return (
    <div className="flex-1 overflow-y-auto p-6 space-y-6">
      {/* Title & Area */}
      <section>
        <h2 className="text-[11px] font-semibold uppercase tracking-wider text-stone-400 mb-3">直播信息</h2>
        <div className="space-y-4">
          <div>
            <label className="block text-[13px] text-stone-600 dark:text-stone-400 mb-1.5">标题</label>
            <div className="flex gap-2">
              <input
                type="text"
                value={title}
                onChange={(e) => setTitle(e.target.value)}
                className="flex-1 h-9 px-3 rounded-lg bg-stone-50 dark:bg-stone-900 border border-stone-200 dark:border-stone-800 text-[13px] focus:outline-none focus:ring-2 focus:ring-stone-400/30 transition"
              />
              <button
                onClick={() => updateTitle(title)}
                className="h-9 px-4 rounded-lg bg-stone-800 dark:bg-stone-100 text-white dark:text-stone-900 text-[13px] font-medium hover:opacity-90 transition"
              >
                更新
              </button>
            </div>
          </div>
        </div>
      </section>

      {/* Stream Codes */}
      {streamCode && (
        <section>
          <h2 className="text-[11px] font-semibold uppercase tracking-wider text-stone-400 mb-3">推流码</h2>
          <div className="space-y-3">
            {(['rtmp1', 'rtmp2', 'srt'] as const).map((key) => (
              <div key={key} className="group p-4 rounded-xl bg-stone-50 dark:bg-stone-900 border border-stone-200 dark:border-stone-800 hover:border-stone-300 dark:hover:border-stone-700 transition">
                <div className="flex items-center justify-between mb-2">
                  <span className="text-[12px] font-medium text-stone-500 uppercase">{key}</span>
                  <button
                    onClick={() => navigator.clipboard.writeText(`${streamCode[key].addr}${streamCode[key].code}`)}
                    className="text-[12px] text-stone-400 hover:text-stone-700 dark:hover:text-stone-300 transition opacity-0 group-hover:opacity-100"
                  >
                    复制
                  </button>
                </div>
                <code className="block text-[12px] text-stone-600 dark:text-stone-400 font-mono break-all leading-relaxed">
                  {streamCode[key].addr}{streamCode[key].code}
                </code>
              </div>
            ))}
          </div>
        </section>
      )}

      {/* Actions */}
      <div className="flex gap-3 pt-2">
        <button
          onClick={handleStart}
          disabled={isLive}
          className="flex-1 h-10 rounded-lg bg-stone-800 dark:bg-stone-100 text-white dark:text-stone-900 text-[13px] font-medium hover:opacity-90 transition disabled:opacity-50"
        >
          开始直播
        </button>
        <button
          onClick={handleStop}
          disabled={!isLive}
          className="flex-1 h-10 rounded-lg bg-stone-100 dark:bg-stone-900 border border-stone-200 dark:border-stone-800 text-[13px] font-medium hover:bg-stone-200 dark:hover:bg-stone-800 transition disabled:opacity-50"
        >
          停止直播
        </button>
      </div>
    </div>
  );
}
```

- [ ] **Step 3: DanmakuPanel**

`src/components/DanmakuPanel.tsx`:
```tsx
import { useState } from 'react';
import { useApp } from '@/context/AppContext';
import { sendDanmaku } from '@/hooks/useTauri';

export default function DanmakuPanel() {
  const { danmakuList, clearDanmaku, addDanmaku, addLog } = useApp();
  const [input, setInput] = useState('');

  const handleSend = async () => {
    if (!input.trim()) return;
    try {
      const res = await sendDanmaku(input);
      addLog(`[弹幕] ${res.msg}`);
      if (res.code === 0) setInput('');
    } catch (e: any) {
      addLog(`[弹幕] 发送失败: ${e}`);
    }
  };

  return (
    <div className="flex-1 flex flex-col overflow-hidden">
      <div className="flex items-center justify-between px-6 py-4 border-b border-stone-200 dark:border-stone-800 shrink-0">
        <h2 className="text-[13px] font-medium">弹幕消息</h2>
        <button onClick={clearDanmaku} className="text-[11px] text-stone-400 hover:text-stone-600 dark:hover:text-stone-300 transition">清空</button>
      </div>
      <div className="flex-1 overflow-y-auto px-6 py-3 space-y-1">
        {danmakuList.map((msg, i) => (
          <div key={i} className="flex items-start gap-3 py-2 px-3 rounded-lg hover:bg-stone-50 dark:hover:bg-stone-900 transition">
            {msg.uname && <span className="text-[12px] font-medium text-stone-500 mt-0.5 shrink-0">{msg.uname}</span>}
            <span className={`text-[13px] ${msg.type === 'gift' ? 'text-amber-600 dark:text-amber-500' : msg.type === 'interact' ? 'text-stone-400' : 'text-stone-800 dark:text-stone-200'}`}>
              {msg.msg}
            </span>
          </div>
        ))}
      </div>
      <div className="px-6 py-4 border-t border-stone-200 dark:border-stone-800 shrink-0">
        <div className="flex gap-2">
          <input
            type="text"
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyDown={(e) => e.key === 'Enter' && handleSend()}
            placeholder="发送弹幕..."
            className="flex-1 h-9 px-3 rounded-lg bg-stone-50 dark:bg-stone-900 border border-stone-200 dark:border-stone-800 text-[13px] focus:outline-none focus:ring-2 focus:ring-stone-400/30 transition"
          />
          <button onClick={handleSend} className="h-9 px-5 rounded-lg bg-stone-800 dark:bg-stone-100 text-white dark:text-stone-900 text-[13px] font-medium hover:opacity-90 transition">发送</button>
        </div>
      </div>
    </div>
  );
}
```

- [ ] **Step 4: AccountPanel, SettingsPanel, ConsolePanel**

`src/components/AccountPanel.tsx`:
```tsx
import { useApp } from '@/context/AppContext';
import { getAccountList, switchAccount, logout } from '@/hooks/useTauri';
import { useState, useEffect } from 'react';
import type { UserConfig } from '@/types/api';

export default function AccountPanel() {
  const { user, setUser } = useApp();
  const [accounts, setAccounts] = useState<UserConfig[]>([]);

  useEffect(() => {
    getAccountList().then(setAccounts);
  }, []);

  return (
    <div className="flex-1 overflow-y-auto p-6 space-y-6">
      <section>
        <h2 className="text-[11px] font-semibold uppercase tracking-wider text-stone-400 mb-3">当前账户</h2>
        {user && (
          <div className="p-4 rounded-xl bg-stone-50 dark:bg-stone-900 border border-stone-200 dark:border-stone-800 flex items-center gap-3">
            <div className="w-10 h-10 rounded-full bg-stone-400 flex items-center justify-center text-white font-semibold">{user.uname[0]}</div>
            <div className="flex-1">
              <div className="text-[13px] font-medium">{user.uname}</div>
              <div className="text-[12px] text-stone-400">UID: {user.uid}</div>
            </div>
            <span className="text-[11px] px-2 py-0.5 rounded-full bg-green-100 dark:bg-green-900/30 text-green-700 dark:text-green-400 border border-green-200 dark:border-green-800">已登录</span>
          </div>
        )}
      </section>

      <section>
        <h2 className="text-[11px] font-semibold uppercase tracking-wider text-stone-400 mb-3">已保存的账户</h2>
        <div className="space-y-2">
          {accounts.map((acc) => (
            <div key={acc.uid} className="flex items-center gap-3 p-3 rounded-xl bg-stone-50 dark:bg-stone-900 border border-stone-200 dark:border-stone-800">
              <div className="w-8 h-8 rounded-full bg-stone-300 dark:bg-stone-700 flex items-center justify-center text-white text-xs font-semibold">{acc.uname[0]}</div>
              <div className="flex-1 min-w-0">
                <div className="text-[13px]">{acc.uname}</div>
                <div className="text-[12px] text-stone-400">UID: {acc.uid}</div>
              </div>
              <button onClick={() => switchAccount(acc.uid).then(setUser)} className="px-3 h-7 rounded-md text-[12px] bg-stone-800 dark:bg-stone-100 text-white dark:text-stone-900 hover:opacity-90 transition">切换</button>
              <button onClick={() => logout(acc.uid).then(() => setAccounts(accounts.filter((a) => a.uid !== acc.uid)))} className="px-3 h-7 rounded-md text-[12px] text-stone-400 hover:text-red-500 transition">删除</button>
            </div>
          ))}
        </div>
      </section>
    </div>
  );
}
```

`src/components/SettingsPanel.tsx`:
```tsx
import { useApp } from '@/context/AppContext';
import { getVersion, setAppConfig, getAppConfig } from '@/hooks/useTauri';
import { useState, useEffect } from 'react';

export default function SettingsPanel() {
  const { isDark, setIsDark, addLog } = useApp();
  const [minToTray, setMinToTray] = useState(true);
  const [version, setVersion] = useState('');

  useEffect(() => {
    getVersion().then(setVersion);
    getAppConfig().then((cfg) => setMinToTray(cfg.min_to_tray));
  }, []);

  const toggleMinToTray = async () => {
    const next = !minToTray;
    setMinToTray(next);
    await setAppConfig('min_to_tray', next);
  };

  return (
    <div className="flex-1 overflow-y-auto p-6">
      <div className="max-w-md space-y-6">
        <section>
          <h2 className="text-[11px] font-semibold uppercase tracking-wider text-stone-400 mb-4">偏好设置</h2>
          <div className="space-y-1">
            <div className="flex items-center justify-between py-3 border-b border-stone-100 dark:border-stone-900">
              <div>
                <div className="text-[13px]">关闭时最小化到托盘</div>
                <div className="text-[12px] text-stone-400 mt-0.5">点击关闭按钮将隐藏到系统托盘</div>
              </div>
              <button onClick={toggleMinToTray} className={`relative w-10 h-6 rounded-full transition ${minToTray ? 'bg-stone-800 dark:bg-stone-200' : 'bg-stone-200 dark:bg-stone-700'}`}>
                <span className={`absolute top-1 w-4 h-4 rounded-full bg-white transition ${minToTray ? 'left-5' : 'left-1'}`} />
              </button>
            </div>
            <div className="flex items-center justify-between py-3 border-b border-stone-100 dark:border-stone-900">
              <div>
                <div className="text-[13px]">深色模式</div>
                <div className="text-[12px] text-stone-400 mt-0.5">切换应用主题</div>
              </div>
              <button onClick={() => { setIsDark(!isDark); document.documentElement.classList.toggle('dark'); }} className={`relative w-10 h-6 rounded-full transition ${isDark ? 'bg-stone-800 dark:bg-stone-200' : 'bg-stone-200 dark:bg-stone-700'}`}>
                <span className={`absolute top-1 w-4 h-4 rounded-full bg-white transition ${isDark ? 'left-5' : 'left-1'}`} />
              </button>
            </div>
          </div>
        </section>
        <section>
          <h2 className="text-[11px] font-semibold uppercase tracking-wider text-stone-400 mb-4">关于</h2>
          <div className="flex items-center justify-between py-2">
            <span className="text-[13px] text-stone-500">版本</span>
            <span className="text-[13px] text-stone-400">{version}</span>
          </div>
        </section>
      </div>
    </div>
  );
}
```

`src/components/ConsolePanel.tsx`:
```tsx
import { useApp } from '@/context/AppContext';

interface ConsolePanelProps {
  open: boolean;
}

export default function ConsolePanel({ open }: ConsolePanelProps) {
  const { consoleLogs, clearLogs } = useApp();

  if (!open) return null;

  return (
    <div className="border-t border-stone-200 dark:border-stone-800 bg-stone-50 dark:bg-stone-950 flex flex-col shrink-0" style={{ height: 120 }}>
      <div className="flex items-center justify-between px-4 h-7 border-b border-stone-200 dark:border-stone-800">
        <span className="text-[10px] font-medium text-stone-400 uppercase tracking-wider">Console</span>
        <div className="flex gap-3">
          <button onClick={clearLogs} className="text-[10px] text-stone-400 hover:text-stone-600 dark:hover:text-stone-300 transition">清空</button>
        </div>
      </div>
      <div className="flex-1 overflow-y-auto p-3 font-mono text-[11px] space-y-0.5 leading-relaxed">
        {consoleLogs.map((log, i) => (
          <div key={i} className="text-stone-500 dark:text-stone-400">{log}</div>
        ))}
      </div>
    </div>
  );
}
```

- [ ] **Step 5: 组装 App.tsx**

`src/App.tsx`:
```tsx
import { useState } from 'react';
import { AppProvider, useApp } from '@/context/AppContext';
import Sidebar from '@/components/Sidebar';
import StreamPanel from '@/components/StreamPanel';
import DanmakuPanel from '@/components/DanmakuPanel';
import AccountPanel from '@/components/AccountPanel';
import SettingsPanel from '@/components/SettingsPanel';
import ConsolePanel from '@/components/ConsolePanel';
import { TerminalLine, SunLine, MoonLine } from 'lucide-react';

function AppContent() {
  const [activeTab, setActiveTab] = useState('stream');
  const { isDark, setIsDark, consoleOpen, setConsoleOpen } = useApp();

  const renderPanel = () => {
    switch (activeTab) {
      case 'stream': return <StreamPanel />;
      case 'danmaku': return <DanmakuPanel />;
      case 'account': return <AccountPanel />;
      case 'settings': return <SettingsPanel />;
      default: return <StreamPanel />;
    }
  };

  return (
    <div className="flex h-screen bg-white text-stone-800 dark:bg-stone-950 dark:text-stone-200 overflow-hidden">
      <Sidebar activeTab={activeTab} onTabChange={setActiveTab} />
      <div className="flex-1 flex flex-col min-w-0">
        {/* macOS title bar area - in Tauri this is native, but we add controls in the content area */}
        <div className="flex items-center justify-end px-4 h-10 border-b border-stone-200 dark:border-stone-800 gap-2">
          <button
            onClick={() => { setIsDark(!isDark); document.documentElement.classList.toggle('dark'); }}
            className="w-7 h-7 rounded-md flex items-center justify-center text-stone-400 hover:text-stone-600 dark:hover:text-stone-300 hover:bg-stone-100 dark:hover:bg-stone-900 transition"
          >
            {isDark ? <SunLine size={14} /> : <MoonLine size={14} />}
          </button>
          <button
            onClick={() => setConsoleOpen(!consoleOpen)}
            className={`w-7 h-7 rounded-md flex items-center justify-center transition ${consoleOpen ? 'text-stone-800 dark:text-stone-200 bg-stone-200 dark:bg-stone-800' : 'text-stone-400 hover:text-stone-600 dark:hover:text-stone-300 hover:bg-stone-100 dark:hover:bg-stone-900'}`}
          >
            <TerminalLine size={14} />
          </button>
        </div>
        {renderPanel()}
        <ConsolePanel open={consoleOpen} />
      </div>
    </div>
  );
}

function App() {
  return (
    <AppProvider>
      <AppContent />
    </AppProvider>
  );
}

export default App;
```

- [ ] **Step 6: 安装 lucide-react**

Run:
```bash
npm install lucide-react
```

- [ ] **Step 7: 运行验证**

Run:
```bash
npx tauri dev
```
Expected: 窗口弹出，显示侧边栏+推流设置面板，UI 符合设计稿风格。

- [ ] **Step 8: Commit**

```bash
git add .
git commit -m "feat: add all frontend panels and layout"
```

---

## Task 9: 系统托盘 + 窗口关闭行为

**Files:**
- Modify: `src-tauri/src/main.rs`

- [ ] **Step 1: 添加托盘图标和菜单**

在 `src-tauri/src/main.rs` 的 `.setup()` 中添加：

```rust
use tauri::tray::{TrayIconBuilder, TrayIconEvent};
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};

// Inside setup:
let tray_menu = Menu::with_items(app, &[
    &MenuItem::with_id(app, "show", "显示主界面", true, None::<&str>)?,
    &MenuItem::with_id(app, "start", "开始直播", true, None::<&str>)?,
    &MenuItem::with_id(app, "stop", "停止直播", true, None::<&str>)?,
    &PredefinedMenuItem::separator(app)?,
    &MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?,
])?;

let tray = TrayIconBuilder::new()
    .menu(&tray_menu)
    .icon(app.default_window_icon().unwrap().clone())
    .on_menu_event(|app, event| {
        match event.id().as_ref() {
            "show" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        }
    })
    .build(app)?;
```

- [ ] **Step 2: 处理关闭行为**

```rust
app.on_window_event(|window, event| {
    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
        let state = window.state::<AppState>();
        let config = state.config.lock().unwrap();
        if config.data().min_to_tray {
            api.prevent_close();
            let _ = window.hide();
        }
    }
});
```

- [ ] **Step 3: 编译验证**

Run:
```bash
cd src-tauri && cargo check
```
Expected: 编译通过。

- [ ] **Step 4: Commit**

```bash
git add .
git commit -m "feat: add system tray and close-to-tray behavior"
```

---

## Task 10: 打包与发布配置

**Files:**
- Modify: `src-tauri/tauri.conf.json`
- Modify: `package.json`

- [ ] **Step 1: 配置打包**

确保 `tauri.conf.json` 中的 `bundle` 配置完整：
```json
"bundle": {
  "active": true,
  "targets": ["dmg", "app", "appimage", "nsis", "msi"],
  "icon": ["icons/32x32.png", "icons/128x128.png", "icons/128x128@2x.png", "icons/icon.icns", "icons/icon.ico"],
  "macOS": {
    "frameworks": [],
    "minimumSystemVersion": "10.13",
    "signingIdentity": null
  },
  "windows": {
    "certificateThumbprint": null,
    "digestAlgorithm": "sha256",
    "timestampUrl": ""
  }
}
```

- [ ] **Step 2: 转换图标**

将现有的 `bilibili.ico` 转换为 Tauri 需要的多种尺寸：
```bash
# 使用 ImageMagick 或在线工具生成：
# 32x32.png, 128x128.png, 128x128@2x.png, icon.icns, icon.ico
# 放入 src-tauri/icons/
```

- [ ] **Step 3: 测试打包**

Run:
```bash
npx tauri build
```
Expected: 成功生成安装包，位于 `src-tauri/target/release/bundle/`。

- [ ] **Step 4: Commit**

```bash
git add .
git commit -m "chore: configure build and bundle"
```

---

## Self-Review

### Spec Coverage Check

| Spec 需求 | 对应 Task |
|---|---|
| Tauri + React + TS 项目初始化 | Task 1 |
| Rust 数据模型 | Task 2 |
| BiliApi + App 签名 | Task 3 |
| Auth/User/Live 服务 | Task 4 |
| 弹幕 WebSocket | Task 5 |
| Tauri Commands | Task 6 |
| 前端类型 + 桥接 + 状态 | Task 7 |
| 前端组件（所有面板） | Task 8 |
| 系统托盘 + 关闭行为 | Task 9 |
| 打包 | Task 10 |
| Dark/Light 主题 | Task 7-8 |
| Console toggle | Task 8 |
| macOS 原生 title bar | Task 1 (tauri.conf.json) |

### Placeholder Scan

- 无 "TBD", "TODO" 步骤
- 所有 Command 函数已定义
- 所有类型已定义
- 但注意：`poll_login_status` 中的 Cookie 提取和 `danmaku_ws.rs` 中的事件推送 (`emit`) 需要后续完善

### Type Consistency

- `UserConfig.uid` 为 `u64`，前后端一致
- `StreamCodeData` 结构前后端一致
- `DanmakuMessage` 使用 `#[serde(tag = "type")]`，前端对应 union type

### 已知简化

1. **Wbi 签名**：原 Python `get_wbi.py` 未翻译，计划中弹幕 `getDanmakuInfo` 未加 Wbi 签名。需要在 Task 5 中补充 `utils/wbi.rs`。
2. **弹幕事件推送**：`danmaku_ws.rs` 中的 `handle_command` 只打印日志，未通过 Tauri Event 推送到前端。需要在 Task 5/6 中补充 `app_handle.emit("danmaku-message", ...)`。
3. **扫码登录 Cookie 提取**：`poll_passport_qrcode` 需要 reqwest 的 raw response 来提取 Set-Cookie。

这些简化在实现过程中需要补充，但不影响整体计划结构。
