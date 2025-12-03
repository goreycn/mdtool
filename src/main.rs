use anyhow::{Context, Ok};
use clap::Parser;
use std::process::Command;
use which::which;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(value_name = "url")]
    input: String,
}

fn check_hf_cli() -> anyhow::Result<()> {
    unsafe {
        std::env::set_var("HF_ENDPOINT", "https://hf-mirror.com");
    }

    if which("hf").is_ok() {
        println!("hf 命令已存在，跳过安装");
        return Ok(());
    }
    println!("未检测到 hf 命令,正在自动安装 hf");

    let status = Command::new("sh")
        .arg("-c")
        .arg("curl -LsSf https://hf.co/cli/install.sh | bash")
        .status()?;

    if status.success() {
        println!("hf 安装成功");
        Ok(())
    } else {
        anyhow::bail!("hf 安装失败,退出码: {:?}", status.code());
    }
}

fn check_model_scope_cli() -> anyhow::Result<()> {
    if which("modelscope").is_ok() {
        println!("modelscope 命令已存在,路过安装");
        return Ok(());
    }
    println!("未检测到 modelscope　命令，正在自动安装");

    let status = Command::new("pip3")
        .arg("install")
        .arg("modelscope")
        .status()?;

    if status.success() {
        println!("modelscope 安装成功");
        Ok(())
    } else {
        anyhow::bail!("modelscope 安装失败，退出码: {:?}", status.code());
    }
}

fn handle_hf(url: &str) -> anyhow::Result<()> {
    let prefix = "https://huggingface.co/";
    let suffix = "/resolve/main/";

    if !url.starts_with(prefix) || !url.contains(suffix) {
        anyhow::bail!("不是一个有效的 hf 下载链接")
    }

    let rest = url.strip_prefix(prefix).context("去除前缀失败")?;
    let file_start = rest
        .find(suffix)
        .ok_or_else(|| anyhow::anyhow!("找不到 /resolve/main/"))?
        + suffix.len();

    let repo_id = &rest[..file_start - suffix.len()];
    let filename_with_query = &rest[file_start..];
    let filename = filename_with_query
        .split('?')
        .next()
        .unwrap_or(filename_with_query);

    println!("仓库: {}", repo_id);
    println!("文件名: {}", filename);

    // 开始下载
    let status = Command::new("hf")
        .arg("download")
        .arg(repo_id)
        .arg(filename)
        .arg("--local-dir")
        .arg(".")
        .status()?;

    if status.success() {
        println!("下载完成");
        Ok(())
    } else {
        anyhow::bail!("下载失败: code: {:?}", status.code());
    }
}

fn handle_modelscope(url: &str) -> anyhow::Result<()> {
    // https://modelscope.cn/models/deepseek-ai/DeepSeek-V3.2/file/view/master/assets%2Fpaper.pdf?status=1
    let prefix = "https://modelscope.cn/models/";
    let suffix = "/file/view/master/";

    if !url.starts_with(prefix) || !url.contains(suffix) {
        anyhow::bail!("不是一个有效的 hf 下载链接")
    }

    let rest = url.strip_prefix(prefix).context("去除前缀失败")?;
    let file_start = rest
        .find(suffix)
        .ok_or_else(|| anyhow::anyhow!("找不到 /file/view/master/"))?
        + suffix.len();

    let repo_id = &rest[..file_start - suffix.len()];
    let filename_with_query = &rest[file_start..];
    let filename = filename_with_query
        .split('?')
        .next()
        .unwrap_or(filename_with_query);

    println!("仓库: {}", repo_id);
    println!("文件名: {}", filename);

    // 开始下载
    let status = Command::new("modelscope")
        .arg("download")
        .arg("--model")
        .arg(repo_id)
        .arg(filename)
        .arg("--local_dir")
        .arg(".")
        .status()?;

    if status.success() {
        println!("下载完成");
    } else {
        anyhow::bail!("下载失败: code: {:?}", status.code());
    }

    // modelscope 如何下载文件包含路径的话，会有多余的目录层级，这里需要处理一下
    if filename.contains("/") {
        match std::env::consts::OS {
            "linux" | "macos" => {
                let status = Command::new("mv").arg(filename).arg(".").status()?;
                if !status.success() {
                    anyhow::bail!("文件移动失败 :{:?}", status.code());
                }
            }
            _ => println!("windows用户，请手动移动一下文件位置 :{}", filename),
        }
    }

    Ok(())
}

fn main() -> anyhow::Result<()> {
    check_hf_cli()?;
    check_model_scope_cli()?;

    let args = Args::parse();
    println!("{}", args.input);

    let url = args.input;

    if url.contains("huggingface.co") {
        // https://huggingface.co/comfyanonymous/flux_text_encoders/resolve/main/clip_l.safetensors
        handle_hf(&url)?
    } else if url.contains("modelscope.cn") {
        // https://modelscope.cn/models/deepseek-ai/DeepSeek-V3.2/file/view/master/assets%2Fpaper.pdf?status=1
        handle_modelscope(&url)?
    } else {
        println!("未识别的 url")
    }

    Ok(())
}
