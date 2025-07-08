# Discord Bot 开发指南

使用 Rust 构建 Discord 机器人的完整指南，基于 Serenity、Poise 和 Sea-ORM。

## 架构概览

这个 Discord 机器人展示了现代 Rust 模式，用于构建可扩展、可维护的机器人，具有清晰的关注点分离。

### 核心架构模式

#### 1. **工作空间结构**

```
dog-bot/
├── src/           # 主应用程序代码
├── entities/      # 数据库实体定义（工作空间成员）
├── migration/     # 数据库迁移（工作空间成员）
└── Cargo.toml     # 工作空间配置
```

**主要优势：**

- 模块化代码组织
- 数据库组件独立编译
- 应用逻辑与数据层清晰分离

#### 2. **分层架构**

```
┌─────────────────────────────────────────┐
│              命令层 (Commands)           │  <- 用户交互的斜杠命令
├─────────────────────────────────────────┤
│              处理器层 (Handlers)         │  <- Discord 事件处理
├─────────────────────────────────────────┤
│              服务层 (Services)           │  <- 业务逻辑和数据访问
├─────────────────────────────────────────┤
│              数据库层 (Database)         │  <- ORM 实体和迁移
└─────────────────────────────────────────┘
```

**目录结构：**

- `src/commands/` - 斜杠命令和用户交互
- `src/handlers/` - Discord 事件处理器（消息、表情反应等）
- `src/services/` - 业务逻辑和数据访问模式
- `src/database.rs` - 数据库连接和实例管理

#### 3. **通过 Context 进行依赖注入**

```rust
// 全局配置（保留）
pub static BOT_CONFIG: LazyLock<Config> = LazyLock::new(|| {
    // 从文件 + 环境变量加载配置
});

// 数据库实例通过 Context 传递
pub struct BotData {
    pub database: BotDatabase,
}

// 在 Poise 框架中使用
type Context<'a> = poise::Context<'a, BotData, BotError>;
```

**优势：**

- 更灵活的依赖管理，便于测试
- 避免全局状态的潜在问题
- 更清晰的数据流和依赖关系
- 配置仍使用全局变量以保持简单性

## 数据库架构

### 迁移优先开发

#### 1. **创建迁移文件**

```rust
// migration/src/m20220101_000001_create_table.rs
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Messages::Table)
                    .if_not_exists()
                    .col(big_unsigned_uniq(Messages::MessageId).primary_key())
                    .col(big_unsigned(Messages::UserId).not_null())
                    .col(big_unsigned(Messages::GuildId).not_null())
                    .col(big_unsigned(Messages::ChannelId).not_null())
                    .col(timestamp_with_time_zone(Messages::Timestamp))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Messages::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Messages {
    Table,
    MessageId,
    UserId,
    GuildId,
    ChannelId,
    Timestamp,
}
```

#### 2. **注册迁移**

```rust
// migration/src/lib.rs
pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_table::Migration),
        ]
    }
}
```

#### 3. **从数据库生成实体**

**命令：**

```bash
# 首先运行迁移
cargo run --bin migration

# 从现有数据库生成实体
sea-orm-cli generate entity \
    --database-url "sqlite://sqlite.db" \
    --output-dir entities/src/entities
```

**生成的实体：**

```rust
// entities/src/entities/messages.rs
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "messages")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub message_id: i64,
    pub user_id: i64,
    pub guild_id: i64,
    pub channel_id: i64,
    pub timestamp: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
```

#### 4. **服务层模式**

```rust
// src/services/messages.rs
use entities::messages::*;
use sea_orm::*;

pub struct MessageService<'a> {
    db: &'a BotDatabase,
}

impl<'a> MessageService<'a> {
    pub fn new(db: &'a BotDatabase) -> Self {
        Self { db }
    }
    
    pub async fn record(&self, message_id: MessageId, user_id: UserId, ...) -> Result<(), BotError> {
        let message = ActiveModel {
            message_id: Set(message_id.get() as i64),
            user_id: Set(user_id.get() as i64),
            // ... 其他字段
        };
        
        Entity::insert(message)
            .on_conflict(OnConflict::column(Column::MessageId).do_nothing().to_owned())
            .exec(self.db.inner())
            .await?;
        Ok(())
    }
}
```

**数据库访问模式：**

