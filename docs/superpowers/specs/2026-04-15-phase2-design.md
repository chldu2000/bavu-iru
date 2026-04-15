# Phase 2: 密码生成器 + 搜索分组 — 设计文档

**日期**: 2026-04-15
**状态**: 已确认

---

## 1. 概述

Phase 2 目标是完善密码管理器的日常使用核心体验。实现顺序：

```
① 密码生成器 → ② 密码强度 → ③ 搜索增强 → ④ 文件夹/标签 → ⑤ 收藏 → ⑥ UI 打磨
```

### 关键设计决策

| 决策项 | 选择 | 理由 |
|--------|------|------|
| 密码生成器 UI | 弹出面板 (Popover) | 空间充裕，可展示完整配置和强度条 |
| 分组系统 | 文件夹 + 标签 | 文件夹主分类 + 标签交叉筛选，最灵活 |
| 密码强度评估 | Rust 端实现 | 密码不离开后端，无前端依赖 |
| 实现策略 | 功能优先，逐步叠加 | 每完成一个功能就可用 |

---

## 2. 密码生成器 (Popover)

### Rust 后端

**新增文件**: `src-tauri/src/commands/generator.rs`

```rust
struct GeneratorOptions {
    length: usize,           // 8-64, 默认 20
    uppercase: bool,         // 默认 true
    lowercase: bool,         // 默认 true
    digits: bool,            // 默认 true
    special: bool,           // 默认 true
    exclude_chars: String,   // 排除的字符，默认空
}

#[tauri::command]
fn generate_password(options: GeneratorOptions) -> Result<String, AppError>
```

- 使用 `rand::thread_rng()` 生成密码
- 字符池在 Rust 侧根据配置构建
- 确保至少包含每种选中字符类型的一个字符

### 前端

**新增组件**: `src/lib/components/PasswordGenerator.svelte`

- Popover 定位在密码输入框的骰子按钮旁
- 配置面板：长度滑块、字符类型开关、排除字符输入
- 生成结果实时预览
- "使用"按钮将密码填入表单字段
- "重新生成"按钮刷新密码

**修改**: `PasswordField.svelte` — 增加骰子图标按钮，点击触发 Popover

---

## 3. 密码强度评估 (Rust)

### Rust 后端

**新增文件**: `src-tauri/src/crypto/strength.rs`

```rust
struct StrengthResult {
    score: u8,         // 0-4 (很弱/弱/一般/强/很强)
    label: String,     // 中文描述
    feedback: String,  // 改进建议
}

fn evaluate_strength(password: &str) -> StrengthResult
```

评估维度：
- 长度评分 (< 8: 0分, 8-11: 1分, 12-15: 2分, 16-19: 3分, ≥20: 4分)
- 字符类型多样性（大写、小写、数字、特殊字符各加分）
- 连续字符检测（abc, 123 扣分）
- 重复字符检测（aaa, 111 扣分）
- 常见密码黑名单（内置 Top 1000，命中直接 0 分）

**新增文件**: `src-tauri/src/commands/strength.rs`

```rust
#[tauri::command]
fn evaluate_password_strength(password: String) -> Result<StrengthResult, AppError>
```

- 创建/更新条目时自动计算强度，存储到 `entries.strength`

### 前端

**新增组件**: `src/lib/components/PasswordStrength.svelte`

- 5 段颜色条（红/橙/黄/绿/深绿）+ 文字标签
- 在 PasswordGenerator Popover 内和 EntryForm 密码字段旁显示
- 输入时实时调用后端评估

---

## 4. 搜索增强

保持前端过滤方案（当前条目量级前端足够），增强筛选能力：

### 前端改动

**修改**: `EntryList.svelte`

- 当前搜索：按 title 匹配
- 增强为：同时匹配 title、username、url（模糊匹配）
- 搜索结果中高亮匹配文字
- 增加按文件夹/标签筛选（与文件夹标签系统联动）

---

## 5. 文件夹 + 标签系统

### 数据库变更

`folders` 表已存在，新增：

```sql
CREATE TABLE tags (
    id         TEXT PRIMARY KEY,
    name       TEXT NOT NULL,
    color      TEXT DEFAULT '#6366f1',  -- 默认靛蓝色
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE entry_tags (
    entry_id TEXT NOT NULL,
    tag_id   TEXT NOT NULL,
    PRIMARY KEY (entry_id, tag_id),
    FOREIGN KEY (entry_id) REFERENCES entries(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);
```

`entries` 表新增字段：

```sql
ALTER TABLE entries ADD COLUMN is_favorite INTEGER DEFAULT 0;
```

### Rust 后端

**新增文件**: `src-tauri/src/commands/folders.rs`

```rust
#[tauri::command]
fn create_folder(name: String, parent_id: Option<String>) -> Result<Folder, AppError>

#[tauri::command]
fn rename_folder(id: String, name: String) -> Result<Folder, AppError>

#[tauri::command]
fn delete_folder(id: String) -> Result<(), AppError>

#[tauri::command]
fn list_folders() -> Result<Vec<Folder>, AppError>
```

**新增文件**: `src-tauri/src/commands/tags.rs`

```rust
#[tauri::command]
fn create_tag(name: String, color: Option<String>) -> Result<Tag, AppError>

#[tauri::command]
fn update_tag(id: String, name: String, color: String) -> Result<Tag, AppError>

#[tauri::command]
fn delete_tag(id: String) -> Result<(), AppError>

#[tauri::command]
fn list_tags() -> Result<Vec<Tag>, AppError>

#[tauri::command]
fn add_tag_to_entry(entry_id: String, tag_id: String) -> Result<(), AppError>

#[tauri::command]
fn remove_tag_from_entry(entry_id: String, tag_id: String) -> Result<(), AppError>
```

