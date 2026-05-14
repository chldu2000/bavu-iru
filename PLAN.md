# 密码管理器 — Svelte + Tauri 实现计划

## 1. 项目概述

基于 **Svelte (前端)** + **Tauri v2 (后端)** 构建本地优先的密码管理器，所有敏感数据以加密形式存储在本地，不依赖云服务。

---

## 2. 功能需求

### 2.1 核心功能 (P0)

| # | 功能 | 说明 |
|---|------|------|
| 1 | 主密码设置与验证 | 首次使用设置主密码，后续以此解锁保险库 |
| 2 | 密码条目 CRUD | 增删改查：网站、用户名、密码、备注、自定义字段 |
| 3 | 加密本地存储 | 使用主密码派生密钥，加密存储全部条目 |
| 4 | 密码生成器 | 可配置长度、字符集、排除字符，一键生成强密码 |
| 5 | 搜索与分组 | 按标题/域名搜索，支持文件夹/标签分组 |
| 6 | 保险库锁定/解锁 | 手动锁定，需重新输入主密码解锁 |

### 2.2 拓展功能 (P1)

| # | 功能 | 说明 |
|---|------|------|
| 7 | 剪贴板支持 | 一键复制用户名/密码，定时清除剪贴板 |
| 8 | 自动锁定 | 空闲超时自动锁定；系统锁屏时锁定 |
| 9 | 自动填充 | 通过浏览器扩展与桌面应用通信，识别登录表单并填充 |
| 10 | 导入/导出 | 支持 CSV / JSON / 其他密码管理器格式（KeePass、Bitwarden 等） |
| 11 | 密码强度评估 | 实时评估密码强度，标记弱密码/重复密码 |
| 12 | 密码过期提醒 | 为条目设置过期时间，到期提醒更换 |

### 2.3 远期功能 (P2)

| # | 功能 | 说明 |
|---|------|------|
| 13 | TOTP 支持 | 存储和生成基于时间的一次性密码 |
| 14 | 多保险库 | 支持创建多个独立保险库，各自主密码 |
| 15 | 生物识别解锁 | macOS Touch ID / Windows Hello |
| 16 | 浏览器扩展 | 配合自动填充的 Chrome/Firefox/Edge 扩展 |
| 17 | 自毁机制 | 连续 N 次输入错误主密码后擦除数据（可选） |

---

## 3. 技术架构

```
┌─────────────────────────────────────────┐
│              Svelte UI 层                │
│  ┌─────────┐ ┌──────────┐ ┌───────────┐ │
│  │ 条目列表 │ │ 编辑面板  │ │ 生成器    │ │
│  └─────────┘ └──────────┘ └───────────┘ │
│         ↕ Tauri IPC (invoke)             │
├─────────────────────────────────────────┤
│            Tauri Rust 后端               │
│  ┌──────────┐ ┌─────────┐ ┌───────────┐ │
│  │ 加密模块  │ │ 存储层  │ │ 剪贴板    │ │
│  │ (AES-256)│ │ (SQLite)│ │ /系统API  │ │
│  └──────────┘ └─────────┘ └───────────┘ │
└─────────────────────────────────────────┘
        │
   本地加密文件 (vault.db)
```

### 关键技术选型

| 层 | 技术 | 说明 |
|----|------|------|
| 前端框架 | Svelte 5 + SvelteKit (SPA 模式) | 响应式轻量，适合桌面应用 |
| 桌面容器 | Tauri v2 | Rust 后端，体积小，系统 API 丰富 |
| 本地数据库 | SQLite (via rusqlite) | 单文件，成熟稳定 |
| 加密 | RustCrypto: AES-256-GCM | AEAD 认证加密，防篡改 |
| 密钥派生 | Argon2id | 抗 GPU/ASIC，OWASP 推荐 |
| 样式 | TailwindCSS | 快速构建一致 UI |

---

## 4. 安全设计

### 4.1 密钥派生与加密方案

```
用户主密码 (plaintext)
        │
        ▼
  Argon2id KDF
  ├── salt: 32 bytes 随机 (持久化，不保密)
  ├── time_cost: 3
  ├── memory_cost: 64 MiB
  ├── parallelism: 4
  └── output: 256-bit master key
        │
        ▼
  AES-256-GCM 加密
  ├── key: master key
  ├── iv/nonce: 12 bytes 随机 (每条加密独立)
  ├── plaintext: 序列化的条目数据
  └── output: ciphertext + auth_tag
```

### 4.2 安全注意事项

#### 数据存储
- **主密码绝不持久化**：仅在内存中保存派生密钥，锁定/退出立即清除
- **每个加密块独立 nonce**：绝不复用 nonce/IV
- **认证加密 (AEAD)**：AES-256-GCM 提供机密性 + 完整性校验
- **数据库文件整体加密**：不只是字段加密，防止元数据泄露
- **敏感字段零化**：Rust 侧使用 `zeroize` crate，用完即清

