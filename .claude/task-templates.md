# ZoPlus 子代理任务模板库

本文件包含 ZoPlus 项目常用的任务模板，供子代理任务使用。

---

## 子代理任务模板：动态数据库自省系统重构

### 任务类型
[bug-fix | feature | refactor | docs]

### 任务目标
1. 简要描述

### 自检原则（强制遵守）
1. 任何代码修改前必须先验证假设
2. 涉及表名/字段名的操作必须先连接数据库验证
3. 禁止盲目执行具体指令
4. 输出自检证据

### 动态数据库自省系统使用规范

**必须引入的依赖**：
```rust
use crate::db::metadata::get_cached_metadata;
use crate::db::dynamic::{DynamicSqlBuilder, ZoteroTableCandidates};
```

**ZoteroTableCandidates 常用常量**：
- CREATORS: itemCreators/itemAuthors/itemCreator
- ITEM_DATA: itemData
- ITEM_DATA_VALUES: itemDataValues
- ITEMS: items
- FIELDS: fields
- 其他见 dynamic.rs

**使用模式**：
```rust
let metadata = get_cached_metadata(conn)?;
let dynamic = DynamicSqlBuilder::new(&metadata);

//查找表（支持多候选表名）
if let Some(table) = dynamic.find_table(ZoteroTableCandidates::CREATORS) {
    let sql = format!("SELECT ... FROM {} ...", table);
}

// 检查表是否存在
if dynamic.table_exists("items") { }

// 检查字段是否存在
if dynamic.column_exists("items", "itemID") { }
```

### 验收标准
1. 编译通过
2. 功能测试通过
3. 无硬编码表名/字段名
4. Git 提交信息格式正确

### 约束
1. 不改变业务逻辑
2. 保持错误处理
3. 保持日志输出

---

## 子代理任务模板：ZoPlus 开发任务

### 任务类型
[bug-fix | feature | refactor | docs]

### 任务目标
简要描述要完成的任务

### 项目路径
D:\Desktop\Zoplus

### 项目技术栈
- Tauri + Rust 后端
- React + TypeScript 前端
- rusqlite 访问 Zotero SQLite 数据库
- 动态数据库自省系统

### 自检原则（强制遵守）
1. 任何代码修改前必须先验证假设
2. 禁止盲目执行具体指令
3. 输出自检证据

### 常用命令
```bash
# 编译检查
cargo check

# 前端类型检查
npm run type-check

# Git 状态
git status
```

### Git提交规范
```
[模块] 功能描述

- 可选的详细说明
```

### 验收标准
1. 编译通过
2. 功能正常
3. 无引入新问题

### 输出要求
1. 修改文件清单
2. 核心代码
3. 验证步骤
4. Git 提交信息

---

## 子代理任务模板：文档更新

### 任务类型
docs

### 任务目标
更新/创建项目文档

### 文档更新规范

**需要提交到 Git 的文件**：
- README.md
- api.md
- 设计文档
- CHANGELOG.md

**禁止提交的文件**：
- .env 及任何包含密钥的文件
- node_modules/
- 临时文件

### Git 操作要求
1. 先 `git status` 检查变更
2. `git add` 添加需要提交的文件
3. 创建提交，使用规范格式

### 验收标准
1. 文档内容准确完整
2. Git 操作正确
3. 无敏感信息泄露