# AI Native Browser

> 基于 CDP (Chrome DevTools Protocol) 的 AI 浏览器自动化框架

## 概述

AI Native Browser 是一个用 Rust 构建的浏览器自动化框架，通过 CDP 控制 Chrome 浏览器，提供语义化页面分析和自动化操作能力。

## 特性

- 🌐 **CDP 集成** - 通过 Chrome DevTools Protocol 控制浏览器
- 📊 **语义解析** - 将 DOM 转换为可操作的语义树
- 🔍 **流量监控** - 追踪网络请求和响应
- 🚀 **高性能** - Rust + async runtime
- 📦 **二进制分发** - 轻量级可执行文件

## 架构

```
┌─────────────────────────────────────────┐
│            gateway_app                  │
│         (Tauri GUI 应用)                │
└──────────────┬──────────────────────────┘
               │
               ▼
┌─────────────────────────────────────────┐
│            core_engine                  │
├─────────────────────────────────────────┤
│  server/    │  API Server (Axum)       │
│  cdp/       │  Chrome 控制             │
│  parser/    │  语义解析                │
│  inspector/ │  流量监控                │
│  error/     │  错误处理                │
│  performance/│ 性能优化                │
└─────────────────────────────────────────┘
```

## 快速开始

### 构建

```bash
# 克隆项目
cd ai-native-browser

# 开发模式
cargo build

# Release 构建
cargo build --release

# 或使用 build.sh
./build.sh
```

### 运行

```bash
# 启动 API 服务器
./target/release/core-server

# 启动 GUI 应用 (可选)
./target/release/gateway_app
```

### API 端点

| 端点 | 方法 | 描述 |
|------|------|------|
| `/health` | GET | 健康检查 |
| `/v1/status` | GET | 获取状态 |
| `/v1/sense` | GET | 语义页面分析 |
| `/v1/act` | POST | 执行操作 |
| `/v1/browser/start` | POST | 启动浏览器 |
| `/v1/browser/stop` | POST | 停止浏览器 |
| `/v1/human_bridge` | POST | 请求人工介入 |

### 使用示例

```bash
# 启动浏览器
curl -X POST http://127.0.0.1:9222/v1/browser/start

# 获取页面语义树
curl "http://127.0.0.1:9222/v1/sense?url=https://example.com"

# 点击元素
curl -X POST http://127.0.0.1:9222/v1/act \
  -H "Content-Type: application/json" \
  -d '{"action": "click", "target_id": 1}'

# 输入文本
curl -X POST http://127.0.0.1:9222/v1/act \
  -H "Content-Type: application/json" \
  -d '{"action": "type", "target_id": 1, "value": "Hello World"}'
```

## 项目结构

```
ai-native-browser/
├── bin/                    # 编译后的二进制文件
├── core_engine/            # 核心引擎库
│   ├── src/
│   │   ├── cdp/           # CDP 客户端
│   │   ├── parser/        # HTML 语义解析
│   │   ├── inspector/     # 流量监控
│   │   ├── server/        # API 服务器
│   │   ├── error.rs       # 错误处理
│   │   └── performance.rs # 性能优化
│   └── tests/             # E2E 测试
├── gateway_app/           # Tauri GUI 应用
│   ├── src/              # 主程序
│   ├── web/              # 前端资源
│   └── icons/            # 应用图标
├── Cargo.toml            # Workspace 配置
└── README.md            # 本文件
```

## 配置

### 环境变量

| 变量 | 默认值 | 描述 |
|------|--------|------|
| `BROWSER_PATH` | 系统默认 | Chrome 路径 |
| `SERVER_HOST` | 127.0.0.1 | 服务器地址 |
| `SERVER_PORT` | 9222 | 服务器端口 |

### 配置文件

编辑 `core_engine/src/lib.rs` 中的 `EngineConfig`:

```rust
pub struct EngineConfig {
    pub browser_path: Option<String>,
    pub user_data_dir: Option<String>,
    pub headless: bool,
    pub sandbox: bool,
    pub host: String,
    pub port: u16,
    pub dom_idle_threshold_ms: u64,
    pub network_idle_threshold_ms: u64,
}
```

## 开发

### 运行测试

```bash
# 单元测试
cargo test --package core_engine

# E2E 测试 (需要先启动服务器)
cargo test --package core_engine --test e2e_api_test
```

### 代码检查

```bash
cargo check
cargo clippy
cargo fmt
```

## 性能

| 指标 | 数值 |
|------|------|
| 核心引擎大小 | 7.6 MB |
| Gateway 应用大小 | 15.5 MB |
| 启动时间 | < 1s |
| 内存占用 | ~50 MB |

## 路线图

- [x] CDP 基础集成
- [x] 语义解析
- [x] 流量监控
- [x] API 服务器
- [x] E2E 测试
- [x] 错误处理强化
- [x] 性能优化
- [x] 二进制打包
- [ ] WebSocket 支持
- [ ] 更多浏览器支持
- [ ] Docker 支持

## 许可证

MIT License
