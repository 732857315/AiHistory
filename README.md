# AiHistory · 从工具到智能

从石器、自动机关与写字人偶，到编程、神经网络和模型星河。一个使用 **Rust / WebAssembly** 实时演算画面与音乐的多语言历史网站。

- 网站：**https://risc.ink/AiHistory/**
- GitHub Pages 入口：https://732857315.github.io/AiHistory/ （继承账号自定义域名）
- 仓库：https://github.com/732857315/AiHistory
- 许可证：[MIT](LICENSE)，© 2026 732857315

## 体验

首页直接播放连续旅程，不要求选择章节。16 个历史时刻共 154 秒，其后 180 秒进入模型星河，最后停留 24 秒查看最新收录的比较快照，总长约 6 分钟。早期神经网络、CNN 与 Transformer 获得更长解释时间。纸张与剪影逐步过渡为电路、网络和星河。

打开即进入大画面影院。历史部分使用分层纸张、蚀刻纹理、机械插画与会运动的原理图；星河部分使用程序生成的旋臂、星尘、轨迹与模型彗星。手机的星河单独排版，避免把桌面标签直接缩小。

支持中文、英语、日语、西班牙语；提供时间线跳转、进度拖动、暂停、倍速、全屏、大字与减弱动态效果。解释与模型档案默认折叠，展开会暂停，以便查看能力、局限和第一手来源。模型档案支持搜索和跳转发布日期；时间线随当前时刻横向定位。

音乐默认开启，由原创乐谱实时合成；浏览器阻止自动发声时，点击“开启声音”。可选朗读使用设备的 Speech Synthesis 声音；声音与语言可用性取决于浏览器/操作系统，部分声音可能由浏览器在线提供。项目不集成第三方 TTS 接口或下载语音文件。不支持的语言仍提供完整文字。页面进入后台时暂停。

键盘：空格播放/暂停，长按只切换一次；左右方向键移动 10 秒（文本框、选择框、按钮和折叠标题等控件保留原生键盘操作）。用户开启减少动态效果偏好时，初次访问暂停，手动播放后保留静态背景与关键状态变化。页面也响应系统偏好的即时更改，开启时暂停，关闭时等待手动播放；页面上的动态效果按钮可覆盖本次访问的系统设置。

## 数据与比较边界

收录 20 个精选模型，发布资料核验截至 **2026-10-24**。12 个历史成绩来自截至 **2025-01-22** 的公开报告，分为两个提示协议的 MMLU 赛道。另有 **2026-10-24 查阅的 3 个 AdvancedIF 成绩快照**，采用独立赛道，观察日期不冒充成绩发表日期。这不是当前全球总榜，也不是本站独立、统一算力条件下的重测。

发布时间与成绩公布时间分别记录。只有到达成绩来源日期，模型才进入匹配指标和协议的排名；没有可比成绩的模型仍会登场，但不会被记为零分。切换赛道重建比较，不拼接异类分数。历史图形均为原理示意，不冒充实物复原。

完整模型版本、日期、成绩、协议差异及来源见 [模型数据说明](sources/models.md)。历史节点的第一手来源位于 [history.rs](src/history.rs)。

## 实现

| 文件 | 用途 |
| --- | --- |
| `src/app.rs` | Rust DOM、播放、交互、多语言、朗读与状态管理 |
| `src/lib.rs` | 时间映射、评测日期门控、排名与单元测试 |
| `src/scene.rs` | 1200×675 历史画布：蚀刻插画、自动机关、网络与注意力 |
| `src/cosmos.rs` | 程序星尘、轨道追逐、成绩间距、避让标签与手机构图 |
| `src/audio.rs` | Web Audio 振荡器与包络，原创音乐调度与释放 |
| `src/history.rs` / `src/models.rs` | 四语历史与模型数据 |
| `assets/music-score.json` | 原创音符级乐谱，见 [音乐说明](assets/MUSIC.md) |
| `web/` | HTML、CSS、矢量图标；必要的 WASM 初始化胶水 |
| `scripts/` | Python 构建、Windows 环境适配与浏览器验证 |

所有应用行为使用 Rust/WASM；不使用 Node/npm、媒体播放器、预录音频、视频、图片序列或商业音频采样。发布时只需静态托管，无服务器密钥或在线模型推理服务。

## 构建与验证

需要 Rust stable、Python 3.11+。Windows 需要安装 Visual Studio C++ Build Tools 与 Windows SDK；Python 构建器会尝试发现已有工具。

```text
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.129 --locked
python scripts/build.py
python -m http.server 8080 --bind 127.0.0.1 --directory dist
```

访问 http://127.0.0.1:8080/ 。不要直接以 `file://` 打开，WASM 需要通过 HTTP 加载。`dist/` 和 `target/` 不提交。

```text
python scripts/rust_command.py fmt --check
python scripts/rust_command.py test --locked
python scripts/rust_command.py clippy --locked --all-targets -- -D warnings
python scripts/rust_command.py clippy --locked --target wasm32-unknown-unknown -- -D warnings
python -m pip install --target .local/python -r scripts/requirements-test.txt
python -X utf8 scripts/browser_check.py
```

浏览器验证使用已有 Chrome/Edge，在 `/AiHistory/` 子路径检查实际动画、声音解锁与真实合成波形、暂停时画布像素冻结、语言切换、时间跳转、赛道隔离、折叠档案和两种手机宽度，截图保存在忽略的 `.local/screenshots/`。没有可用浏览器时可以通过 `--browser` 指定路径。

## 发布

仓库 Settings → Pages → Source 选择 **GitHub Actions**。推送 `main` 后，工作流执行格式、测试、原生与 WASM Clippy 检查，构建并部署 `dist/`。项目继承 `risc.ink`，**不要在此项目添加根域名 CNAME，也不要改动账号主页的域名配置**。

提交身份：`732857315 <732857315@qq.com>`。项目代码、原创图形、文字和乐谱使用 MIT；链接到的论文、商标及其他第三方来源保留其原有权利。
