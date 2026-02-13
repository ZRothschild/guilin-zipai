# 桂林字牌 - Guilin Paizi

基于 deep-research-report.md 实现的桂林字牌游戏。

## 项目结构

```
guilin-zipai/
├── Cargo.toml              # Workspace 配置
├── Trunk.toml              # 前端构建配置
├── crates/
│   ├── core/               # 核心对局引擎
│   │   ├── src/
│   │   │   ├── lib.rs      # 库入口
│   │   │   ├── card.rs     # 牌张定义（80张）
│   │   │   ├── deck.rs     # 牌堆、洗牌
│   │   │   ├── hand.rs     # 手牌管理
│   │   │   ├── meld.rs     # 吃碰扫牌型
│   │   │   ├── game.rs     # 游戏状态机
│   │   │   ├── player.rs   # 玩家结构
│   │   │   ├── error.rs    # 错误类型
│   │   │   └── constants.rs # 游戏常量
│   │   └── Cargo.toml
│   ├── skills/             # 技能系统
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── trigger.rs  # 触发器
│   │   │   ├── effect.rs   # 效果定义
│   │   │   └── skills.rs   # 12种技能实现
│   │   └── Cargo.toml
│   ├── economy/            # 经济系统
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── currency.rs # 欢乐豆
│   │   │   ├── ranking.rs  # Elo段位
│   │   │   └── settlement.rs # 结算
│   │   └── Cargo.toml
│   ├── server/             # WebSocket服务器
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── server.rs   # 服务器主逻辑
│   │   │   ├── room.rs     # 房间管理
│   │   │   ├── message.rs  # 协议定义
│   │   │   ├── anti_cheat.rs # 反作弊
│   │   │   └── handler.rs
│   │   └── Cargo.toml
│   └── client/             # Yew前端
│       ├── src/
│       │   ├── lib.rs
│       │   ├── main.rs     # 应用入口
│       │   ├── components/ # UI组件
│       │   ├── pages/      # 页面
│       │   ├── services/   # WebSocket服务
│       │   └── models.rs
│       ├── index.html
│       └── Cargo.toml
├── static/
│   ├── index.html
│   └── style.css           # 桂林山水风格CSS
└── README.md
```

## 技术栈

- **后端**: Rust + tokio + tokio-tungstenite (WebSocket)
- **前端**: Rust + Yew (WASM) 
- **桌面**: Tauri (可选)
- **通信**: WebSocket 实时双向通信

## 核心功能

### 桂林字牌规则
- ✅ 80张牌（小写1-10 + 大写壹-拾，各4张）
- ✅ 红牌：二、七、十及其大写
- ✅ 胡牌类型：顺子、二七十、大小三搭、坎、扫、开舵
- ✅ 庄家21张，闲家20张，亮出挡底
- ✅ 吃/碰/扫、放炮必胡规则
- ✅ 胡息计算（10胡起胡）

### 技能系统 (12种)
信息类:
- 听势 - 显示下家牌型倾向
- 观流 - 查看最近3张弃牌
- 算余 - 提示剩余牌张数
- 明算 - 展示牌池剩余总数

容错类:
- 稳手 - 出牌后2秒内可撤回
- 缓冲 - 点炮时减1番
- 重整 - 重排手牌

收益类:
- 稳豆 - 失败损失减5%
- 加码 - 胡牌额外+3%
- 提速 - 6番以上返还+2%

风险类:
- 孤注 - 听牌后±6%波动
- 反压 - 针对对手±5%

### 经济系统
- ✅ 欢乐豆货币
- ✅ 每日签到补助（破产保护）
- ✅ 抽水机制（5%）
- ✅ Elo段位系统（青铜→王者）
- ✅ 结算系统

### 游戏模式
- 匹配对局（1v1 / 2v2）
- 段位赛
- 锦标赛
- 好友房

## 构建与运行

### 前置要求
```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# 安装 wasm 目标
rustup target add wasm32-unknown-unknown

# 安装 Trunk (前端构建工具)
cargo install trunk

# 安装 Tauri CLI (可选，用于桌面端)
cargo install tauri-cli
```

### 开发模式

```bash
# 启动后端服务器
cd crates/server
cargo run

# 启动前端 (新终端)
trunk serve
```

### 构建生产版本

```bash
# 构建前端
trunk build --release

# 构建后端
cargo build --release -p guilin-paizi-server
```

## 游戏界面

- **首页**: 创建/加入房间、游戏模式选择、个人信息
- **房间页**: 玩家列表、房间设置、准备/开始
- **游戏页**: 牌桌、手牌、技能栏、操作按钮

## 架构设计

```
客户端 (Yew + WASM)
    ↕ WebSocket
服务端 (tokio + tungstenite)
    ↕
游戏房间管理
    ↕
对局引擎 (GameState)
    ↕
技能系统 | 经济系统 | 反作弊
```

## 测试

```bash
# 运行所有测试
cargo test

# 运行特定crate测试
cargo test -p guilin-paizi-core
cargo test -p guilin-paizi-skills
```

## 未来规划

- [ ] Tauri 桌面端封装
- [ ] 移动端适配
- [ ] AI对战模式
- [ ] 更多桂林文化主题皮肤
- [ ] 语音聊天
- [ ] 观战系统

## License

MIT
