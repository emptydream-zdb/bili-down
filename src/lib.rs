use anyhow::{Result, anyhow};
use dirs;
use regex::Regex;
use reqwest::{Client, Response};
use std::io::{self, Write};
use std::{fs, path::Path};
use tempfile::NamedTempFile;
use tokio::process::Command;

pub fn set_cookie() -> Result<()> {
    let config_path = if let Some(home) = dirs::home_dir() {
        home.join(".config").join("bilidown").join("cookie.env")
    } else {
        return Err(anyhow!("无法获取用户家目录"));
    };
    fs::create_dir_all(config_path.parent().unwrap())?;
    print!("请输入 BiliBili Cookie 信息: ");
    io::stdout().flush()?;
    let mut cookie = String::new();
    io::stdin().read_line(&mut cookie)?;
    fs::write(&config_path, cookie.trim()).unwrap();
    println!("Cookie 信息已保存到: {}", config_path.display());
    Ok(())
}

fn get_cookie() -> Result<String> {
    let config_path = if let Some(home) = dirs::home_dir() {
        home.join(".config").join("bilidown").join("cookie.env")
    } else {
        return Err(anyhow!("无法获取用户家目录"));
    };

    match fs::read_to_string(&config_path) {
        Ok(cookies) => Ok(cookies.trim().to_string()),
        Err(_err) => {
            println!("Cookie 文件不存在或无法读取，不使用 Cookie。");
            Ok(String::from(""))
        }
    }
}

async fn get_bili(client: &Client, url: &str) -> Result<Response> {
    let cookies: String = get_cookie()?;
    let response = client
        .get(url)
        .header(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
        )
        .header("Accept-Encoding", "gzip")
        .header("Cookie", cookies)
        .header("Referer", "https://www.bilibili.com/")
        .send()
        .await?;
    if response.status().is_success() {
        Ok(response)
    } else {
        Err(anyhow!("请求失败，状态码: {}", response.status()))
    }
}
/// 使用正则表达式匹配文本
/// text: 要匹配的文本
/// regex: 正则表达式字串，必须是原始字符串
fn regex_match(text: &str, regex: &str) -> Result<String> {
    let re = Regex::new(regex)?;
    if let Some(captures) = re.captures(text) {
        // dbg!(captures.get(0).unwrap().as_str());
        if let Some(url) = captures.get(1) {
            return Ok(url.as_str().to_string());
        }
    }
    Err(anyhow!("未匹配成功！"))
}

// <title data-vue-meta="true">不如 元气涂涂_哔哩哔哩_bilibili</title>

fn get_video_name(text: &str) -> Result<String> {
    let video_name = regex_match(
        text,
        r#"<title data-vue-meta="true">([^<]+)_哔哩哔哩_bilibili</title>"#,
    )?;
    Ok(video_name)
}

fn get_video_url(text: &str) -> Result<String> {
    // 匹配 "video" 数组中的第一个 baseUrl
    let video_url = regex_match(text, r#""video":\s*\[\s*\{[^}]*"baseUrl":"([^"]+)""#)?;

    Ok(video_url)
}

fn get_audio_url(text: &str) -> Result<String> {
    // 匹配 "audio" 数组中的第一个 url
    let audio_url = regex_match(text, r#""audio":\s*\[\s*\{[^}]*"baseUrl":"([^"]+)""#)?;

    Ok(audio_url)
}

async fn download_file(url: &str, path: &Path, client: &Client) -> Result<()> {
    let response = get_bili(client, url).await?;
    let bytes = response.bytes().await?;
    fs::write(path, &bytes)?;
    Ok(())
}

async fn merge_video_audio_async(
    video_path: &Path,
    audio_path: &Path,
    output_path: &Path,
) -> Result<()> {
    let output = Command::new("ffmpeg")
        .arg("-i")
        .arg(video_path.to_str().unwrap())
        .arg("-i")
        .arg(audio_path.to_str().unwrap())
        .arg("-c:v")
        .arg("copy")
        .arg("-c:a")
        .arg("copy")
        .arg(output_path.to_str().unwrap())
        .output()
        .await?;

    if output.status.success() {
        Ok(())
    } else {
        let error = String::from_utf8_lossy(&output.stderr);
        Err(anyhow!("FFmpeg合并失败: {}", error))
    }
}

async fn complete_download(
    video_url: &str,
    audio_url: &str,
    output_path: &Path,
    client: &Client,
) -> Result<()> {
    let video_temp = NamedTempFile::new()?;
    let audio_temp = NamedTempFile::new()?;

    let video_temp_path = video_temp.path();
    let audio_temp_path = audio_temp.path();

    download_file(video_url, video_temp_path, &client).await?;
    download_file(audio_url, audio_temp_path, &client).await?;

    // 使用 FFmpeg 合并视频和音频
    merge_video_audio_async(video_temp_path, audio_temp_path, output_path).await
}

fn check_path(path: &str) -> Result<()> {
    let path = Path::new(path);
    if !path.exists() {
        return Err(anyhow!("指定的路径不存在: {}", path.display()));
    }
    if !path.is_dir() {
        return Err(anyhow!("指定的路径不是目录: {}", path.display()));
    }
    Ok(())
}

pub async fn run_download(url: String, path: String) -> Result<()> {
    println!("开始下载视频: {}", url);
    check_path(&path)?;
    // 创建 HTTP 客户端
    let client = reqwest::Client::new();

    let response = get_bili(&client, &url).await?;
    if !response.status().is_success() {
        return Err(anyhow!("请求失败，状态码: {}", response.status()));
    }

    let result = response.text().await?;

    let video_name = get_video_name(&result)?;
    println!("视频名称: {}", video_name);
    let out_path = Path::new(&path).join(format!("{}.mp4", video_name));
    if out_path.exists() {
        return Err(anyhow!("文件已存在: {}", out_path.display()));
    }

    let video_url = get_video_url(&result)?;
    let audio_url = get_audio_url(&result)?;

    complete_download(&video_url, &audio_url, &out_path, &client).await?;
    println!("视频下载完成: {}", out_path.display());

    Ok(())
}
