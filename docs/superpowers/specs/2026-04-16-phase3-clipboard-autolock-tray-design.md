# Phase 3 设计文档：剪贴板集成 + 自动锁定 + 系统托盘

## 概述

Phase 3 提升密码管理器的日常使用便捷性和安全性，包含三个子系统：
1. 剪贴板集成（一键复制 + 定时清除）
2. 自动锁定（空闲超时 + 系统锁屏 + 窗口失焦）
3. 系统托盘（最小化到托盘 + 快速锁定）

按子系统顺序实现：剪贴板 → 自动锁定 → 系统托盘。每个子系统完成后独立验证。

---

## 1. 剪贴板集成

### 1.1 架构

```
前端 (Svelte)                        Rust 后端
┌─────────────┐    invoke    ┌──────────────────────┐
│ 点击复制按钮 │ ──────────→ │ clipboard_copy(text,  │
│             │              │   sensitive,          │
│             │              │   clear_seconds?)     │
│             │              │   ├─ 写入系统剪贴板    │
│             │              │   └─ sensitive时启动   │
│             │              │      定时器(clear_sec) │
│             │ ←────────── │ event: clipboard-cleared│
│ Toast通知    │   事件回调   │ (由定时器触发)         │
└─────────────┘              └──────────────────────┘
```

### 1.2 行为规格

- **所有复制操作走 Tauri IPC**：替换现有 `navigator.clipboard.writeText()`，通过 `invoke('clipboard_copy')` 由 Rust 侧执行
- **敏感字段区分**：`sensitive: bool` 参数区分。密码、隐藏字段为 `true`（启动定时清除），用户名、URL 为 `false`（直接写入不清除）
- **定时清除在 Rust 侧**：`clipboard_copy` 内部 spawn tokio 定时任务，超时后调用 `clipboard_clear()` 并 emit `clipboard-cleared` 事件
- **前端响应**：复制成功显示"已复制"Toast；收到 `clipboard-cleared` 事件后显示"剪贴板已清除"Toast（两段式反馈）
- **可配置清除时间**：从 settings store 读取 `clipboardClearSeconds`，默认 30 秒，范围 10-120 秒

### 1.3 涉及文件

| 文件 | 改动 |
|------|------|
| `src-tauri/src/commands/clipboard.rs` | 实现 `clipboard_copy`、`clipboard_clear` 命令 |
| `src-tauri/src/commands/mod.rs` | 导出 clipboard 模块 |
| `src-tauri/src/lib.rs` | 注册 clipboard 命令、注册 clipboard-manager 插件 |
| `src/lib/components/EntryDetail.svelte` | 替换 `navigator.clipboard` 为 Tauri IPC 调用 |
| `src/lib/components/PasswordField.svelte` | 同上 |
| `src/lib/stores/settings.ts` | 已有 `clipboardClearSeconds`，需接入实际逻辑 |
| `src/lib/utils/tauri.ts` | 添加 clipboard 相关 IPC 封装函数 |

### 1.4 Rust 命令签名

```rust
#[tauri::command]
async fn clipboard_copy(
    app: tauri::AppHandle,
    text: String,
    sensitive: bool,
    clear_seconds: Option<u64>,
) -> Result<(), AppError>;

#[tauri::command]
async fn clipboard_clear(app: tauri::AppHandle) -> Result<(), AppError>;
```

---

## 2. 自动锁定

### 2.1 架构

```
前端 (Svelte)                         Rust 后端
┌──────────────────┐   invoke    ┌─────────────────────────┐
│ IdleTimer 模块    │ ────────→ │ vault_lock()             │
│ (mousemove/keydown│            │   ├─ zeroize 密钥        │
│  /click 计时器)    │            │   ├─ emit vault-locked   │
│                   │            │   └─ 更新托盘图标状态     │
│ FocusTimer 模块   │ ←──────── │                         │
│ (blur/visibility  │  事件监听   │ LockScreenListener trait │
│  change 计时器)    │            │   ├─ macOS: NSWorkspace  │
│                   │            │   ├─ Linux: *(预留)      │
│ 计时器暂停/恢复    │            │   └─ Windows: *(预留)    │
│ (锁定时暂停)       │            │   └─ 触发 vault_lock()   │
└──────────────────┘            └─────────────────────────┘
```

### 2.2 行为规格

**空闲超时（前端）：**
- Svelte `$effect` 注册 `mousemove`、`keydown`、`click` 事件监听
- 每次交互重置倒计时
- 超时后调用 `vault_lock()`
- 默认 5 分钟，可配置 1-30 分钟或关闭

**系统锁屏（Rust 后端）：**
- 通过 trait 抽象 `LockScreenListener`，按平台条件编译实现
- macOS 实现：监听 `NSWorkspaceSessionDidResignActiveNotification`
- Linux / Windows：预留接口，暂时为 no-op
- 检测到锁屏直接调用 `vault_lock()`

**窗口失焦超时（前端）：**
- 监听 `visibilitychange` 和 `blur` 事件
- 独立计时器，默认关闭，可配置 1-10 分钟
- 与空闲计时器独立运行

**统一锁定流程：**
- 所有触发源（空闲/锁屏/失焦）走同一个 `vault_lock()` 命令
- 锁定后所有计时器暂停
- 锁定后自动隐藏窗口（配合托盘）

