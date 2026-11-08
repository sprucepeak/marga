# Marga (道 / Mārga) 🧭

> **万流归元，循道而行。**
>
> 让指定的 Windows 应用程序或域名流量单独走代理（底层网络驱动重定向 · 零侵入 · 免 DLL 注入 · 免改写文件）。

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Version](https://img.shields.io/badge/Version-1.0.0-emerald.svg)](package.json)
[![Platform](<https://img.shields.io/badge/Platform-Windows%2010%20%7C%2011%20(x64)-informational.svg>)](https://www.microsoft.com/windows)
[![Tauri](https://img.shields.io/badge/Tauri-v2-orange.svg)](https://v2.tauri.app/)

## 🌟 核心特性 (v1.0.0)

### 1. 零侵入架构 (Zero Intrusion)

- 纯 Windows 内核级网络数据包透明重定向（基于已签名的 WinDivert 驱动）。
- **无需注入 DLL**、无需 API Hook、不修改目标进程与可执行文件，杜绝反作弊拦截及应用崩溃风险。

### 2. 双重分流策略体系

- **按应用程序分流 (Process Routing)**：
  - **原生文件选择**：支持浏览选择任何 `.exe` 执行文件。
  - **多模式路径匹配**：支持完整路径精确匹配、目录前缀及通配符（`*` / `?`）。
  - **动态进程树追踪 (Process Tree)**：自动追踪目标程序衍生出来的子进程（完美支持 Chromium、Electron、VS Code、Slack、Cursor 等多进程架构程序）。
  - **运行中进程拾取**：内置活动进程列表选择器，带进程图标与路径显示，一键添加。
- **按域名智能分流 (Domain Routing)**：
  - 支持 **主域名 / 后缀匹配**（如 `.github.com`）、**精确匹配**、**关键字匹配** 与 **前缀匹配**。

### 3. SOCKS5 / HTTP 代理支持

- 支持标准 SOCKS5 与 HTTP CONNECT 代理协议。
- 支持用户名 / 密码认证。
- 内置 **网络连通性一键测试**，直观显示延迟 (ms) 与链路状态。

### 4. 网络防污染与协议降级防护

- **TLS SNI 域名嗅探**：首包解析 TLS ClientHello SNI，将真实域名交由代理在远端解析，彻底杜绝本地 DNS 污染与 DNS 泄露。
- **UDP 443 / QUIC 自动降级**：阻断命中进程的 UDP 443 请求，迫使浏览器与应用快速平滑回退至可靠的 TCP 代理线路。
- **安全防环路机制**：Marga 自身 PID 及代理节点的直连连接自动排除，杜绝流量回环死锁。

### 5. 优秀的桌面端交互体验

- **自动 UAC 提权**：启动代理时如未具备管理员权限，无需繁琐的退出操作，应用将自动请求管理员权限平滑重启并无缝启动代理。
- **托盘常驻与最小化**：支持托盘快速开关代理、窗口呼出/隐藏、最小化到系统托盘。
- **开机自启**：支持开机静默后台启动。
- **实时日志观察流**：实时滚动展示命中进程、目标 IP / 域名、连接状态与协议类型。

---

## 🏗️ 技术架构

```
┌────────────────────────────────────────────────────────┐
│                   Marga 桌面客户端                      │
│               Vue 3 + TypeScript + Tailwind CSS         │
└───────────────────────────┬────────────────────────────┘
                            │  Tauri v2 IPC (invoke / event)
┌───────────────────────────▼────────────────────────────┐
│                      Rust 后端核心                      │
│  ├── 配置管理 (AppConfig) ──── 本地 JSON 持久化存储      │
│  ├── 进程管理 (ProcessManager) ─ 枚举 / 进程树 / 规则判定 │
│  ├── 域名规则匹配 (DomainRules) ─ 后缀 / 关键字 / 完整匹配│
│  ├── 包过滤引擎 (WinDivert) ── 双向 NAT / 丢弃 / 环路排除 │
│  ├── 本地中继 (RelayServer) ── Tokio 异步 TCP 连接接入   │
│  ├── 域名嗅探 (TLS SNI Parser) 首包嗅探避免本地 DNS 污染 │
│  └── 代理客户端 (ProxyClient) SOCKS5 / HTTP 协议中继 │
└────────────────────────────────────────────────────────┘
```

---

## 🚀 快速上手与构建

### 1. 环境依赖

- **操作系统**：Windows 10 (21H2+) / Windows 11 (x64)
- **Rust**：`>= 1.70` ([官方安装教程](https://rustup.rs/))
- **Node.js**：`>= 18` 与 [pnpm](https://pnpm.io/)
- **驱动说明**：已内置随应用打包的官方签名版 `WinDivert.dll` 与 `WinDivert64.sys`。

### 2. 安装依赖

```powershell
pnpm install
```

### 3. 本地开发与调试

> **提示**：WinDivert 网络驱动交互需要管理员权限。建议使用管理员身份打开 PowerShell / Windows Terminal。

```powershell
pnpm tauri dev
```

### 4. 生产打包发布

```powershell
pnpm tauri build
```

构建生成的独立安装包与可执行文件位于 `src-tauri/target/release/bundle/`。

---

## 📄 开源许可证

本项目基于 [MIT License](LICENSE) 开源发布。
核心网络驱动组件基于 [WinDivert](https://reqrypt.org/windivert.html)（以动态链接库形式加载，遵循 LGPLv3/GPLv2 许可）。
