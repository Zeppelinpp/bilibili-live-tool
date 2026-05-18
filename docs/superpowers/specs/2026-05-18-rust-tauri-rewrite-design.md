# B站直播工具 - Rust + Tauri 重构设计文档

## 1. 项目概述

将现有 Python + pywebview + Vue 3 桌面应用重构为 **Rust 后端 + Tauri 2.x 前端** 架构。

**重构动机**：
- 解决 Python + PyInstaller 打包体积大（数十至上百 MB）、启动慢、易反编译的问题
- 学习 Rust 异步生态与 Tauri 桌面开发
- 获得更小的安装包（预计 3-10 MB）、更低的内存占用、原生级的性能

## 2. 技术栈

### 后端（Rust）

| 用途 | Crate | 说明 |
|---|---|---|
| 异步运行时 | `tokio` | 替代 Python asyncio/threading |
| HTTP 客户端 | `reqwest` | 替代 Python requests，支持 async |
| WebSocket 客户端 | `tokio-tungstenite` | 替代 Python aiohttp ws，连接弹幕服务器 |
| 序列化 | `serde` + `serde_json` | 替代 Python json |
| 配置持久化 | `serde` + `toml` | 替代 Python 文件读写，配置存于系统标准目录 |
| 跨平台目录 | `directories` | 获取系统配置/数据目录（XDG/Linux、AppData/Windows、Application Support/macOS） |
| 错误处理 | `anyhow` + `thiserror` | 结构化错误类型 |
| 日志 | `tracing` + `tracing-subscriber` | 替代 Python logging，支持结构化日志 |
| 定时任务 | `tokio::time` | 心跳、轮询等 |

### 前端

| 用途 | 技术 | 说明 |
|---|---|---|
| 桌面框架 | Tauri 2.x | 替代 pywebview，提供窗口、托盘、系统 API |
| UI 框架 | React 18 + TypeScript | 替代 Vue 3，重写所有组件 |
| 构建工具 | Vite | Tauri 默认 |
| 样式 | TailwindCSS | 原子化 CSS |
| 组件库 | shadcn/ui | 按需引入的精美组件 |
| 状态管理 | React useContext + useReducer | 应用复杂度适中，无需 Redux/Zustand |

## 3. 项目结构

```
bilibili_live_stream_code/
├── src-tauri/                  # Tauri + Rust 后端
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── src/
│   │   ├── main.rs             # 入口：初始化 Tauri App、注册 Command、启动托盘
│   │   ├── lib.rs              # 模块导出
│   │   ├── commands/           # Tauri Commands（暴露给前端的 API）
│   │   │   ├── auth.rs         # 扫码登录
│   │   │   ├── live.rs         # 开播/停播/推流码
│   │   │   ├── danmaku.rs        # 弹幕监控/发送
│   │   │   ├── user.rs         # 用户信息/账户切换
│   │   │   ├── window.rs       # 窗口控制
│   │   │   └── config.rs       # 应用配置
│   │   ├── services/           # 业务逻辑层
│   │   │   ├── bili_api.rs     # B站 HTTP API 封装（reqwest）
│   │   │   ├── danmaku_ws.rs     # 弹幕 WebSocket（tokio-tungstenite）
│   │   │   ├── auth_service.rs # 登录逻辑
│   │   │   ├── live_service.rs # 直播控制
│   │   │   ├── user_service.rs # 用户/多账户管理
│   │   │   └── config_store.rs # 配置读写（toml）
│   │   ├── models/             # 数据模型（serde）
│   │   │   ├── user.rs
│   │   │   ├── live.rs
│   │   │   ├── danmaku.rs
│   │   │   └── config.rs
│   │   └── utils/              # 工具函数
│   │       ├── crypto.rs       # App 签名（MD5）
│   │       ├── mask.rs         # 日志脱敏
│   │       └── wbi.rs          # Wbi 签名参数生成
│   └── icons/
├── src/                        # 前端 React 源码
│   ├── main.tsx
│   ├── App.tsx
│   ├── components/             # 组件（对应原 Vue 组件）
│   │   ├── WindowControls.tsx
│   │   ├── Sidebar.tsx
│   │   ├── StreamPanel.tsx     # 推流码面板
│   │   ├── DanmakuPanel.tsx      # 弹幕面板
│   │   ├── AccountPanel.tsx    # 账户管理
│   │   ├── QrCodeLogin.tsx     # 扫码登录
│   │   ├── ConsolePanel.tsx    # 日志控制台
│   │   └── ui/                 # shadcn/ui 组件
│   ├── hooks/
│   │   ├── useTauri.ts         # Tauri invoke/listen 封装
│   │   ├── useDanmaku.ts         # 弹幕状态与事件监听
│   │   └── useLive.ts          # 直播状态管理
│   ├── context/
│   │   └── AppContext.tsx      # 全局状态（用户、配置、直播状态）
│   ├── types/
│   │   └── api.ts              # 前后端共享类型
│   └── styles/
│       └── globals.css
├── public/
├── package.json
├── vite.config.ts
├── tailwind.config.ts
├── tsconfig.json
└── index.html
```

