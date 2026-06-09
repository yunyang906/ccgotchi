<p align="center">
  <img src="assets/logo.svg" alt="ccgotchi" width="680">
</p>

<p align="center">
  <a href="README.md">English</a> · <a href="README.zh-CN.md">简体中文</a> · <a href="README.ja.md">日本語</a> · <b>한국어</b>
</p>

<p align="center">
  사용량 진행 막대와 애니메이션 ASCII <b>펫</b>이 있는 <a href="https://docs.claude.com/en/docs/claude-code">Claude Code</a> <b>상태줄</b> —— 터미널 속 작은 다마고치.
</p>

여유가 있을 때는 펫이 행복하지만, 할당량을 다 써갈수록 시들시들해집니다. 18종의 펫과 숨겨진 **✨ 샤이니(무지개)** 모드.

<p align="center">
  <img src="assets/demo.svg" alt="ccgotchi 상태줄 데모" width="900">
</p>

Claude Code가 상태줄 명령에 전달하는 JSON을 읽어 한 줄(또는 여러 줄)을 출력합니다. 데몬 없음, 텔레메트리 없음 —— 작은 바이너리 하나뿐입니다.

## 기능

- **5시간** 및 **7일(주간)** 사용 한도 창(Pro/Max)에 대한 **사용량 막대**와 **리셋 카운트다운**.
- **컨텍스트 창** 막대 (API 모드에서도 작동 —— API 모드에는 5h/주간 창이 없으므로 해당 세그먼트는 자동으로 생략됩니다).
- **오른쪽에 고정된 애니메이션 펫**. 표정/체력 = `100 − 가장 많이 사용한 창` 이므로 작업에 따라 눈에 띄게 반응합니다. 눈을 깜빡이고 입을 움직이며 매초 다시 애니메이션됩니다.
- **18종**(Claude Buddy 라인업): cat, chonk, rabbit, duck, goose, owl, penguin, turtle, snail, dragon, octopus, axolotl, ghost, robot, blob, cactus, mushroom, capybara.
- **✨ 샤이니 모드** —— 문자마다 흐르는 무지개색(트루컬러), 숨겨진 변형.
- **펫 색상 선택** —— 자동(체력에 따라) 또는 고정 색상(주황, 분홍, 파랑…).
- **세그먼트별 표시/숨기기** —— 5h / 7d / 컨텍스트를 각각 독립적으로 전환.
- **막대 스타일 변경 가능**: 점 / 블록 / 음영 / 사각형 / 빗금 / 배터리, 컬러 또는 단색.

## 설치

### macOS 앱 (메뉴 막대 트레이) —— 권장

앱을 빌드한 후 메뉴 막대 아이콘을 클릭하여 모든 것을 설정하세요. 외울 명령이 없습니다:

```bash
git clone https://github.com/yunyang906/ccgotchi
cd ccgotchi
cargo build --release --workspace
./package_macos.sh
open build/ccgotchi.app
```

실행하면 상태줄이 자동으로 Claude Code에 연결되고 🐈 메뉴 막대 아이콘이 추가됩니다. 메뉴에서: 펫 선택, ✨ 샤이니 전환, 막대 스타일 / 색상 / 언어 변경 —— 변경 사항은 즉시 적용됩니다. "Restore(제거)"로 되돌릴 수 있습니다.

### CLI (모든 플랫폼)

```bash
cargo install --git https://github.com/yunyang906/ccgotchi ccgotchi
ccgotchi setup       # 상태줄을 ~/.claude/settings.json에 연결
ccgotchi restore     # 되돌리기 (이전 statusLine 복원)
```

`setup`은 다음을 작성합니다(기존 statusLine은 백업됨):

```json
{ "statusLine": { "type": "command", "command": "ccgotchi statusline", "refreshInterval": 1 } }
```

`refreshInterval: 1`은 유휴 상태에서도 펫이 계속 애니메이션되도록 합니다. 새 Claude Code 세션을 열면(또는 1초 기다리면) 표시됩니다.

> 사전 빌드된 바이너리는 [Releases](https://github.com/yunyang906/ccgotchi/releases) 페이지에 있습니다: Windows / macOS(arm64, x86) / Linux용 CLI 및 macOS 트레이 앱(arm64, x86).

## 설정

**메뉴 막대 앱**으로 클릭하거나 CLI에서 무엇이든 설정하세요:

```bash
ccgotchi pet cat            # cat|chonk|rabbit|...|capybara|off (18종)
ccgotchi petcolor auto      # auto(체력에 따라) | orange|pink|red|yellow|green|cyan|blue|purple|white|gray
ccgotchi shiny on           # 무지개 펫 (on|off)
ccgotchi barstyle dots      # dots|block|shade|square|slant|battery
ccgotchi barcolor auto      # auto(사용량에 따라 녹/황/적) | mono(단색)
ccgotchi resetfmt eta       # eta | arrow (↻) | paren | cn (余) | off
ccgotchi show ctx off       # 세그먼트 숨기기/표시: 5h|7d|ctx (on|off)
ccgotchi lang ko            # en | zh | ja | ko ($LANG에서 자동 감지)
ccgotchi config             # 현재 설정 출력
```

### 국제화(i18n)

세그먼트 라벨은 현지화됩니다. 언어는 `$LANG` / `$LC_ALL`에서 자동 감지됩니다(감지 실패 시 영어로 폴백). `ccgotchi lang <code>`로 명시적으로 지정할 수도 있습니다:

| 언어 | 5h 창 | 7일 창 | 컨텍스트 |
|------|-----------|--------------|---------|
| `en` | `5h`      | `7d`         | `ctx`   |
| `zh` | `5h`      | `周`         | `上下文` |
| `ja` | `5h`      | `週`         | `文脈`   |
| `ko` | `5h`      | `주`         | `컨텍스트` |

언어 추가는 한 줄짜리 PR로 끝납니다: `src/lib.rs`의 `labels()`에 match 암을 추가하면 됩니다.

## 펫 체력

`체력 = 100 − max(5h%, 주간%, 컨텍스트%)`:

| 체력 | 기분 | 예시 |
|--------|------|---------|
| ≥ 60   | 행복 | `( ^.^ )` |
| 30–60  | 보통 | `( o.o )` |
| 10–30  | 아픔 | `( T.T )` |
| < 10   | 빈사 | `( x.x )` |

## 참고 / 작동 방식

- 터미널 너비는 `COLUMNS` 환경 변수에서 가져옵니다. Claude Code가 명령 실행 전에 이를 설정합니다(v2.1.153+). 펫의 오른쪽 정렬이 이를 이용합니다.
- Claude Code는 상태줄 출력의 각 줄 앞 공백을 제거하므로([#29206](https://github.com/anthropics/claude-code/issues/29206)), 오른쪽 정렬 패딩에는 점자 공백 `U+2800`을 사용합니다(일반 공백이 아니며 제거를 견디고 공백으로 렌더링됨).

## 크레딧

[ccstatusline](https://github.com/sirmalloc/ccstatusline), [ccpet](https://github.com/terryso/ccpet) 및 Anthropic의 Claude Buddy에서 영감을 받았습니다. [ClaudeLight](https://github.com/yunyang906/ClaudeLight) 프로젝트(하드웨어 신호등 클라이언트)에서 분리되었습니다.

## Star 기록

<a href="https://star-history.com/#yunyang906/ccgotchi&Date">
  <img src="https://api.star-history.com/svg?repos=yunyang906/ccgotchi&type=Date" alt="Star History Chart" width="100%">
</a>

## 라이선스

MIT —— [LICENSE](LICENSE) 참조.