### 2.3 LockScreenListener trait 设计

```rust
pub trait LockScreenListener: Send + Sync {
    /// 开始监听系统锁屏事件，锁屏时调用回调
    fn start_listening(&self, on_lock: Box<dyn Fn() + Send + Sync>);
    /// 停止监听
    fn stop_listening(&self);
}

#[cfg(target_os = "macos")]
pub struct MacOSLockScreenListener { /* ... */ }

// Linux/Windows 预留
```

### 2.4 涉及文件

| 文件 | 改动 |
|------|------|
| `src-tauri/src/security/autolock.rs` | 实现 `LockScreenListener` trait + macOS 实现 |
| `src-tauri/src/security/mod.rs` | 导出 autolock 模块 |
| `src-tauri/src/lib.rs` | 在 setup 中初始化锁屏监听器 |
| `src/lib/stores/vault.ts` | 监听 `vault-locked` 事件切换 UI |
| 新增：`src/lib/lib/idleTimer.ts` | 前端空闲检测 + 窗口失焦检测模块 |

---

## 3. 系统托盘

### 3.1 架构

```
Rust 后端
┌─────────────────────────────────────────┐
│  系统托盘 (tray-icon feature)            │
│                                         │
│  托盘图标 (🔒/🔓)  ←── 保险库状态联动    │
│  ├─ 菜单项：                            │
│  │   ├─ "保险库状态: 已解锁/已锁定"  (禁用)│
│  │   ├─ "显示窗口"   → show/hide 窗口    │
│  │   ├─ "立即锁定"   → vault_lock()      │
│  │   ├─ ─────────── (分隔线)             │
│  │   └─ "退出"       → app.exit()        │
│  │                                      │
│  窗口关闭拦截 → window.hide()            │
│  双击托盘图标 → 显示窗口                  │
└─────────────────────────────────────────┘
```

### 3.2 行为规格

- **窗口关闭拦截**：`on_window_event` 回调拦截 `CloseRequested`，调用 `window.hide()` + `event.prevent_close()`
- **双击托盘图标**：显示窗口并聚焦
- **菜单动态更新**：保险库锁定后"立即锁定"变灰色禁用，状态文本切换
- **图标状态联动**：锁定/解锁时更新托盘图标
- **锁定后自动隐藏**：通过自动锁定触发 `vault_lock()` 时同时隐藏窗口，用户从托盘"显示窗口"回来看到锁屏界面

### 3.3 涉及文件

| 文件 | 改动 |
|------|------|
| 新增：`src-tauri/src/tray.rs` | 托盘初始化、菜单构建、事件处理 |
| `src-tauri/src/lib.rs` | 在 setup 中创建托盘、注册窗口关闭拦截 |
| `src-tauri/tauri.conf.json` | 无需修改托盘配置（代码创建） |
| `src-tauri/icons/` | 添加托盘图标文件（locked.png / unlocked.png） |

---

## 4. 设置页面

### 4.1 涉及的配置项

| 设置项 | 默认值 | 范围 | 存储位置 |
|--------|--------|------|----------|
| 空闲自动锁定 | 5 分钟 | 1-30 分钟 / 关闭 | settings store (前端) |
| 剪贴板清除时间 | 30 秒 | 10-120 秒 | settings store (前端) |
| 窗口失焦锁定 | 关闭 | 开/关，1-10 分钟 | settings store (前端) |

### 4.2 涉及文件

| 文件 | 改动 |
|------|------|
| 新增：`src/lib/components/Settings.svelte` | 设置页面组件 |
| `src/lib/stores/settings.ts` | 扩展已有 store，接入实际逻辑 |
| `src/routes/+page.svelte` 或布局 | 添加设置页面入口（齿轮图标） |

---

## 5. 实现顺序

按子系统逐个推进，每步独立验证：

1. **剪贴板集成** — Rust 命令 + 前端替换 + Toast 反馈
2. **自动锁定** — 前端空闲检测 + Rust 锁屏监听 + 窗口失焦
3. **系统托盘** — 托盘初始化 + 窗口关闭拦截 + 图标联动
4. **设置页面** — 统一配置入口，接入上述三个子系统的参数

---

## 6. 关键依赖

| 依赖 | 用途 | 状态 |
|------|------|------|
| `tauri-plugin-clipboard-manager` | 系统剪贴板读写 | 已在 Cargo.toml，未注册 |
| `tauri` feature `tray-icon` | 系统托盘 | 已启用，未使用 |
| `tokio` | 异步定时任务（剪贴板清除） | 需确认是否需要显式添加 |

---

## 7. 设计决策摘要

| 决策点 | 选择 |
|--------|------|
| 系统托盘菜单 | 标准模式：显示窗口 / 立即锁定 / 保险库状态 / 退出 |
| 剪贴板清除反馈 | 两段式 Toast：复制 + 清除各一个提示 |
| 空闲判定方式 | 前端交互监听（mousemove/keydown/click） |
| 关闭按钮行为 | 关闭 = 最小化到托盘 |
| 实现策略 | 按子系统逐个实现（方案 A） |
| 系统锁屏监听 | trait 抽象 + 平台条件编译，macOS 先实现 |
