<p align="center">
  <img src="assets/logo.svg" alt="ccgotchi" width="680">
</p>

<p align="center">
  <a href="README.md">English</a> · <b>简体中文</b> · <a href="README.ja.md">日本語</a> · <a href="README.ko.md">한국어</a>
</p>

<p align="center">
  一个带用量进度条和动画 ASCII <b>萌宠</b>的 <a href="https://docs.claude.com/en/docs/claude-code">Claude Code</a> <b>状态栏</b> —— 你终端里的电子宠物。
</p>

额度还充裕时萌宠开心，额度快烧光时它就蔫了。18 种宠物，外加隐藏的 **✨ 七彩（shiny）** 模式。

<p align="center">
  <img src="assets/demo.svg" alt="ccgotchi 状态栏演示" width="900">
</p>

它读取 Claude Code 传给状态栏命令的 JSON，输出一行（或多行）。无后台进程、无遥测 —— 就一个小小的二进制。

## 功能

- **用量进度条**：**5 小时**和 **7 天（周）**额度窗口（Pro/Max），并显示**重置倒计时**。
- **上下文窗口**进度条（API 模式同样可用 —— API 模式没有 5h/周窗口，那两段会自动省略）。
- **右侧常驻动画萌宠**：表情/健康 = `100 − 你用得最多的那个窗口`，随你的使用实时变化。会眨眼、动嘴，每秒重绘一次。
- **18 种宠物**（Claude Buddy 全家桶）：cat、chonk、rabbit、duck、goose、owl、penguin、turtle、snail、dragon、octopus、axolotl、ghost、robot、blob、cactus、mushroom、capybara。
- **✨ 七彩模式** —— 逐字符流动的彩虹（真彩色），隐藏款。
- **自定义宠物颜色** —— 自动（随健康变色）或固定颜色（橙、粉、蓝……）。
- **每段可显示/隐藏** —— 5h / 7d / 上下文 各自独立开关。
- **进度条样式可换**：点状 / 实心块 / 阴影 / 方块 / 斜线 / 电池，彩色或单色皆可。

## 安装

### macOS 应用（菜单栏托盘）—— 推荐

构建应用后，点菜单栏图标即可完成全部配置，不用记任何命令：

```bash
git clone https://github.com/yunyang906/ccgotchi
cd ccgotchi
cargo build --release --workspace
./package_macos.sh
open build/ccgotchi.app
```

启动后会自动把状态栏接入 Claude Code，并在菜单栏加一个 🐈 图标。从菜单里：选宠物、开关 ✨ 七彩、改进度条样式 / 颜色 / 语言 —— 改动实时生效。点「Restore（卸载）」即可还原。

### CLI（任意平台）

```bash
cargo install --git https://github.com/yunyang906/ccgotchi ccgotchi
ccgotchi setup       # 把状态栏写入 ~/.claude/settings.json
ccgotchi restore     # 还原（恢复你之前的 statusLine）
```

`setup` 会写入（并备份已有的 statusLine）：

```json
{ "statusLine": { "type": "command", "command": "ccgotchi statusline", "refreshInterval": 1 } }
```

`refreshInterval: 1` 让宠物在空闲时也能持续动起来。新开一个 Claude Code 会话（或等一秒）即可看到。

> 预编译二进制见 [Releases](https://github.com/yunyang906/ccgotchi/releases)：CLI 覆盖 Windows / macOS（arm64、x86）/ Linux，另带 macOS 托盘应用（arm64、x86）。

## 配置

用**菜单栏应用**点点点，或用 CLI 设置任意项：

```bash
ccgotchi pet cat            # cat|chonk|rabbit|...|capybara|off（18 种）
ccgotchi petcolor auto      # auto（随健康）| orange|pink|red|yellow|green|cyan|blue|purple|white|gray
ccgotchi shiny on           # 七彩宠物（on|off）
ccgotchi barstyle dots      # dots|block|shade|square|slant|battery
ccgotchi barcolor auto      # auto（按用量绿/黄/红）| mono（单色）
ccgotchi resetfmt eta       # eta | arrow (↻) | paren | cn (余) | off
ccgotchi show ctx off       # 隐藏/显示某段：5h|7d|ctx （on|off）
ccgotchi lang zh            # en | zh | ja | ko（默认从 $LANG 自动识别）
ccgotchi config             # 打印当前配置
```

### 国际化

各段标签会本地化。语言默认从 `$LANG` / `$LC_ALL` 自动识别（识别不到则回退英文），也可用 `ccgotchi lang <code>` 显式指定：

| 语言 | 5h 窗口 | 7 天窗口 | 上下文 |
|------|---------|----------|--------|
| `en` | `5h`    | `7d`     | `ctx`   |
| `zh` | `5h`    | `周`     | `上下文` |
| `ja` | `5h`    | `週`     | `文脈`   |
| `ko` | `5h`    | `주`     | `컨텍스트` |

加一种语言只需一行 PR：在 `src/lib.rs` 的 `labels()` 里加一个匹配分支即可。

## 宠物健康

`健康 = 100 − max(5h%, 周%, 上下文%)`：

| 健康 | 心情 | 示例 |
|------|------|------|
| ≥ 60   | 开心 | `( ^.^ )` |
| 30–60  | 一般 | `( o.o )` |
| 10–30  | 难受 | `( T.T )` |
| < 10   | 濒死 | `( x.x )` |

## 说明 / 原理

- 终端宽度来自 `COLUMNS` 环境变量 —— Claude Code 在运行命令前会设置它（v2.1.153+）。宠物靠它右对齐。
- Claude Code 会裁掉状态栏输出每行的行首空白（[#29206](https://github.com/anthropics/claude-code/issues/29206)），所以右对齐的填充用的是盲文空白 `U+2800`（不是普通空格，能扛过裁剪，且渲染为空白）。

## 致谢

灵感来自 [ccstatusline](https://github.com/sirmalloc/ccstatusline)、[ccpet](https://github.com/terryso/ccpet) 以及 Anthropic 的 Claude Buddy。从 [ClaudeLight](https://github.com/yunyang906/ClaudeLight) 项目（硬件红绿灯客户端）中抽离而来。

## Star 趋势

<a href="https://star-history.com/#yunyang906/ccgotchi&Date">
  <img src="https://api.star-history.com/svg?repos=yunyang906/ccgotchi&type=Date" alt="Star History Chart" width="100%">
</a>

## 许可

MIT —— 见 [LICENSE](LICENSE)。