```rust
// 在处理器/命令中使用
use crate::services::MessageService;

async fn handle_message(ctx: &Context, msg: &Message, db: &BotDatabase) {
    MessageService::new(db)
        .record(msg.id, msg.author.id, msg.guild_id.unwrap(), msg.channel_id, msg.timestamp)
        .await
        .unwrap();
}
```

## 框架集成

### Poise 命令框架

```rust
// src/commands/system.rs
use poise::Command;

#[poise::command(slash_command)]
pub async fn ping(ctx: Context<'_>) -> Result<(), BotError> {
    ctx.say("Pong!").await?;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn stats(ctx: Context<'_>) -> Result<(), BotError> {
    // 通过 Context 访问数据库
    let database = &ctx.data().database;
    // 使用数据库进行操作...
    Ok(())
}

pub fn commands() -> Vec<Command<BotData, BotError>> {
    vec![ping(), stats()]
}
```

### 事件处理器模式

```rust
// src/handlers/active.rs
use serenity::all::*;

pub struct ActiveHandler;

#[async_trait::async_trait]
impl EventHandler for ActiveHandler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.bot {
            return;
        }
        
        // 记录消息活动
        if let Some(guild_id) = msg.guild_id {
            // 需要通过某种方式获取数据库实例，例如通过 Context 或参数传递
            // MessageService::new(&database)
            //     .record(msg.id, msg.author.id, guild_id, msg.channel_id, msg.timestamp)
            //     .await
            //     .unwrap_or_else(|e| tracing::error!("记录消息失败: {}", e));
        }
    }
}
```

## 配置管理

### 基于 Figment 的配置

```rust
// src/config.rs
use figment::{Figment, providers::{Format, Json, Env}};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Config {
    pub token: String,
    pub roles: RoleConfig,
    pub tree_holes: HashMap<ChannelId, TreeHoleConfig>,
}

pub static BOT_CONFIG: LazyLock<Config> = LazyLock::new(|| {
    Figment::new()
        .merge(Json::file("config.json"))
        .merge(Env::prefixed("RUST_BOT_"))
        .extract()
        .expect("配置无效")
});
```

## 内存优化：Jemalloc 的重要性

### 为什么 Jemalloc 对 Discord 机器人至关重要

Discord 机器人在处理大量消息、用户和服务器数据时，Serenity 的内置缓存系统会产生大量的内存分配和释放。**使用 Jemalloc 可以将内存占用降低 60-80%**，特别是对于长时间运行的机器人。

### 配置 Jemalloc

```toml
# Cargo.toml
[target.'cfg(not(target_env = "msvc"))'.dependencies]
tikv-jemalloc-ctl = { version = "0.6", features = ["stats", "use_std"] }
tikv-jemallocator = "0.6"
```

```rust
// src/main.rs
#[cfg(not(target_env = "msvc"))]
use tikv_jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;
```

**特别优化的场景：**

- **缓存频繁更新**：用户状态变化、消息缓存
- **长时间运行**：减少内存碎片化
- **高并发处理**：多线程环境下的内存分配优化

## Discord Bot 开发推荐库

### 核心框架

- **`serenity`** - Discord API 封装和事件处理
- **`poise`** - 基于 Serenity 构建的命令框架
- **`tokio`** - 支持多线程的异步运行时

### 数据库与 ORM

- **`sea-orm`** - 现代异步 ORM，支持迁移
- **`sea-orm-migration`** - 数据库迁移框架
- **`sea-orm-cli`** - 代码生成工具

### 配置与环境

- **`figment`** - 灵活的配置管理（JSON + 环境变量）
- **`clap`** - 命令行参数解析
- **`serde`** - 序列化/反序列化

### 性能与内存优化

- **`tikv-jemallocator`** - 内存分配器优化（**强烈推荐**）
- **`tikv-jemalloc-ctl`** - 内存统计和控制
- **`arc-swap`** - 共享状态的原子引用计数
- **`dashmap`** - 并发 HashMap 实现

### 工具库

- **`itertools`** - 迭代器工具和组合器
- **`chrono`** - 日期和时间处理
- **`rand`** - 随机数生成
- **`futures`** - 异步编程工具

### 开发与调试

- **`tracing`** - 结构化日志和诊断
- **`tracing-subscriber`** - 日志输出格式化和过滤
- **`snafu`** - 错误处理和上下文管理

### HTTP 与外部 API

- **`reqwest`** - HTTP 客户端，用于外部 API 调用
- **`serde_json`** - JSON 处理

### 系统监控

- **`sysinfo`** - 系统信息收集
- **`owo-colors`** - 终端颜色输出