**修改**: `src-tauri/src/db/models.rs`

- 新增 `Tag` 模型
- 新增 `EntryTag` 关联模型
- `Folder` 增加 `entry_count` 字段（查询时计算）
- `Entry` 增加 `tags: Vec<Tag>` 和 `is_favorite: bool` 字段

**修改**: `src-tauri/src/db/repository.rs`

- 新增文件夹 CRUD 操作
- 新增标签 CRUD 操作
- 新增 entry_tags 关联操作
- 修改 entry 查询：JOIN tags，返回完整条目（含标签列表）
- 修改 entry 列表排序：`ORDER BY is_favorite DESC, updated_at DESC`

### 前端

**新增组件**: `src/lib/components/FolderTree.svelte`

- 左侧栏导航区域
- 显示"所有条目"默认项 + 文件夹列表
- 选中文件夹时过滤条目列表
- 右键菜单：新建文件夹、重命名、删除
- 显示每个文件夹的条目数量

**新增组件**: `src/lib/components/TagCloud.svelte`

- 左侧栏标签区域
- 带颜色标记的标签列表
- 点击标签过滤条目
- 新增/编辑标签的内联操作

**修改**: `EntryForm.svelte`

- 新增文件夹选择下拉框
- 新增标签多选组件（可创建新标签）

**修改**: `EntryDetail.svelte`

- 显示条目所属文件夹和标签

**修改**: `EntryList.svelte`

- 整合文件夹/标签筛选状态
- 收藏条目显示星标图标

**修改**: `+page.svelte` (主页面)

- 左侧栏重构为：搜索 → 文件夹导航 → 标签 → 条目列表

---

## 6. 收藏/置顶

### 数据库

`entries.is_favorite` 字段（同上）

### Rust 后端

**修改**: `src-tauri/src/commands/entries.rs`

```rust
#[tauri::command]
fn toggle_favorite(entry_id: String) -> Result<Entry, AppError>
```

- 列表查询自动按 `is_favorite DESC` 排序

### 前端

**修改**: `EntryList.svelte`

- 收藏条目前显示实心星标 ⭐
- 未收藏条目 hover 时显示空心星标
- 点击星标切换收藏状态

---

## 7. UI 打磨

### 复制反馈

- 复制操作后显示 toast 提示（"已复制到剪贴板"）
- 复制按钮短暂变为 ✓ 图标

### 搜索高亮

- 搜索匹配的文字在列表中高亮显示（加粗或背景色标记）

### 整体布局

左侧栏结构（从上到下）：

```
┌─────────────────────┐
│ 🔍 搜索框           │
├─────────────────────┤
│ 📁 所有条目 (12)    │
│ 📁 工作 (5)         │
│ 📁 个人 (4)         │
│ 📁 金融 (3)         │
│   + 新建文件夹      │
├─────────────────────┤
│ 🏷️ 社交  🏷️ 邮箱   │
│ 🏷️ 开发  🏷️ 购物   │
│   + 新建标签        │
├─────────────────────┤
│ ⭐ Twitter          │
│ ⭐ GitHub           │
│    Slack             │
│    公司邮箱          │
│    ...               │
└─────────────────────┘
```

---

## 8. 不在范围内

以下明确排除在 Phase 2 之外：

- 剪贴板定时清除（Phase 3）
- 自动锁定（Phase 3）
- 导入/导出（Phase 4）
- 浏览器扩展（Phase 5）
- TOTP、多保险库、生物识别（Phase 6）
- 端到端加密集成（Phase 1 遗留，独立处理）

---

## 9. 新增文件清单

### Rust 后端

| 文件 | 说明 |
|------|------|
| `src-tauri/src/commands/generator.rs` | 密码生成命令 |
| `src-tauri/src/commands/strength.rs` | 强度评估命令 |
| `src-tauri/src/commands/folders.rs` | 文件夹 CRUD 命令 |
| `src-tauri/src/commands/tags.rs` | 标签 CRUD 命令 |
| `src-tauri/src/crypto/strength.rs` | 强度评估算法 |

### 前端

| 文件 | 说明 |
|------|------|
| `src/lib/components/PasswordGenerator.svelte` | 密码生成器 Popover |
| `src/lib/components/PasswordStrength.svelte` | 强度指示条 |
| `src/lib/components/FolderTree.svelte` | 文件夹导航 |
| `src/lib/components/TagCloud.svelte` | 标签云 |
| `src/lib/components/Toast.svelte` | 操作反馈提示 |

### 修改文件

| 文件 | 变更 |
|------|------|
| `src-tauri/src/db/models.rs` | Tag 模型，Entry 增加 tags/is_favorite |
| `src-tauri/src/db/repository.rs` | 文件夹/标签/收藏查询 |
| `src-tauri/src/commands/entries.rs` | toggle_favorite |
| `src-tauri/src/commands/mod.rs` | 注册新命令 |
| `src-tauri/src/crypto/mod.rs` | 导出 strength 模块 |
| `src-tauri/src/lib.rs` | 注册新 Tauri 命令 |
| `src/lib/components/EntryList.svelte` | 搜索增强 + 收藏星标 + 筛选 |
| `src/lib/components/EntryDetail.svelte` | 显示文件夹/标签/收藏 |
| `src/lib/components/EntryForm.svelte` | 集成生成器 + 文件夹/标签选择 |
| `src/lib/components/PasswordField.svelte` | 骰子按钮触发 Popover |
| `src/lib/stores/entries.ts` | 新增文件夹/标签/收藏操作 |
| `src/lib/utils/tauri.ts` | 新增 IPC 命令封装 |
| `src/routes/+page.svelte` | 左侧栏布局重构 |