#### 运行时安全
- **内存锁定**：使用 `mlock` 防止密钥被换出到磁盘 swap
- **剪贴板定时清除**：复制后 30 秒自动清空剪贴板
- **防截屏**：密码显示区域标记为不可截屏 (Tauri 窗口属性)
- **进程隔离**：不通过命令行参数或环境变量传递密码
- **日志脱敏**：禁止将密码、密钥、解密内容写入日志

#### 主密码
- **强度检查**：设置时强制最低强度要求（长度 ≥ 12，混合字符类型）
- **防暴力破解**：Argon2id 本身就是慢哈希，增加每次尝试成本
- **可选锁定延迟**：连续失败后指数退避

#### 自动锁定
- 用户可配置空闲超时（默认 5 分钟）
- 监听系统锁屏事件，立即锁定
- 应用窗口失焦超时锁定（可配置）

#### 导入/导出
- 导出时必须再次验证主密码
- 导出文件可选加密（重新输入密码保护导出文件）
- 导入前展示预览，让用户确认来源可信

#### 应用分发
- 代码签名：macOS / Windows 均需签名，防止被篡改
- 更新验证：Tauri updater 启用签名验证
- CSP 策略：严格限制前端可加载的资源来源

### 4.3 威胁模型

| 威胁 | 对策 |
|------|------|
| 本地文件被窃取 | AES-256-GCM 加密，无主密码无法解密 |
| 内存 dump | mlock + 及时 zeroize |
| 剪贴板窥探 | 定时清除，使用专用剪贴板通道 |
| 暴力破解主密码 | Argon2id 慢哈希 + 可选锁定延迟 |
| 供应链攻击 | 锁定依赖版本，代码签名 |
| 中间人更新 | Tauri updater 签名验证 |

---

## 5. 分阶段实现计划

### Phase 1：基础框架 + 核心加密 (2 周)

**目标**：搭建项目骨架，实现加密存储和基本 CRUD

- [x] 初始化 Tauri v2 + Svelte 5 项目
- [x] 设计 SQLite 数据库 schema（加密条目表、元数据表）
- [x] 实现 Argon2id 密钥派生模块 (Rust)
- [x] 实现 AES-256-GCM 加密/解密模块 (Rust)
- [x] 实现主密码设置与验证流程
- [x] 实现保险库锁定/解锁 (内存中持有密钥的状态管理)
- [x] 密码条目基本 UI：列表展示 + 新增/编辑/删除
- [x] 集成测试：加密 → 存储 → 解密 → 验证 完整流程

**交付物**：可运行的 MVP，能加密存储和检索密码条目

### Phase 2：密码生成器 + 搜索分组 (1 周)

**目标**：完善日常使用核心体验

- [x] 密码生成器：可配置长度、大小写、数字、特殊字符、排除字符
- [x] 密码强度实时评估（zxcvbn 算法或简化版）
- [x] 搜索功能：按标题、域名、用户名模糊搜索
- [x] 文件夹/标签分组系统
- [x] 收藏/置顶功能
- [x] 密码条目 UI 完善：显示/隐藏密码、一键复制

**交付物**：功能完整的本地密码管理核心

### Phase 3：剪贴板 + 自动锁定 (1 周)

**目标**：提升日常使用便捷性和安全性

- [x] 剪贴板集成：一键复制用户名/密码/其他字段
- [x] 剪贴板定时清除（默认 30 秒，可配置）
- [x] 空闲超时自动锁定（可配置时长）
- [x] 系统锁屏事件监听 → 自动锁定
- [x] 窗口失焦超时锁定
- [x] 系统托盘集成：最小化到托盘、快速锁定

**交付物**：安全且便捷的桌面密码管理器

### Phase 4：导入导出 + 数据管理 (1 周)

**目标**：数据可迁移，支持从其他管理器迁移

- [x] 导出功能：CSV / JSON / 加密备份
- [x] 导入功能：CSV / JSON 通用格式
- [x] KeePass CSV 导入适配
- [x] Bitwarden JSON 导入适配
- [x] Chrome/Firefox CSV 导入适配
- [x] 导入预览：展示即将导入的条目数量和冲突
- [x] 导出时二次密码验证
- [x] 数据完整性检查工具

**交付物**：支持主流格式数据迁移

### Phase 5：自动填充 + 浏览器扩展 (2-3 周)

**目标**：实现浏览器内自动识别并填充密码

- [ ] 设计本地 HTTP/WebSocket 通信协议（桌面应用 ↔ 浏览器扩展）
- [ ] 浏览器扩展基础框架（Manifest V3）
- [ ] 扩展：检测登录表单，提取域名和表单字段
- [ ] 扩展 ↔ 桌面应用通信：请求匹配条目
- [ ] 桌面应用：按域名匹配条目，用户确认后发送
- [ ] 扩展：填充用户名和密码到表单
- [ ] 通信加密：扩展与桌面应用间通信使用端到端加密
- [ ] 用户授权：每次自动填充需用户在桌面应用中确认

**交付物**：完整的浏览器自动填充体验

### Phase 6：高阶功能 + 打磨 (持续)

**目标**：功能完善和用户体验优化

