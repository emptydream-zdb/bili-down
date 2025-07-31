# BiliDown - Bilibili 视频下载工具

一个用 Rust 编写的 Bilibili 视频下载工具，支持视频url链接解析下载。

## 特性

- 🚀 **高性能**: 使用 Rust 异步编程，下载速度快
- 🎵 **完整下载**: 自动下载视频和音频并合并为完整的 MP4 文件
- 🍪 **Cookie 支持**: 支持登录状态下载更高质量的视频
- 📁 **灵活路径**: 可自定义下载保存目录
- ⚡ **命令行工具**: 简单易用的命令行界面

## 系统要求

- 操作系统: Windows, macOS, Linux
- [FFmpeg](https://ffmpeg.org/download.html) (用于视频音频合并)

### 安装 FFmpeg

#### Windows
1. 从 [FFmpeg 官网](https://ffmpeg.org/download.html) 下载二进制文件
2. 将 FFmpeg 添加到系统 PATH 环境变量中

#### macOS
```bash
brew install ffmpeg
```

#### Linux
-  Ubuntu/Debian
```bash
sudo apt update
sudo apt install ffmpeg
```
- Arch
```bash
sudo pacman -Sy ffmpeg
```
- 其他发行版根据相应的包管理器命令下载。

## 安装

### 从源码编译

1. 确保已安装 [Rust](https://rustlang.org/)
2. 克隆项目并编译：

```bash
git clone https://github.com/emptydream-zdb/bili-down.git
cd bili-down
cargo build --release
```

编译后的可执行文件位于 `target/release/bilidown.exe` (Windows) 或 `target/release/bilidown` (Unix)

## 使用方法

Usage: bilidown [OPTIONS]

Options:
  -u, --url <URL>    待下载视频的 URL
  -p, --path <PATH>  视频下载的保存路径 [default: ./]
  -c, --cookie       设置 Cookie
  -h, --help         Print help
  -V, --version      Print version

### 基本用法

下载视频到当前目录：
```bash
bilidown -u "https://www.bilibili.com/video/BV1Bp4y1V79u"
```

下载视频到指定目录：
```bash
bilidown -u "https://www.bilibili.com/video/BV1Bp4y1V79u" -p "/home/user/video/"
```

### Cookie 设置

对于需要登录才能观看的视频或获取更高质量的视频，需要先设置 Cookie：

```bash
bilidown -c
```

系统会提示输入 Cookie 信息。Cookie 信息获取方法：

1. 在浏览器中登录 Bilibili
2. 按 F12 打开开发者工具
3. 切换到 Network (网络) 标签页
4. 刷新页面或访问任意 Bilibili 页面
5. 找到对 bilibili.com 的请求
6. 在 Request Headers 中找到 Cookie 字段，复制其值
7. 粘贴到程序提示中

Cookie 信息会保存在用户配置目录中，过期前不再需要重新设置：
- Windows: `%USERPROFILE%\.config\bilidown\cookie.env`
- macOS/Linux: `~/.config/bilidown/cookie.env`

Cookie具有有效期，定期需要重新设置。


## 工作原理

1. **解析视频页面**: 访问 Bilibili 视频页面并提取视频信息
2. **获取媒体链接**: 从页面中提取视频和音频的下载链接
3. **分别下载**: 并行下载视频流和音频流到临时文件
4. **合并媒体**: 使用 FFmpeg 将视频和音频合并为完整的 MP4 文件
5. **清理临时文件**: 自动删除临时下载文件

## 技术栈

- **语言**: Rust (2024 Edition)
- **HTTP 客户端**: reqwest
- **异步运行时**: Tokio
- **命令行解析**: clap
- **正则表达式**: regex
- **视频处理**: FFmpeg

## 注意事项

⚠️ **重要提醒**：
- 请遵守 Bilibili 的使用条款和版权规定
- 仅供个人学习和研究使用
- 不要用于商业用途或侵犯版权
- 建议适度使用，避免对服务器造成过大压力

## 常见问题

### Q: 下载失败怎么办？
A: 请检查：
1. 网络连接是否正常
2. 视频 URL 是否正确
3. 是否需要登录观看（设置 Cookie）
4. FFmpeg 是否正确安装

### Q: 下载的视频质量不高？
A: 设置 Cookie 后可以获取更高质量的视频源。

### Q: 支持批量下载吗？
A: 当前版本不支持批量下载，后续版本会考虑添加此功能。

## 开发计划

- [ ] 支持批量下载
- [ ] 添加下载进度显示
- [ ] 支持选择视频质量
- [ ] 支持下载字幕
- [ ] 图形界面版本

## 贡献

欢迎提交 Issue 和 Pull Request！

## 许可证

本项目采用 GNU 3.0 许可证 - 详见 [LICENSE](LICENSE) 文件

## 作者

emptydream-zdb

---

**免责声明**: 本工具仅供学习交流使用，请遵守相关法律法规和平台规定。