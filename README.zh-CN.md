<p align="center">
  <img src="src-tauri/icons/icon.png" width="128" height="128" alt="SecKeyBox logo">
</p>

<h1 align="center">SecKeyBox</h1>

<p align="center">
  面向开发者的本地优先密码管理器
</p>

<p align="center">
  <a href="https://github.com/EthanCheung-gh/SecKeyBox/releases"><img src="https://img.shields.io/badge/release-v0.2.0-blue?style=flat-square" alt="Release"></a>
  <img src="https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey?style=flat-square" alt="Platforms">
  <a href="license"><img src="https://img.shields.io/badge/license-Apache--2.0-green?style=flat-square" alt="License"></a>
</p>

<p align="center">
  <a href="README.md">English</a> · 简体中文
</p>

---

## 为什么选择 SecKeyBox

开发者的凭证并不符合经典密码管理器"网站 + 密码"的模型。SecKeyBox 内置八类专用条目——从数据库连接、SSH 服务器，到云凭证、许可证和 SMTP 账号——全部保存在本地加密的 SQLite 数据库里，与任何服务器零交互。

**安装包不包含任何内置凭证。** 首次启动会引导你创建保险库并自设主密码。无账号、无云端、无遥测。

## 功能

| 领域 | 说明 |
|------|------|
| 8 类凭证条目 | 账号、API Key、环境变量、数据库连接、SSH 服务器、云凭证、许可证、SMTP 账号 |
| 专用字段 | 例如数据库：类型/主机/端口/库名/用户/密码/连接串；SSH：主机/端口/用户/密码/私钥路径/私钥口令 |
| 加密 | 每字段独立 AES-256-GCM 加密 + 随机 nonce；密钥由主密码经 Argon2id 派生（m=64MB，t=3，p=4） |
| 主密码修改 | 校验当前密码、派生新密钥，并在单个事务内重加密**全部**密文——失败即回滚，库仍可用旧密码打开 |
| 分组管理 | 每类凭证一个内置分组；所有分组（含内置）均可重命名、删除，删除时级联清理组内条目 |
| 搜索 | 侧边栏搜索框实时过滤条目标题与副标题 |
| 导入 / 导出 | JSON 备份导出；导入支持合并 / 替换两种模式（备份中的数据保持密文） |
| 会话安全 | 5 分钟无活动自动锁定；复制敏感值 30 秒后自动清空剪贴板；密钥释放时内存清零 |
| 外观 | 亮色 / 暗色 / 跟随系统主题 |

备注字段按设计以明文存储（便于搜索）；所有密码 / 密钥 / 秘密字段均为密文。

## 安全模型

```
主密码
   │  Argon2id (m=64MB, t=3, p=4)
   ▼
32 字节密钥 ──► AES-256-GCM，每字段独立随机 nonce
   │
   ▼
本地 SQLite 数据库中的敏感字段
```

- 主密码永不离开本机；数据库中仅保存其 PHC 哈希与盐（用于校验）。
- 锁定即销毁内存密钥；解锁需重新派生。
- 修改主密码会在事务内原子轮换全部数据密钥（轮换、空密文、回滚均有[单元测试](src-tauri/src/db/operations.rs)覆盖）。

## 下载

前往[最新 Release](https://github.com/EthanCheung-gh/SecKeyBox/releases/latest) 获取安装包：

| 平台 | 产物 |
|------|------|
| Windows | `.msi` 或 `.exe`（NSIS 安装器） |
| macOS（Apple Silicon） | `.dmg` |
| Linux | `.deb`、`.rpm`、`.AppImage` |

## 从源码构建

前置要求：[Rust](https://rustup.rs) 1.70+、Node.js 18+，以及你所在平台的 [Tauri v2 系统依赖](https://tauri.app/start/prerequisites/)（Debian/Ubuntu 为 `libwebkit2gtk-4.1-dev libgtk-3-dev` 等）。

```bash
git clone https://github.com/EthanCheung-gh/SecKeyBox.git
cd SecKeyBox
npm install

npm run tauri dev     # 以开发模式运行桌面应用
npm run tauri build   # 在 src-tauri/target/release/bundle/ 产出安装包
```

常用脚本：

```bash
npm run dev           # 仅前端，跑在普通浏览器里（内置 mock 替代 Tauri
                      # 后端，附带演示数据，便于纯界面调试）
npm run build         # 类型检查 + 生产构建
cd src-tauri && cargo test   # 单元测试，含重加密与级联删除测试
```

## 项目结构

```
SecKeyBox/
├── src/                    # React 前端
│   ├── components/         # 布局、弹窗、解锁界面、基础组件
│   ├── stores/             # Zustand 状态（vault / ui / settings）
│   ├── lib/                # Tauri API 封装、工具函数
│   └── mocks/              # 仅开发用的 Tauri 后端浏览器 mock
├── src-tauri/              # Rust 后端
│   └── src/
│       ├── commands/       # Tauri 命令（vault / items / 导入导出）
│       ├── crypto/         # Argon2id KDF + AES-256-GCM
│       ├── db/             # 建表、迁移、数据访问
│       └── state.rs        # 内存保险库状态（主密钥）
└── .github/workflows/      # tag 触发的发布 CI（三平台）
```

## 开发计划

- [ ] 浏览器扩展
- [ ] 移动端应用
- [ ] 端到端加密云同步
- [ ] 密码生成器
- [ ] 附件与标签
- [ ] 自动填充

## 贡献

欢迎 Issue 与 Pull Request。较大的改动请先开 Issue 讨论设计。

## 许可证

[Apache License 2.0](license) © 2026 EthanCheung
