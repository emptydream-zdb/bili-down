use anyhow::{Result, anyhow};
use bilidown::{run_download, set_cookie};
use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, long_about = None)]
struct Args {
    /// 待下载视频的 URL
    #[arg(short, long)]
    url: Option<String>,

    /// 保存下载视频的目录路径
    #[arg(short, long, default_value_t = String::from("./"))]
    path: String,

    /// 设置 Cookie
    #[arg(short, long, default_value_t = false)]
    cookie: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse(); // 解析命令行参数

    if args.cookie {
        set_cookie()?;
        println!("Cookie 设置完成！");
    } else {
        let url = args.url.ok_or_else(|| anyhow!("必须提供视频 URL"))?;
        let path = args.path;
        run_download(url, path).await?;
    }
    Ok(())
}
