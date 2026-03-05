# 桂林字牌游戏启动指南

## 系统要求

- **Rust**: 1.70+ (安装: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)
- **Node.js**: 16+ (可选，用于更好的开发体验)
- **现代浏览器**: Chrome 90+, Firefox 88+, Safari 14+

## 快速开始

### 1. 安装依赖

```bash
# 安装Rust (如果未安装)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 安装Trunk (Rust WASM构建工具)
cargo install trunk

# 添加WASM目标
rustup target add wasm32-unknown-unknown
```

### 2. 启动服务器

```bash
# 终端1: 启动游戏服务器
cd crates/server
cargo run

# 服务器将在 http://127.0.0.1:8080 运行
```

### 3. 启动客户端

```bash
# 终端2: 启动游戏客户端
trunk serve

# 客户端将在 http://127.0.0.1:3000 运行
```

### 4. 开始游戏

1. 打开浏览器访问: http://127.0.0.1:3000
2. 点击"创建房间"
3. 等待其他玩家加入
4. 所有玩家准备后开始游戏

## 使用脚本启动

### Windows
```bash
# 双击 run.bat 或运行:
run.bat
```

### Linux/Mac
```bash
# 添加执行权限
chmod +x run.sh

# 启动
./run.sh [server|client|all]

# 示例
./run.sh server    # 启动服务器
./run.sh client    # 启动客户端
```

## 项目结构

```
guilin-zipai/
├── crates/
│   ├── core/          # 核心游戏逻辑
│   ├── skills/        # 技能系统
│   ├── economy/       # 经济系统
│   ├── server/        # WebSocket服务器
│   └── client/        # Yew WASM客户端
├── static/            # 静态资源
│   └── style.css      # 桂林山水风格CSS
├── Trunk.toml         # WASM构建配置
├── run.sh             # Linux/Mac启动脚本
└── run.bat            # Windows启动脚本
```

## 开发模式

### 服务器开发
```bash
cd crates/server
cargo run              # 运行服务器
cargo test             # 运行测试
cargo check            # 检查编译
```

### 客户端开发
```bash
cd crates/client
trunk serve            # 开发服务器 + 热重载
cargo test             # 运行测试
cargo check            # 检查编译
```

### 全栈开发
```bash
# 终端1: 服务器
cd crates/server && cargo run

# 终端2: 客户端  
trunk serve --watch
```

## 测试

### 运行所有测试
```bash
cargo test
```

### 运行特定crate测试
```bash
cargo test -p guilin-paizi-core      # 核心逻辑测试
cargo test -p guilin-paizi-skills    # 技能系统测试
cargo test -p guilin-paizi-server    # 服务器测试
cargo test -p guilin-paizi-client    # 客户端测试
```

## 构建生产版本

### 构建服务器
```bash
cd crates/server
cargo build --release
# 可执行文件: target/release/guilin-paizi-server.exe
```

### 构建客户端
```bash
trunk build --release
# 输出目录: dist/
```

## 故障排除

### 常见问题

1. **Trunk安装失败**
   ```bash
   # 使用国内镜像
   export RUSTUP_DIST_SERVER=https://mirrors.ustc.edu.cn/rust-static
   export RUSTUP_UPDATE_ROOT=https://mirrors.ustc.edu.cn/rust-static/rustup
   cargo install trunk
   ```

2. **WASM构建失败**
   ```bash
   # 清理缓存
   cargo clean
   trunk clean
   # 重新构建
   trunk build
   ```

3. **端口被占用**
   ```bash
   # 更改端口
   trunk serve --port 3001
   # 或
   cd crates/server && cargo run -- --port 8081
   ```

4. **浏览器无法连接**
   - 检查服务器是否运行: `curl http://127.0.0.1:8080`
   - 检查防火墙设置
   - 确保浏览器支持WebSocket

### 日志查看

```bash
# 服务器日志
RUST_LOG=debug cargo run

# 客户端日志
# 浏览器控制台查看 wasm_logger 输出
```

## 技术栈

- **后端**: Rust + tokio + tokio-tungstenite
- **前端**: Rust + Yew + WASM
- **通信**: WebSocket + JSON
- **样式**: 自定义CSS (桂林山水风格)
- **构建**: Trunk + Cargo

## 许可证

MIT