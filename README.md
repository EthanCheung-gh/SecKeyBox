# SecKeyBox

<p align="center">
  <img src="https://raw.gitcode.com/EthanCheung/ImageGallery/raw/main/icon.png" width="128" height="128" alt="SecKeyBox Logo">
</p>

<p align="center">
  <strong>安全的跨平台密码管理器，专为开发者设计</strong>
</p>

<p align="center">
  <a href="#功能特性">功能特性</a> •
  <a href="#安全特性">安全特性</a> •
  <a href="#安装指南">安装指南</a> •
  <a href="#使用说明">使用说明</a> •
  <a href="#技术栈">技术栈</a>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Tauri-2.x-blue?style=flat-square" alt="Tauri">
  <img src="https://img.shields.io/badge/Rust-1.70+-orange?style=flat-square" alt="Rust">
  <img src="https://img.shields.io/badge/React-18-blue?style=flat-square" alt="React">
  <img src="https://img.shields.io/badge/License-MIT-green?style=flat-square" alt="License">
</p>

---

## 简介

SecKeyBox 是一个现代化的密码管理器，专为开发者和技术人员设计。它不仅支持传统的账号密码管理，还提供 API 密钥、环境变量等开发者常用的凭证管理功能。采用 Rust + Tauri 构建，兼顾安全性与性能。

### 为什么选择 SecKeyBox？

- 🔒 **军工级加密** - Argon2id + AES-256-GCM 双重保护
- ⚡ **极速体验** - Rust 后端，秒级启动
- 🎨 **精美界面** - 支持亮色/暗色主题
- 🚀 **开发者友好** - 专为开发者设计的凭证类型
- 🌍 **跨平台** - Windows、macOS、Linux 全支持

---

## 功能特性

### Phase 1: 核心功能 ✅

| 功能 | 描述 |
|------|------|
| **主密码保护** | Argon2id 密钥派生，保障主密码安全 |
| **账号管理** | 存储网站账号、密码、网址、备注 |
| **分组管理** | 自定义分组，支持嵌套（预留） |
| **搜索排序** | 实时搜索，多种排序方式 |
| **收藏功能** | 标记常用项目，快速筛选 |
| **剪贴板安全** | 复制敏感信息后 30 秒自动清除 |

### Phase 2: 开发者功能 ✅

| 功能 | 描述 |
|------|------|
| **API 密钥管理** | 存储 API Key、端点、认证方式、轮换日期 |
| **环境变量管理** | 多键值对存储，适合项目配置管理 |
| **全局快速搜索** | `Ctrl+Shift+Space` 快速搜索窗口 |
| **内置分组** | Accounts、API Keys、Environment Variables |

### Phase 3: 高级功能 ✅

| 功能 | 描述 |
|------|------|
| **主题切换** | 自动检测系统主题，支持手动切换 |
| **密码强度** | 添加密码时实时显示强度指示 |
| **数据导入导出** | JSON 格式备份，支持合并/替换模式 |
| **自动锁定** | 5 分钟无活动自动锁定 |
| **暗色模式** | 完整的暗色主题支持 |

---

## 安全特性

### 加密方案

```
┌─────────────────────────────────────────────────────┐
│                  SecKeyBox 安全架构                  │
├─────────────────────────────────────────────────────┤
│  主密码                                              │
│     ↓                                                │
│  Argon2id (m=64MB, t=3, p=4)                        │
│     ↓                                                │
│  32字节派生密钥 → AES-256-GCM 加密                  │
│     ↓                                                │
│  敏感数据（密码、API Key、环境变量值）              │
└─────────────────────────────────────────────────────┘
```

### 安全细节

- **密钥派生**: Argon2id，内存成本 64MB，3 次迭代，并行度 4
- **数据加密**: AES-256-GCM，每个字段使用唯一 nonce
- **内存安全**: 使用 `zeroize` 在 drop 时清除密钥
- **本地存储**: SQLite 数据库，加密后的数据存储在本地
- **零知识**: 服务器不存储任何数据，完全本地离线使用

---

## 安装指南

### 系统要求

- **Windows**: Windows 10/11
- **macOS**: macOS 11+ (Big Sur)
- **Linux**: Ubuntu 20.04+, Fedora 34+

### 下载安装