## 开发工作流

### 1. **数据库优先开发**

```bash
# 创建新迁移
sea-orm-cli migrate generate create_users_table

# 应用迁移
cargo run --bin migration

# 重新生成实体
sea-orm-cli generate entity --database-url "sqlite://sqlite.db" --output-dir entities/src/entities
```

### 2. **命令开发**

```bash
# 快速语法检查
cargo check

# 运行代码检查器
cargo clippy

# 格式化代码
cargo fmt

# 运行测试
cargo test
```

### 3. **Docker 交叉编译开发**

项目包含 Docker 配置用于 Linux 交叉编译，特别适用于在 macOS 上开发但需要部署到 Linux 服务器的场景。

```bash
# 构建 Docker 镜像
docker build -t dc-bot:latest .

# 启动开发容器
docker-compose up -d

# 进入容器进行开发
docker-compose exec dev bash

# 在容器内编译 Linux 版本
cargo build --release --target x86_64-unknown-linux-gnu
```

**Docker 配置特点：**

- 使用 `linux/amd64` 平台确保兼容性
- 预安装所需的系统依赖（pkg-config、libfreetype6-dev 等）
- 使用 `mold` 链接器加速编译
- 持久化 cargo 缓存以提高构建速度
- 配置 tmpfs 用于临时文件存储

### 4. **数据库测试**

```rust
#[tokio::test]
async fn test_message_service() {
    let db = BotDatabase::new_memory().await.unwrap();
    
    // 为测试应用迁移
    let migrations = Migrator::migrations();
    let manager = SchemaManager::new(db.inner());
    for migration in migrations {
        migration.up(&manager).await.unwrap();
    }
    
    // 测试服务逻辑
    let service = MessageService::new(&db);
    // ... 测试实现
}
```

## 最佳实践

### 1. **错误处理**

- 一致使用 `Result<T, E>`
- 使用 `snafu` 实现合适的错误上下文
- 在适当的级别记录错误

### 2. **性能优化**

- 使用 `LazyLock` 进行昂贵的初始化
- 实现合适的数据库索引
- **必须使用 Jemalloc** 来优化内存使用
- 在高吞吐量场景下使用连接池

### 3. **安全性**

- 永远不要记录敏感数据（令牌、某些情况下的用户 ID）
- 实现合适的基于角色的访问控制
- 使用环境变量存储机密信息

### 4. **代码组织**

- 将关注点分离到不同的模块
- 使用结构体而非 trait 来组织服务逻辑
- 通过 Context 传递依赖而非全局变量
- 实现全面的测试

### 5. **内存管理**

- 监控内存使用情况，特别是在缓存大量数据时
- 定期清理不必要的缓存数据
- 使用适当的数据结构来最小化内存占用

### 6. **并发优化**

对于需要处理大量异步操作的场景，**强烈推荐使用 `FuturesOrdered` 或 `FuturesUnordered`** 来并发化处理：

```rust
use futures::stream::{FuturesUnordered, StreamExt};

// 并发处理多个异步操作
async fn process_multiple_messages(messages: Vec<Message>) -> Vec<Result<(), Error>> {
    let futures = messages
        .into_iter()
        .map(|msg| async move {
            // 处理单个消息的异步操作
            process_message(msg).await
        })
        .collect::<FuturesUnordered<_>>();
    
    futures.collect().await
}

// 对于需要保持顺序的场景
use futures::stream::FuturesOrdered;

async fn process_messages_ordered(messages: Vec<Message>) -> Vec<Result<(), Error>> {
    let futures = messages
        .into_iter()
        .map(|msg| process_message(msg))
        .collect::<FuturesOrdered<_>>();
    
    futures.collect().await
}
```

**并发优化的关键场景：**

- **批量数据库操作**：同时处理多个数据库查询
- **外部 API 调用**：并发调用多个外部服务
- **Discord API 操作**：批量发送消息、更新用户状态
- **文件处理**：并发处理图片生成、文件上传

**性能提升：**

- 对于 I/O 密集型操作，并发处理可以带来 **3-10 倍** 的性能提升
- 特别适用于处理大量 Discord 事件的机器人
- 避免因单个慢操作阻塞整个事件处理流

这种架构为构建可扩展的 Discord 机器人提供了坚实的基础，具有合适的关注点分离、可维护的代码结构和强大的数据持久化能力。通过使用 Jemalloc，可以显著减少内存占用，特别是对于处理大量 Discord 缓存数据的机器人。