## 4. 后端模块设计

### 4.1 BiliApi（`services/bili_api.rs`）

封装所有 B站 HTTP API 调用，内部持有 `reqwest::Client` 和 Cookie Jar。

**核心方法**：

- `get_passport_qrcode() -> Result<QrCodeData>` — 获取登录二维码
- `poll_passport_qrcode(key: &str) -> Result<(LoginStatus, CookieJar)>` — 轮询登录状态
- `get_user_info() -> Result<UserInfo>` — 获取用户信息
- `get_room_id_by_uid(uid: u64) -> Result<u64>` — 获取直播间 ID
- `get_area_list() -> Result<Vec<Area>>` — 获取分区列表
- `update_title(room_id: u64, title: &str, csrf: &str) -> Result<()>` — 修改标题
- `update_area(room_id: u64, area_id: u64, csrf: &str) -> Result<()>` — 修改分区
- `start_live(room_id: u64, area_id: u64, csrf: &str) -> Result<StreamCodeData>` — 开播并返回推流码
- `stop_live(room_id: u64, csrf: &str) -> Result<()>` — 停播
- `send_danmaku(room_id: u64, msg: &str, csrf: &str) -> Result<()>` — 发送弹幕
- `get_danmaku_info(room_id: u64) -> Result<DanmakuServerInfo>` — 获取弹幕服务器信息

**App 签名**（B站直播姬）：
```rust
const APP_KEY: &str = "aae92bc66f3edfab";
const APP_SEC: &str = "af125a0d5279fd576c1b4418a3e8276d";

fn app_sign(params: &mut HashMap<String, String>) -> HashMap<String, String> {
    params.insert("appkey".to_string(), APP_KEY.to_string());
    // 排序 + MD5(query + APP_SEC)
}
```

### 4.2 DanmakuWs（`services/danmaku_ws.rs`）

管理弹幕 WebSocket 连接，运行在独立的 tokio task 中。

**职责**：

- 连接 B站弹幕服务器（wss://host:port/sub）
- 发送认证包（operation=7）
- 发送心跳包（operation=2，每30秒）
- 接收并解码数据包（protobuf + brotli/zlib 解压）
- 解析弹幕、礼物、进场消息，通过 Tauri Event 推送到前端
- 断线自动重连（指数退避：5s -> 10s -> 20s -> 60s cap）

**关键结构**：

```rust
pub struct DanmakuService {
    client: reqwest::Client,
    state: Arc<Mutex<SessionState>>,
    tx: mpsc::Sender<DanmakuCommand>,
}

enum DanmakuCommand {
    Connect { room_id: u64 },
    Disconnect,
    SendDanmaku { msg: String },
}
```

### 4.3 配置存储（`services/config_store.rs`）

使用 `directories` 获取系统配置目录，TOML 格式持久化。

**配置路径**：
- Windows: `%APPDATA%/BiliLiveTool/config.toml`
- macOS: `~/Library/Application Support/BiliLiveTool/config.toml`
- Linux: `~/.config/BiliLiveTool/config.toml`

**数据结构**：
```rust
#[derive(Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub current_uid: Option<u64>,
    pub users: HashMap<u64, UserConfig>,
    pub min_to_tray: bool,
}

#[derive(Serialize, Deserialize)]
pub struct UserConfig {
    pub uid: u64,
    pub uname: String,
    pub face: String,
    pub cookie: String,
    pub room_id: String,
    pub csrf: String,
    pub last_title: String,
    pub last_area_id: u64,
    pub last_area_name: Vec<String>,
    pub level: u32,
    pub follower: u32,
    pub following: u32,
}
```