- [ ] TOTP 生成与展示（RFC 6238）
- [ ] 多保险库支持
- [ ] 生物识别解锁 (macOS Touch ID / Windows Hello)
- [ ] 自定义字段支持（文本、隐藏文本、URL 等）
- [ ] 密码过期提醒
- [ ] 暗色/亮色主题
- [ ] 国际化 (i18n)
- [ ] 应用签名与自动更新
- [ ] 性能优化与安全审计

---

## 6. 数据库 Schema 设计

```sql
-- 元数据表：存储 salt、版本等非敏感信息
CREATE TABLE meta (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
-- 存储: kdf_salt, kdf_params, version, created_at, last_modified

-- 条目表：敏感字段以加密 JSON blob 存储
CREATE TABLE entries (
    id           TEXT PRIMARY KEY,          -- UUID v4
    folder_id    TEXT,                      -- 所属文件夹
    title        TEXT NOT NULL,             -- 加密存储
    username     TEXT,                      -- 加密存储
    password     TEXT,                      -- 加密存储
    url          TEXT,                      -- 加密存储
    notes        TEXT,                      -- 加密存储
    custom_fields TEXT,                     -- 加密 JSON
    tags         TEXT,                      -- 加密 JSON array
    strength     INTEGER,                  -- 密码强度评分 (0-4)
    expires_at   TEXT,                      -- ISO8601 可选
    created_at   TEXT NOT NULL,
    updated_at   TEXT NOT NULL,
    FOREIGN KEY (folder_id) REFERENCES folders(id)
);

-- 文件夹表
CREATE TABLE folders (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,              -- 加密存储
    parent_id   TEXT,
    sort_order  INTEGER DEFAULT 0,
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);

-- 加密参数表（每个条目的 nonce 等）
-- 或将 nonce 嵌入加密 blob 头部
```

**加密策略**：条目敏感字段在 Rust 侧序列化为 JSON → AES-256-GCM 加密 → 存储为 Base64 blob。每个字段独立 nonce，嵌入密文头部。

---

## 7. 项目目录结构 (预期)

```
bavu-iru/
├── src-tauri/                  # Tauri Rust 后端
│   ├── src/
│   │   ├── main.rs
│   │   ├── crypto/             # 加密模块
│   │   │   ├── mod.rs
│   │   │   ├── kdf.rs          # Argon2id 密钥派生
│   │   │   ├── cipher.rs       # AES-256-GCM 加解密
│   │   │   └── keyring.rs      # 内存密钥管理
│   │   ├── db/                 # 数据库层
│   │   │   ├── mod.rs
│   │   │   ├── models.rs
│   │   │   └── repository.rs
│   │   ├── commands/           # Tauri IPC 命令
│   │   │   ├── mod.rs
│   │   │   ├── vault.rs        # 保险库操作
│   │   │   ├── entries.rs      # 条目 CRUD
│   │   │   ├── clipboard.rs    # 剪贴板操作
│   │   │   └── import_export.rs
│   │   └── security/           # 安全相关
│   │       ├── mod.rs
│   │       ├── autolock.rs     # 自动锁定
│   │       └── clear.rs        # 安全清除
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src/                        # Svelte 前端
│   ├── lib/
│   │   ├── components/         # UI 组件
│   │   │   ├── VaultList.svelte
│   │   │   ├── EntryEditor.svelte
│   │   │   ├── PasswordGenerator.svelte
│   │   │   └── ...
│   │   ├── stores/             # Svelte stores
│   │   │   ├── vault.ts
│   │   │   ├── entries.ts
│   │   │   └── settings.ts
│   │   └── utils/
│   ├── routes/
│   ├── app.html
│   └── app.css
├── static/
├── package.json
├── svelte.config.js
├── vite.config.ts
├── tailwind.config.js
└── tsconfig.json
```

---

## 8. 关键依赖

### Rust (Cargo.toml)

```toml
[dependencies]
tauri = { version = "2", features = ["tray-icon"] }
tauri-plugin-clipboard-manager = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
rusqlite = { version = "0.31", features = ["bundled"] }
aes-gcm = "0.10"
argon2 = "0.5"
rand = "0.8"
zeroize = { version = "1", features = ["derive"] }
uuid = { version = "1", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }
```

### 前端 (package.json)

```json
{
  "devDependencies": {
    "@sveltejs/kit": "^2",
    "svelte": "^5",
    "@tauri-apps/api": "^2",
    "tailwindcss": "^4",
    "typescript": "^5"
  }
}
```

---

## 9. 验收标准

| 阶段 | 验收条件 |
|------|----------|
| Phase 1 | 可设置主密码，增删改查加密条目，重启后数据可正确解密 |
| Phase 2 | 密码生成器可用，搜索和分组正常工作 |
| Phase 3 | 复制后剪贴板自动清除，闲置超时自动锁定 |
| Phase 4 | 能从至少 3 种格式导入，导出文件可重新导入验证 |
| Phase 5 | 浏览器扩展能检测表单并与桌面应用通信填充 |
| Phase 6 | TOTP 可用，生物识别解锁正常，应用已签名 |
