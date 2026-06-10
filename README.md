# ferio-linux-helper

**一个基于 TUI 的 Linux 桌面配置与故障排查助手**

使用 Rust + ratatui + crossterm 编写，面向 Linux 桌面用户。提供系统信息查看、软件包管理、systemd 服务管理、网络诊断、桌面修复、日志查看和系统优化等功能。

## 功能模块

| 模块 | 功能描述 |
|------|---------|
| **📊 系统信息** | 显示发行版、内核版本、桌面环境、CPU、内存、磁盘、显卡、会话类型(Wayland/X11)、systemd 状态等 |
| **📦 软件包管理** | 自动识别包管理器 (pacman/dnf/apt/zypper)，提供刷新源、检查更新、清理缓存等操作 |
| **⚙ systemd 管理** | 搜索服务、查看状态、启动/停止/重启/启用/禁用服务 |
| **🌐 网络检查** | 显示 IP/DNS/网关/NetworkManager 状态，支持 Ping 诊断 |
| **🔧 桌面修复** | KDE/GNOME Shell 重启、PipeWire 重启、缩略图/图标缓存清理等 |
| **📋 日志查看** | journalctl 错误日志、启动日志、内核日志、指定服务日志，支持滚动分页 |
| **🚀 一键优化** | Swappiness 调整、日志清理、SSD TRIM、inotify 优化等（附风险提示） |

## 安全机制

- **命令确认**：所有系统修改操作执行前必须在弹窗中展示完整命令，由用户确认后才会执行
- **Dry-Run 模式**：通过 `--dry-run` 或环境变量 `MOCK_EXEC=1` 启动，命令仅记录不执行
- **风险提示**：每个操作均附有风险等级和说明

## 编译与运行

### 依赖

- Rust 工具链 (rustc + cargo)
- Linux 系统 (支持 systemd)

### 快捷开发与构建 (Makefile)

项目根目录下提供了 `Makefile`，提供了极其便利的开发快捷指令：

```bash
make dev      # 以开发者模式运行（免 root 且开启 dry-run，最适合快速迭代界面）
make check    # 快速检查代码语法和类型（不生成二进制，省时省力）
make run      # 以正常特权运行（自动拉起 sudo 提权）
make build    # 编译 Debug 版本的二进制文件
make test     # 执行单元测试
make watch    # 开启“保存自动重构运行”循环（需要先执行 cargo install cargo-watch）
make clean    # 清理构建缓存
```

### 手动编译与运行

```bash
# 编译 Release 版本
cargo build --release

# 正常运行（需要 root 权限，会自动通过 sudo 提权）
./target/release/ferio-linux-helper

# Dry-Run 模式（命令仅记录，不实际执行）
./target/release/ferio-linux-helper --dry-run

# 跳过 Root 检查（仅查看信息，无法执行系统命令）
./target/release/ferio-linux-helper --no-root
```

## 快捷键

| 按键 | 功能 |
|------|------|
| `1`-`7` | 切换到对应模块 |
| `←` `→` / `Tab` | 切换模块标签 |
| `↑` `↓` / `j` `k` | 上下选择 |
| `Enter` | 确认/执行操作 |
| `Esc` | 返回/取消 |
| `q` | 退出程序 |
| `Ctrl+C` | 强制退出 |
| `r` / `R` | 刷新当前模块数据 |
| `/` | 搜索 (systemd 模块) |
| `PgUp`/`PgDn` | 翻页 (日志模块) |

## 项目结构

```
ferio-linux-helper/
├── Cargo.toml
├── README.md
├── src/
│   ├── main.rs              # 入口：参数解析、权限检查、TUI 事件循环
│   ├── privilege.rs          # Root 权限检查与自动 sudo 提权
│   ├── commands.rs           # 统一命令执行器（含 dry-run 支持）
│   ├── tui/
│   │   ├── mod.rs            # 终端管理 (raw mode / alternate screen / panic hook)
│   │   ├── app.rs            # 主应用状态、Tab 导航、确认弹窗
│   │   └── widgets.rs        # 可复用 TUI 组件 (居中弹窗、可选列表)
│   └── modules/
│       ├── mod.rs            # TuiModule trait 定义
│       ├── system_info.rs    # 系统信息模块
│       ├── package_manager.rs # 软件包管理模块
│       ├── systemd.rs        # systemd 服务管理模块
│       ├── network.rs        # 网络检查模块
│       ├── desktop_fixes.rs  # 桌面环境修复模块
│       ├── log_viewer.rs     # 日志查看模块
│       └── optimizations.rs  # 一键优化模块
```

## 设计原则

- **模块化**：每个功能模块独立实现 `TuiModule` trait，方便新增模块
- **安全优先**：所有操作需用户确认，支持 dry-run 测试
- **中文友好**：界面全中文标签和提示
- **轻量依赖**：核心仅依赖 ratatui/crossterm/sysinfo/clap/nix

## License

MIT