### 4.4 Tauri Commands（`commands/`）

每个 Command 函数对应前端一个调用入口：

```rust
#[tauri::command]
async fn get_login_qrcode(state: tauri::State<'_, AppState>) -> Result<QrCodeData, String> {
    state.auth_service.get_qrcode().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn start_live(
    p_name: Option<String>,
    s_name: Option<String>,
    state: tauri::State<'_, AppState>,
) -> Result<StreamCodeData, String> {
    state.live_service.start(p_name, s_name).await.map_err(|e| e.to_string())
}
```

**完整 Command 列表**：

| Command | 参数 | 返回值 |
|---|---|---|
| `get_login_qrcode` | - | `QrCodeData` |
| `poll_login_status` | `key: String` | `LoginResult` |
| `load_saved_config` | - | `Option<UserConfig>` |
| `refresh_current_user` | - | `UserConfig` |
| `get_account_list` | - | `Vec<UserConfig>` + `current_uid` |
| `switch_account` | `uid: u64` | `UserConfig` |
| `logout` | `uid: u64` | `()` |
| `get_partitions` | - | `HashMap<String, Vec<String>>` |
| `update_title` | `title: String` | `()` |
| `update_area` | `p_name, s_name: String` | `()` |
| `start_live` | `p_name?, s_name?` | `StreamCodeData` |
| `stop_live` | - | `()` |
| `start_danmaku_monitor` | - | `()` |
| `stop_danmaku_monitor` | - | `()` |
| `send_danmaku` | `msg: String` | `SendDanmakuResult` |
| `window_min` | - | `()` |
| `window_max` | - | `bool` |
| `window_close` | - | `()` |
| `window_drag` | `x, y: i32` | `()` |
| `get_app_config` | - | `AppConfig` |
| `set_app_config` | `key, value` | `()` |
| `get_version` | - | `String` |

## 5. 前后端通信设计

### 5.1 前端 -> 后端（Tauri invoke）

前端通过 `invoke('command_name', args)` 调用 Rust Command。

```typescript
// src/hooks/useTauri.ts
import { invoke } from '@tauri-apps/api/core';

export async function startLive(pName?: string, sName?: string) {
  return await invoke<StreamCodeData>('start_live', { p_name: pName, s_name: sName });
}
```

### 5.2 后端 -> 前端（Tauri Event）

后端通过 `app_handle.emit(event_name, payload)` 向前端推送事件。

**事件列表**：

| Event | Payload | 触发时机 |
|---|---|---|
| `danmaku-message` | `DanmakuMsg \| GiftMsg \| InteractMsg` | 收到弹幕/礼物/进场 |
| `backend-log` | `String` | 后端产生日志 |
| `app-shown` | `null` | 托盘点击显示窗口 |
| `app-hidden` | `null` | 窗口最小化到托盘 |
| `tray-live-started` | `StreamCodeData` | 托盘快捷开播成功 |
| `tray-live-stopped` | `null` | 托盘快捷停播成功 |
| `tray-live-error` | `String` | 托盘开播失败 |

前端监听：
```typescript
import { listen } from '@tauri-apps/api/event';

listen('danmaku-message', (event) => {
  const msg = event.payload;
  // 更新弹幕列表
});
```

## 6. 系统托盘

使用 Tauri 2.x 的原生托盘 API（`tauri::tray::TrayIconBuilder`）实现，替代 Python 的 pystray/Qt。

**托盘菜单项**：
- 显示主界面
- 开始直播
- 停止直播
- 退出程序

**行为**：
- 点击关闭按钮时，若配置 `min_to_tray=true`，窗口隐藏到托盘而非退出
- 托盘图标点击可恢复窗口

## 7. UI/UX 设计

### 7.1 设计语言

- **Dieter Rams 原则**：功能优先，每个元素都有明确目的，无装饰性元素，排版即信息层级
- **Nordic 风格**：温暖的灰色调（`stone` 色系），大量留白，克制、自然、舒适
- **macOS 原生融合**：使用系统原生 title bar，不突兀

### 7.2 配色系统

基于 Tailwind `stone` 色系，双主题：

