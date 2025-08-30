use anyhow::{Result, anyhow};
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use regex::Regex;
use reqwest::{Client, Response};
use std::io::{self, Write};
use std::path::PathBuf;
use std::{fs, path::Path};
use tempfile::NamedTempFile;
use tokio::process::Command;

fn get_config_path() -> Result<PathBuf> {
    if let Some(home) = dirs::home_dir() {
        Ok(home.join(".config").join("bilidown").join("cookie.env"))
    } else {
        Err(anyhow!("无法获取用户家目录"))
    }
}

pub fn set_cookie() -> Result<()> {
    let config_path = get_config_path()?;
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    } else {
        return Err(anyhow!("无法获取配置文件的父目录"));
    }
    print!("请输入 BiliBili Cookie 信息: ");
    io::stdout().flush()?;
    let mut cookie = String::new();
    io::stdin().read_line(&mut cookie)?;
    fs::write(&config_path, cookie.trim())?;
    println!("Cookie 信息已保存到: {}", config_path.display());
    Ok(())
}

fn get_cookie() -> Result<String> {
    let config_path = get_config_path()?;
    match fs::read_to_string(&config_path) {
        Ok(cookies) => Ok(cookies.trim().to_string()),
        Err(err) => {
            println!("Cookie 文件不存在或无法读取，不使用 Cookie ,错误：{err}");
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
    let captures = re
        .captures(text)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
        .ok_or_else(|| anyhow!("未匹配成功!"))?;
    Ok(captures)
}

fn get_video_name(text: &str) -> Result<String> {
    let video_name = regex_match(text, r#"<title.*?>([^<]+)_哔哩哔哩_bilibili</title>"#)?;
    Ok(video_name)
}

fn get_video_baseurl(text: &str) -> Result<String> {
    // 匹配 "video" 数组中的第一个 baseUrl
    let video_baseurl = regex_match(text, r#""video":\s*\[\s*\{[^}]*"baseUrl":"([^"]+)""#)?;

    Ok(video_baseurl)
}

fn get_audio_baseurl(text: &str) -> Result<String> {
    // 匹配 "audio" 数组中的第一个 url
    let audio_baseurl = regex_match(text, r#""audio":\s*\[\s*\{[^}]*"baseUrl":"([^"]+)""#)?;

    Ok(audio_baseurl)
}

async fn download_file(url: &str, path: &Path, client: &Client, pb: &ProgressBar) -> Result<()> {
    let response = get_bili(client, url).await?;
    let total_size = response.content_length().unwrap_or(0);

    pb.set_length(total_size);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
        .unwrap()
        .progress_chars("#>-"));

    let mut stream = response.bytes_stream();
    let mut downloaded = 0u64;
    let mut file_content = Vec::new();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file_content.extend_from_slice(&chunk);
        downloaded += chunk.len() as u64;
        pb.set_position(downloaded);
    }

    fs::write(path, &file_content)?;
    pb.finish_with_message("下载完成");
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
    video_baseurl: &str,
    audio_baseurl: &str,
    output_path: &Path,
    client: &Client,
) -> Result<()> {
    let video_temp = NamedTempFile::new()?;
    let audio_temp = NamedTempFile::new()?;

    let video_temp_path = video_temp.path();
    let audio_temp_path = audio_temp.path();

    println!("下载视频文件...");
    let video_pb = ProgressBar::new(0);
    download_file(video_baseurl, video_temp_path, client, &video_pb).await?;

    println!("下载音频文件...");
    let audio_pb = ProgressBar::new(0);
    download_file(audio_baseurl, audio_temp_path, client, &audio_pb).await?;

    println!("合并视频和音频...");
    let merge_pb = ProgressBar::new_spinner();
    merge_pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    merge_pb.set_message("正在合并文件...");
    merge_pb.enable_steady_tick(std::time::Duration::from_millis(100));

    let result = merge_video_audio_async(video_temp_path, audio_temp_path, output_path).await;
    merge_pb.finish_with_message("合并完成");

    result
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

fn complete_url(url: String) -> String {
    if url.starts_with("https://www.bilibili.com/video/") {
        url
    } else {
        format!("https://www.bilibili.com/video/{url}")
    }
}

fn generate_unique_filename(base_path: &Path, video_name: &str) -> PathBuf {
    let mut cnt = 0;
    loop {
        let filename = if cnt == 0 {
            format!("{video_name}.mp4")
        } else {
            format!("{video_name}-{cnt}.mp4")
        };
        let path = base_path.join(filename);
        if !path.exists() {
            return path;
        }
        cnt += 1;
    }
}

pub async fn run_download(url: String, path: String) -> Result<()> {
    let url = complete_url(url);

    println!("开始下载视频: {}", &url);
    check_path(&path)?;
    let client = reqwest::Client::new();

    let response = get_bili(&client, &url).await?;
    let result = response.text().await?;

    let video_name = get_video_name(&result)?;
    println!("视频名称: {}", &video_name);

    let out_path = generate_unique_filename(&PathBuf::from(&path), &video_name);
    let video_baseurl = get_video_baseurl(&result)?;
    let audio_baseurl = get_audio_baseurl(&result)?;

    complete_download(&video_baseurl, &audio_baseurl, &out_path, &client).await?;
    println!("视频下载完成: {}", out_path.display());

    Ok(())
}