1. 访问 [Releases](https://gitcode.com/EthanCheung/SecKeyBox/releases) 页面
2. 下载对应平台的安装包
3. 运行安装程序

### 从源码构建

#### 环境准备

```bash
# 1. 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 2. 安装 Node.js 18+
# 访问 https://nodejs.org/ 下载安装

# 3. 安装系统依赖
# Windows: 安装 Visual Studio C++ Build Tools
# macOS: xcode-select --install
# Linux: sudo apt install libgtk-3-dev libwebkit2gtk-4.0-dev
```

#### 构建步骤

```bash
# 克隆项目
git clone https://gitcode.com/EthanCheung/SecKeyBox.git
cd SecKeyBox

# 安装依赖
npm install

# 开发模式运行
npm run tauri dev

# 构建发布版本
npm run tauri build
```

构建后的可执行文件位于 `src-tauri/target/release/` 目录。

---

## 使用说明

### 首次使用

1. 启动 SecKeyBox
2. 点击 "Create Vault" 创建新保险库
3. 设置主密码（至少 8 个字符）
4. 保险库初始化完成，自动进入主界面

### 日常使用

#### 添加项目

1. 选择左侧分组（Accounts/API Keys/Environment Variables）
2. 点击右上角 **+** 按钮
3. 填写项目信息
4. 点击保存

#### 查看详情

- 点击中间列表中的项目
- 右侧详情面板显示完整信息
- 点击复制按钮复制敏感信息

#### 快速搜索

- 按 `Ctrl+Shift+Space` 打开快速搜索
- 输入关键词实时搜索
- 选中项目后显示复制选项

#### 导入/导出

1. 点击左下角 "Import/Export" 按钮
2. 选择 Export 导出备份，或 Import 导入数据
3. 导入时选择 Merge（合并）或 Replace（替换）模式

#### 主题切换

- 点击左下角主题切换按钮
- 支持 Light、Dark、System 三种模式

---

## 项目结构

```
SecKeyBox/
├── src/                          # React 前端
│   ├── components/               # 组件
│   │   ├── ui/                   # 基础 UI 组件
│   │   ├── layout/               # 布局组件
│   │   │   ├── Sidebar.tsx       # 左侧边栏
│   │   │   ├── ItemList.tsx      # 中间列表
│   │   │   └── DetailPanel.tsx   # 右侧详情
│   │   ├── modals/               # 弹窗组件
│   │   └── unlock/               # 解锁界面
│   ├── stores/                   # Zustand 状态管理
│   ├── hooks/                    # 自定义 Hooks
│   └── lib/                      # 工具函数
├── src-tauri/                    # Rust 后端
│   ├── src/
│   │   ├── commands/             # Tauri 命令
│   │   ├── crypto/               # 密码学模块
│   │   └── db/                   # 数据库模块
│   └── Cargo.toml
├── docs/                         # 设计文档
├── package.json
└── README.md
```

---

## 技术栈

### 前端

| 技术 | 版本 | 用途 |
|------|------|------|
| React | 18.x | UI 框架 |
| TypeScript | 5.x | 类型安全 |
| Tailwind CSS | 3.x | 样式 |
| Zustand | 5.x | 状态管理 |
| Lucide React | 0.x | 图标库 |
| Vite | 5.x | 构建工具 |

### 后端

| 技术 | 版本 | 用途 |
|------|------|------|
| Tauri | 2.x | 桌面框架 |
| Rust | 1.70+ | 后端语言 |
| Rusqlite | 0.32 | SQLite 数据库 |
| Argon2 | 0.5 | 密钥派生 |
| AES-GCM | 0.10 | 数据加密 |
| Zeroize | 1.x | 内存清零 |

---

## 开发计划

### 已实现 ✅

- [x] Phase 1: 核心功能（账号管理、分组、加密）
- [x] Phase 2: 开发者功能（API Key、环境变量、快速搜索）
- [x] Phase 3: 高级功能（主题、密码强度、导入导出）

### 计划中 📋

- [ ] 浏览器扩展（Chrome/Firefox）
- [ ] 移动端应用（iOS/Android）
- [ ] 云同步（端到端加密）
- [ ] 密码生成器
- [ ] 附件支持
- [ ] 标签系统
- [ ] 自动填充

---

## 贡献指南

我们欢迎所有形式的贡献！

### 提交 Issue

- 使用 GitHub Issues 提交 bug 报告
- 描述问题时请提供复现步骤
- 标注功能请求为 `enhancement` 标签

### 提交 PR

1. Fork 本仓库
2. 创建功能分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'Add amazing feature'`)
4. 推送分支 (`git push origin feature/amazing-feature`)
5. 创建 Pull Request

---

## 许可证

[Apache License 2.0](license) © 2026 EthanCheung

---

## 致谢

- [Tauri](https://tauri.app/) - 优秀的 Rust 桌面框架
- [React](https://react.dev/) - 强大的 UI 库
- [Argon2](https://github.com/P-H-C/phc-winner-argon2) - 密码哈希竞赛冠军

---

<p align="center">
  Made with ❤️ by SecKeyBox Team
</p>
