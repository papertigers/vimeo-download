use anyhow::{anyhow, Context, Result};
use reqwest::blocking as _reqwest;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use structopt::StructOpt;
use tempfile::NamedTempFile;
use url::Url;

mod model;
use crate::model::*;

#[derive(Debug, StructOpt)]
struct Opt {
    url: Url,
    #[structopt(long, short)]
    filename: Option<String>,
    #[structopt(long, short)]
    directory: Option<String>,
    #[structopt(long, short)]
    verbose: bool,
}

fn download_segments<D>(u: Url, d: D, verbose: bool) -> Result<NamedTempFile>
where
    D: SegmentDownload + 'static,
{
    let base_url = Url::join(&u, d.base_url())?;
    let init_segment = d.init_segment().context("failed to decode init_segment")?;
    let mut w = NamedTempFile::new()?;
    w.write_all(&init_segment)?;

    for seg in d.segments().iter() {
        let s = base_url.join(&seg.url)?;
        w.write_all(&_reqwest::get(s)?.bytes()?)?;
        if verbose {
            println!("downloaded: {} ({:?})", seg.url, d.mime_type());
        }
    }

    w.flush()?;
    Ok(w)
}

fn merge<A, V, O>(audio: A, video: V, name: &str, output_dir: O) -> Result<()>
where
    A: AsRef<Path>,
    V: AsRef<Path>,
    O: AsRef<Path>,
{
    let status = Command::new("ffmpeg")
        .arg("-i")
        .arg(audio.as_ref())
        .arg("-i")
        .arg(video.as_ref())
        .arg("-c")
        .arg("copy")
        .arg(output_dir.as_ref().join(name))
        .status()
        .context("Failed to combine audio and video with ffmpeg")?;

    if !status.success() {
        return Err(anyhow!("ffmpeg exited with status: {:?}", status.code()));
    }
    Ok(())
}

fn main() -> Result<()> {
    let opt = Opt::from_args();
    let mut url = opt.url.clone();

    // ensure that we get the init segment as a base64 value
    url.query_pairs_mut()
        .clear()
        .append_pair("base64_init", "1");

    let master: VimeoResponse = _reqwest::get(url.clone())
        .context("failed to get response from vimeo")?
        .json()
        .context("failed to parse vimeo master.json")?;
    let base_url = Url::join(&url, &master.base_url)?;

    // grab the best video quality available
    let video = master
        .video
        .iter()
        .fold(None, |best: Option<&VimeoVideo>, v| match best {
            None => Some(v),
            Some(b) if (v.height * v.width) > (b.height * b.width) => Some(v),
            _ => best,
        })
        .ok_or_else(|| anyhow!("Vimeo URL contained no video streams"))?
        .clone();

    // grab the best audio quality available
    let audio = master
        .audio
        .iter()
        .fold(None, |best: Option<&VimeoAudio>, a| match best {
            None => Some(a),
            Some(b) if a.bitrate > b.bitrate => Some(a),
            _ => best,
        })
        .ok_or_else(|| anyhow!("Vimeo URL contained no audio streams"))?
        .clone();

    let verbose = opt.verbose;
    let mime = video
        .mime_type
        .parse::<mime::Mime>()
        .context("failed to read video mime-type")?;

    let video_base = base_url.clone();
    let v = thread::Builder::new()
        .name("video_downloader".to_string())
        .spawn(move || download_segments(video_base, video, verbose))
        .expect("failed to spawn video thread");

    let audio_base = base_url;
    let a = thread::Builder::new()
        .name("audio_downloader".to_string())
        .spawn(move || download_segments(audio_base, audio, verbose))
        .expect("failed to spawn audio thread");

    let video = v.join().unwrap()?;
    let audio = a.join().unwrap()?;

    let out_dir = opt
        .directory
        .map(PathBuf::from)
        .unwrap_or(std::env::current_dir()?);

    let filename = opt.filename.unwrap_or_else(|| {
        let file_ext = mime.suffix().map(|n| n.as_str()).unwrap_or("mp4");
        format!("{}.{}", master.clip_id, file_ext)
    });

    merge(&audio, &video, &filename, &out_dir)?;

    println!("\n\nVideo output: {:?}", &out_dir.join(&filename));

    Ok(())
}
