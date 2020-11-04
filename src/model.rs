use anyhow::Result;
use serde::Deserialize;

pub trait SegmentDownload {
    fn base_url(&self) -> &str;
    fn init_segment(&self) -> Result<Vec<u8>>;
    fn mime_type(&self) -> &str;
    fn segments(&self) -> &[Segment];
}

#[derive(Debug, Clone, Deserialize)]
pub struct Segment {
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VimeoVideo {
    pub base_url: String,
    pub bitrate: u64,
    pub width: u64,
    pub height: u64,
    pub mime_type: String,
    pub init_segment: String,
    pub segments: Vec<Segment>,
}

impl SegmentDownload for VimeoVideo {
    fn base_url(&self) -> &str {
        &self.base_url
    }

    fn init_segment(&self) -> Result<Vec<u8>> {
        base64::decode(&self.init_segment).map_err(|e| e.into())
    }

    fn mime_type(&self) -> &str {
        &self.mime_type
    }

    fn segments(&self) -> &[Segment] {
        &self.segments
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct VimeoAudio {
    pub base_url: String,
    pub bitrate: u32,
    pub mime_type: String,
    pub init_segment: String,
    pub segments: Vec<Segment>,
}

impl SegmentDownload for VimeoAudio {
    fn base_url(&self) -> &str {
        &self.base_url
    }

    fn mime_type(&self) -> &str {
        &self.mime_type
    }

    fn init_segment(&self) -> Result<Vec<u8>> {
        base64::decode(&self.init_segment).map_err(|e| e.into())
    }

    fn segments(&self) -> &[Segment] {
        &self.segments
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct VimeoResponse {
    pub clip_id: String,
    pub base_url: String,
    pub audio: Vec<VimeoAudio>,
    pub video: Vec<VimeoVideo>,
}