| Token | Light Mode | Dark Mode |
|---|---|---|
| 背景 | `stone-50` / 白色 | `stone-950` / 近黑 |
| 卡片 | `stone-100` | `stone-900` |
| 边框 | `stone-200` | `stone-800` |
| 主文字 | `stone-800` | `stone-200` |
| 次要文字 | `stone-500` | `stone-400` |
| 强调按钮 | `stone-800` 白字 | `stone-100` 黑字 |
| 次要按钮 | `stone-100` 边框 | `stone-900` 边框 |
| 成功 | `green-600` | `green-400` |
| 警告 | `amber-600` | `amber-400` |
| 礼物 | `amber-600` | `amber-500` |

### 7.3 字体与排版

- 字体：`-apple-system, BlinkMacSystemFont, 'SF Pro Text', 'Segoe UI', Roboto, sans-serif`
- 等宽：`SF Mono, Consolas, monospace`
- 层级：`11px` 小标签（大写+加宽字距）→ `13px` 内容 → 无大标题，靠留白和卡片分组区分

### 7.4 布局

- **窗口**：900×700（比之前略小，更紧凑）
- **Title Bar**：macOS 原生（`decorations: true`），红绿灯按钮 + 居中标题，右侧放主题切换和 console toggle 按钮
- **Sidebar**：208px 宽，顶部用户卡片，中间导航（图标+文字），底部直播状态
- **Content**：主内容区 + 可 toggle 的底部 Console（默认 120px，可收起至 0）

### 7.5 窗口行为

- macOS 原生 title bar，无需自定义拖拽区域
- 启动时居中显示
- 关闭行为由 `min_to_tray` 配置决定

## 8. 关键流程

### 8.1 获取推流码

```
前端: 点击"开始直播"
  → invoke('start_live', { p_name, s_name })
  → Rust LiveService::start()
    1. 获取时间戳 (report/click/now)
    2. 获取 App 版本 (app-blink/liveVersionInfo) + App签名
    3. 调用开播 API (room/v1/Room/startLive) + App签名
    4. 解析返回: rtmp1, rtmp2, srt
  → 返回 StreamCodeData { rtmp1, rtmp2, srt }
  → 前端显示推流地址和推流码
```

### 8.2 弹幕连接

```
前端: 进入弹幕面板 / 开播后自动连接
  → invoke('start_danmaku_monitor')
  → Rust DanmakuService::connect(room_id)
    1. 获取弹幕服务器信息 (getDanmakuInfo) + Wbi签名
    2. 获取 buvid3
    3. WebSocket 连接 (wss://host/sub)
    4. 发送认证包
    5. 启动心跳任务 + 接收任务
  → 收到消息后 emit('danmaku-message', payload)
  → 前端 listen 并更新列表
```

## 9. 错误处理策略

- **HTTP 请求错误**：`reqwest::Error` 统一转换为自定义错误类型，Command 层返回 `Result<T, String>` 给前端
- **WebSocket 错误**：断线时自动触发重连，不抛异常到前端，仅推送日志事件
- **配置读写错误**：使用默认配置启动，错误写入日志
- **前端错误**：TypeScript 类型保证 + try/catch 封装 invoke 调用

## 10. 开发计划概要

1. **初始化**：创建 Tauri 2.x 项目模板（React + TypeScript + Tailwind）
2. **后端骨架**：Cargo 依赖配置、Tauri Command 注册、AppState 管理
3. **配置模块**：ConfigStore（TOML 读写、系统目录）
4. **BiliApi**：HTTP 客户端封装、App 签名、二维码登录
5. **用户/账户**：多账户切换、Cookie 管理
6. **直播服务**：开播/停播/分区列表/推流码解析
7. **弹幕服务**：WebSocket 连接、protobuf 解码、Tauri Event 推送
8. **系统托盘**：Tauri TrayIcon + 菜单
9. **前端组件**：逐个迁移原有 Vue 面板为 React 组件
10. **打包测试**：跨平台构建验证

## 11. 风险与注意事项

1. **Wbi 签名**：原 Python `get_wbi.py` 的算法需要逐行翻译为 Rust，确保与 B站服务端兼容
2. **弹幕协议**：protobuf 结构需要重新生成 Rust 代码（`prost` crate），或手写解码逻辑
3. **Cookie 持久化**：Rust 中需要手动管理 Cookie，reqwest 默认不持久化，需自行维护 CookieJar
4. **多账户切换**：切换用户时需清理旧 Cookie 和弹幕连接，确保无状态泄漏
5. **跨平台差异**：Tauri 2.x 的 tray/window API 在不同平台行为略有差异，需分别测试
