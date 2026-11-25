use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaProbe {
    pub file_name: String,
    pub file_path: String,
    pub file_size: u64,
    pub file_size_human: String,
    pub format_name: String,
    pub format_long_name: String,
    pub duration_seconds: Option<f64>,
    pub duration_human: Option<String>,
    pub overall_bitrate_kbps: Option<f64>,
    pub container_tags: BTreeMap<String, String>,
    pub streams: Vec<MediaStream>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaStream {
    pub index: usize,
    pub stream_type: StreamKind,
    pub codec_name: String,
    pub codec_long_name: String,
    pub properties: BTreeMap<String, String>,
    pub tags: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StreamKind {
    Video,
    Audio,
    Subtitle,
    Data,
    Image,
    Other,
}

impl StreamKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            StreamKind::Video => "Video",
            StreamKind::Audio => "Audio",
            StreamKind::Subtitle => "Subtitle",
            StreamKind::Data => "Data",
            StreamKind::Image => "Image",
            StreamKind::Other => "Other",
        }
    }
}

impl MediaProbe {
    pub fn short_format(&self) -> String {
        match self.format_name.as_str() {
            "mov,mp4,m4a,3gp,3g2,mj2" => "MP4 / MOV".to_string(),
            "matroska,webm" => "MKV / WebM".to_string(),
            "png_pipe" => "PNG".to_string(),
            "image2 / jpeg_pipe" => "JPEG".to_string(),
            "webp_pipe" => "WebP".to_string(),
            "bmp_pipe" => "BMP".to_string(),
            "tiff_pipe" => "TIFF".to_string(),
            "avif_pipe" => "AVIF".to_string(),
            other => other.to_uppercase(),
        }
    }

    pub fn human_summary(&self) -> String {
        let mut parts = Vec::new();
        for s in &self.streams {
            let summary = s.human_summary();
            if summary.is_empty() {
                parts.push(format!("{} ({})", s.stream_type.as_str(), s.codec_name));
            } else {
                parts.push(format!("{} ({}, {})", s.stream_type.as_str(), s.codec_name, summary));
            }
        }
        if parts.is_empty() {
            self.format_long_name.clone()
        } else {
            parts.join("  •  ")
        }
    }
}

impl MediaStream {
    pub fn human_summary(&self) -> String {
        let mut parts = Vec::new();
        match self.stream_type {
            StreamKind::Video => {
                if let Some(res) = self.properties.get("resolution") {
                    parts.push(res.clone());
                }
                if let Some(fps) = self.properties.get("frame_rate") {
                    parts.push(fps.clone());
                }
                if let Some(ar) = self.properties.get("aspect_ratio") {
                    parts.push(ar.clone());
                }
                if let Some(br) = self.properties.get("bitrate") {
                    parts.push(br.clone());
                }
            }
            StreamKind::Audio => {
                if let Some(sr) = self.properties.get("sample_rate") {
                    parts.push(sr.clone());
                }
                if let Some(ch) = self.properties.get("channels") {
                    parts.push(ch.clone());
                }
                if let Some(bd) = self
                    .properties
                    .get("bits_per_sample")
                    .or_else(|| self.properties.get("bit_depth"))
                {
                    parts.push(bd.clone());
                }
                if let Some(br) = self.properties.get("bitrate") {
                    parts.push(br.clone());
                }
            }
            StreamKind::Image => {
                if let Some(res) = self.properties.get("resolution") {
                    parts.push(res.clone());
                }
                if let Some(ar) = self.properties.get("aspect_ratio") {
                    parts.push(ar.clone());
                }
            }
            StreamKind::Subtitle => {
                if let Some(lang) = self.properties.get("language") {
                    parts.push(lang.clone());
                }
                if let Some(name) = self.properties.get("name") {
                    parts.push(name.clone());
                }
            }
            _ => {}
        }
        parts.join(" • ")
    }
}

pub fn format_duration(seconds: f64) -> String {
    if seconds < 0.0 || !seconds.is_finite() {
        return "00:00:00.000".to_string();
    }
    let total_ms = (seconds * 1000.0).round() as u64;
    let ms = total_ms % 1000;
    let total_secs = total_ms / 1000;
    let s = total_secs % 60;
    let total_mins = total_secs / 60;
    let m = total_mins % 60;
    let h = total_mins / 60;
    format!("{:02}:{:02}:{:02}.{:03}", h, m, s, ms)
}

pub fn format_duration_friendly(seconds: f64) -> String {
    if seconds < 0.0 || !seconds.is_finite() {
        return "0s".to_string();
    }
    if seconds < 1.0 {
        return format!("{} ms", (seconds * 1000.0).round() as u64);
    }
    if seconds < 60.0 {
        return format!("{:.1}s", seconds);
    }
    let total_secs = seconds.round() as u64;
    let s = total_secs % 60;
    let total_mins = total_secs / 60;
    let m = total_mins % 60;
    let h = total_mins / 60;
    if h > 0 {
        format!("{}h {:02}m {:02}s", h, m, s)
    } else {
        format!("{}m {:02}s", m, s)
    }
}

pub fn format_bitrate_human(kbps: f64) -> String {
    if kbps >= 1000.0 {
        format!("{:.2} Mbps", kbps / 1000.0)
    } else {
        format!("{:.0} kbps", kbps)
    }
}

pub fn format_bytes_short(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

pub fn format_bytes(bytes: u64) -> String {
    if bytes >= 1024 {
        format!("{} ({} bytes)", format_bytes_short(bytes), bytes)
    } else {
        format!("{} bytes", bytes)
    }
}

fn format_sample_rate(hz: f64) -> String {
    if hz >= 1000.0 {
        let khz = hz / 1000.0;
        if (khz.fract()).abs() < 0.01 {
            format!("{:.0} kHz", khz)
        } else {
            format!("{:.1} kHz", khz)
        }
    } else {
        format!("{:.0} Hz", hz)
    }
}

fn format_channels(count: usize) -> String {
    match count {
        1 => "Mono (1 ch)".to_string(),
        2 => "Stereo (2 ch)".to_string(),
        6 => "5.1 Surround (6 ch)".to_string(),
        8 => "7.1 Surround (8 ch)".to_string(),
        n => format!("{} channels", n),
    }
}

fn gcd(mut a: u32, mut b: u32) -> u32 {
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a
}

pub fn calculate_aspect_ratio(width: u32, height: u32) -> String {
    if width == 0 || height == 0 {
        return "N/A".to_string();
    }
    let d = gcd(width, height);
    let rw = width / d;
    let rh = height / d;
    if (rw == 16 && rh == 9) || (rw == 4 && rh == 3) || (rw == 21 && rh == 9) || (rw == 1 && rh == 1) {
        format!("{}:{}", rw, rh)
    } else {
        let ratio = width as f64 / height as f64;
        format!("{}:{} ({:.2}:1)", rw, rh, ratio)
    }
}

pub fn probe_file<P: AsRef<Path>>(path: P) -> Result<MediaProbe, String> {
    let path = path.as_ref();
    let file_name = path
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "Unknown".to_string());
    let file_path = path.to_string_lossy().to_string();

    let meta = std::fs::metadata(path).map_err(|e| format!("Failed to read file metadata: {}", e))?;
    let file_size = meta.len();
    let file_size_human = format_bytes(file_size);

    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    if matches!(ext.as_str(), "mp4" | "m4v" | "mov" | "m4a" | "3gp" | "3g2") {
        if let Ok(probe) = probe_mp4(path, &file_name, &file_path, file_size, &file_size_human) {
            return Ok(probe);
        }
    }

    if matches!(ext.as_str(), "mkv" | "webm") {
        if let Ok(probe) = probe_matroska(path, &file_name, &file_path, file_size, &file_size_human) {
            return Ok(probe);
        }
    }

    if matches!(ext.as_str(), "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "tiff" | "avif" | "ico") {
        if let Ok(probe) = probe_image(path, &file_name, &file_path, file_size, &file_size_human, &ext) {
            return Ok(probe);
        }
    }

    if let Ok(probe) = probe_symphonia(path, &file_name, &file_path, file_size, &file_size_human) {
        return Ok(probe);
    }

    if let Ok(probe) = probe_mp4(path, &file_name, &file_path, file_size, &file_size_human) {
        return Ok(probe);
    }
    if let Ok(probe) = probe_matroska(path, &file_name, &file_path, file_size, &file_size_human) {
        return Ok(probe);
    }
    if let Ok(probe) = probe_image(path, &file_name, &file_path, file_size, &file_size_human, &ext) {
        return Ok(probe);
    }

    Err(format!(
        "Unable to recognize media format for '{}'. Format is unsupported or corrupted.",
        file_name
    ))
}

fn probe_mp4(
    path: &Path,
    file_name: &str,
    file_path: &str,
    file_size: u64,
    file_size_human: &str,
) -> Result<MediaProbe, String> {
    let file = File::open(path).map_err(|e| e.to_string())?;
    let reader = BufReader::new(file);
    let mp4 = mp4::Mp4Reader::read_header(reader, file_size).map_err(|e| e.to_string())?;

    let dur_secs = mp4.duration().as_secs_f64();
    let duration_seconds = if dur_secs > 0.0 { Some(dur_secs) } else { None };
    let duration_human = duration_seconds.map(format_duration);
    let overall_bitrate_kbps = duration_seconds.and_then(|d| {
        if d > 0.0 {
            Some((file_size as f64 * 8.0) / (d * 1000.0))
        } else {
            None
        }
    });

    let mut container_tags = BTreeMap::new();
    container_tags.insert("major_brand".to_string(), mp4.ftyp.major_brand.to_string());
    container_tags.insert("minor_version".to_string(), mp4.ftyp.minor_version.to_string());
    let compat: Vec<String> = mp4.ftyp.compatible_brands.iter().map(|b| b.to_string()).collect();
    if !compat.is_empty() {
        container_tags.insert("compatible_brands".to_string(), compat.join(", "));
    }

    let mut streams = Vec::new();
    let mut sorted_tracks: Vec<_> = mp4.tracks().iter().collect();
    sorted_tracks.sort_by_key(|(id, _)| *id);

    for (idx, (_, track)) in sorted_tracks.into_iter().enumerate() {
        let track_type = track.track_type().unwrap_or(mp4::TrackType::Video);
        let stream_type = match track_type {
            mp4::TrackType::Video => StreamKind::Video,
            mp4::TrackType::Audio => StreamKind::Audio,
            mp4::TrackType::Subtitle => StreamKind::Subtitle,
        };

        let media_type = track.media_type().ok();
        let box_type = track.box_type().ok();
        let codec_name = box_type
            .map(|b| b.to_string())
            .or_else(|| media_type.map(|m| format!("{:?}", m)))
            .unwrap_or_else(|| "Unknown".to_string());

        let codec_long_name = match codec_name.trim().to_ascii_lowercase().as_str() {
            "avc1" | "h264" => "H.264 / AVC",
            "hev1" | "hvc1" | "hevc" => "H.265 / HEVC",
            "av01" => "AV1 (Alliance for Open Media)",
            "vp09" | "vp9" => "Google VP9",
            "mp4v" => "MPEG-4 Part 2",
            "mp4a" => "AAC (Advanced Audio Coding)",
            "alac" => "Apple Lossless (ALAC)",
            "ac-3" => "Dolby Digital (AC-3)",
            "ec-3" => "Dolby Digital Plus (E-AC-3)",
            "tx3g" => "MP4 Timed Text Subtitles",
            other => other,
        }
        .to_string();

        let mut properties = BTreeMap::new();
        properties.insert("track_id".to_string(), track.track_id().to_string());
        properties.insert("timescale".to_string(), track.timescale().to_string());
        let tr_dur = track.duration().as_secs_f64();
        if tr_dur > 0.0 {
            properties.insert(
                "duration".to_string(),
                format!("{} ({})", format_duration_friendly(tr_dur), format_duration(tr_dur)),
            );
        }
        let bitrate = track.bitrate();
        if bitrate > 0 {
            properties.insert("bitrate".to_string(), format_bitrate_human(bitrate as f64 / 1000.0));
        }

        match stream_type {
            StreamKind::Video => {
                let w = track.width();
                let h = track.height();
                properties.insert("resolution".to_string(), format!("{} × {}", w, h));
                properties.insert("aspect_ratio".to_string(), calculate_aspect_ratio(w as u32, h as u32));
                let fps = track.frame_rate();
                if fps > 0.0 {
                    properties.insert("frame_rate".to_string(), format!("{:.2} fps", fps));
                }
                properties.insert("sample_count".to_string(), track.sample_count().to_string());
            }
            StreamKind::Audio => {
                if let Ok(ch) = track.channel_config() {
                    let ch_num = match ch {
                        mp4::ChannelConfig::Mono => 1,
                        mp4::ChannelConfig::Stereo => 2,
                        mp4::ChannelConfig::Three => 3,
                        mp4::ChannelConfig::Four => 4,
                        mp4::ChannelConfig::Five => 5,
                        mp4::ChannelConfig::FiveOne => 6,
                        mp4::ChannelConfig::SevenOne => 8,
                    };
                    properties.insert("channels".to_string(), format_channels(ch_num));
                }
                if let Ok(freq) = track.sample_freq_index() {
                    properties.insert("sample_rate".to_string(), format_sample_rate(freq.freq() as f64));
                }
                properties.insert("sample_count".to_string(), track.sample_count().to_string());
            }
            _ => {}
        }

        streams.push(MediaStream {
            index: idx,
            stream_type,
            codec_name,
            codec_long_name,
            properties,
            tags: BTreeMap::new(),
        });
    }

    Ok(MediaProbe {
        file_name: file_name.to_string(),
        file_path: file_path.to_string(),
        file_size,
        file_size_human: file_size_human.to_string(),
        format_name: "mov,mp4,m4a,3gp,3g2,mj2".to_string(),
        format_long_name: "QuickTime / MOV / MP4 / M4A (ISO Base Media File)".to_string(),
        duration_seconds,
        duration_human,
        overall_bitrate_kbps,
        container_tags,
        streams,
    })
}

fn probe_matroska(
    path: &Path,
    file_name: &str,
    file_path: &str,
    file_size: u64,
    file_size_human: &str,
) -> Result<MediaProbe, String> {
    let mat = matroska::open(path).map_err(|e| format!("Matroska error: {:?}", e))?;

    let duration_seconds = mat.info.duration.map(|d| d.as_secs_f64());
    let duration_human = duration_seconds.map(format_duration);
    let overall_bitrate_kbps = duration_seconds.and_then(|d| {
        if d > 0.0 {
            Some((file_size as f64 * 8.0) / (d * 1000.0))
        } else {
            None
        }
    });

    let mut container_tags = BTreeMap::new();
    if let Some(ref t) = mat.info.title {
        container_tags.insert("title".to_string(), t.clone());
    }
    if !mat.info.muxing_app.is_empty() {
        container_tags.insert("muxing_app".to_string(), mat.info.muxing_app.clone());
    }
    if !mat.info.writing_app.is_empty() {
        container_tags.insert("writing_app".to_string(), mat.info.writing_app.clone());
    }
    if let Some(ref date) = mat.info.date_utc {
        container_tags.insert("date_utc".to_string(), format!("{:?}", date));
    }

    let mut streams = Vec::new();
    for (idx, track) in mat.tracks.iter().enumerate() {
        let stream_type = match track.tracktype {
            matroska::Tracktype::Video => StreamKind::Video,
            matroska::Tracktype::Audio => StreamKind::Audio,
            matroska::Tracktype::Subtitle => StreamKind::Subtitle,
            _ => StreamKind::Other,
        };

        let codec_name = track.codec_id.clone();
        let codec_long_name = track
            .codec_name
            .clone()
            .unwrap_or_else(|| match codec_name.as_str() {
                "V_MPEG4/ISO/AVC" => "H.264 / AVC".to_string(),
                "V_MPEGH/ISO/HEVC" => "H.265 / HEVC".to_string(),
                "V_VP8" => "VP8".to_string(),
                "V_VP9" => "VP9".to_string(),
                "V_AV1" => "AV1".to_string(),
                "A_AAC" => "AAC (Advanced Audio Coding)".to_string(),
                "A_OPUS" => "Opus Audio".to_string(),
                "A_VORBIS" => "Vorbis Audio".to_string(),
                "A_FLAC" => "FLAC (Free Lossless Audio)".to_string(),
                "S_TEXT/UTF8" => "SubRip Subtitles (SRT)".to_string(),
                "S_TEXT/ASS" => "Advanced SubStation Alpha (ASS)".to_string(),
                other => other.to_string(),
            });

        let mut properties = BTreeMap::new();
        properties.insert("track_number".to_string(), track.number.to_string());
        if let Some(ref lang) = track.language {
            properties.insert("language".to_string(), lang.to_string());
        }
        if let Some(ref name) = track.name {
            properties.insert("name".to_string(), name.clone());
        }
        properties.insert(
            "default_flag".to_string(),
            if track.default { "Yes" } else { "No" }.to_string(),
        );
        properties.insert(
            "forced_flag".to_string(),
            if track.forced { "Yes" } else { "No" }.to_string(),
        );

        match &track.settings {
            matroska::Settings::Video(v) => {
                let w = v.pixel_width as u32;
                let h = v.pixel_height as u32;
                properties.insert("resolution".to_string(), format!("{} × {}", w, h));
                properties.insert("aspect_ratio".to_string(), calculate_aspect_ratio(w, h));
                if let (Some(dw), Some(dh)) = (v.display_width, v.display_height) {
                    properties.insert("display_resolution".to_string(), format!("{} × {}", dw, dh));
                }
                if let Some(interlaced) = v.interlaced {
                    properties.insert(
                        "interlaced".to_string(),
                        if interlaced { "Yes" } else { "No (Progressive)" }.to_string(),
                    );
                }
            }
            matroska::Settings::Audio(a) => {
                properties.insert("channels".to_string(), format_channels(a.channels as usize));
                properties.insert("sample_rate".to_string(), format_sample_rate(a.sample_rate));
                if let Some(depth) = a.bit_depth {
                    properties.insert("bit_depth".to_string(), format!("{}-bit", depth));
                }
            }
            _ => {}
        }

        streams.push(MediaStream {
            index: idx,
            stream_type,
            codec_name,
            codec_long_name,
            properties,
            tags: BTreeMap::new(),
        });
    }

    Ok(MediaProbe {
        file_name: file_name.to_string(),
        file_path: file_path.to_string(),
        file_size,
        file_size_human: file_size_human.to_string(),
        format_name: "matroska,webm".to_string(),
        format_long_name: "Matroska / WebM Multimedia Container".to_string(),
        duration_seconds,
        duration_human,
        overall_bitrate_kbps,
        container_tags,
        streams,
    })
}

fn probe_symphonia(
    path: &Path,
    file_name: &str,
    file_path: &str,
    file_size: u64,
    file_size_human: &str,
) -> Result<MediaProbe, String> {
    use symphonia::core::formats::probe::Hint;
    use symphonia::core::formats::FormatOptions;
    use symphonia::core::io::MediaSourceStream;
    use symphonia::core::meta::MetadataOptions;

    let src = File::open(path).map_err(|e| format!("File open error: {}", e))?;
    let mss = MediaSourceStream::new(Box::new(src), Default::default());

    let mut hint = Hint::new();
    if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
        hint.with_extension(ext);
    }

    let fmt_opts = FormatOptions::default();
    let meta_opts = MetadataOptions::default();

    let mut reader = symphonia::default::get_probe()
        .probe(&hint, mss, fmt_opts, meta_opts)
        .map_err(|e| format!("Symphonia probe error: {:?}", e))?;

    let mut container_tags = BTreeMap::new();
    if let Some(metadata) = reader.metadata().current() {
        for tag in &metadata.media.tags {
            let key = tag
                .std
                .as_ref()
                .map(|k| format!("{:?}", k))
                .unwrap_or_else(|| tag.raw.key.clone());
            container_tags.insert(key, tag.raw.value.to_string());
        }
    }

    let mut max_dur_secs: Option<f64> = None;
    let mut streams = Vec::new();

    for (idx, track) in reader.tracks().iter().enumerate() {
        let mut properties = BTreeMap::new();
        properties.insert("track_id".to_string(), track.id.to_string());

        if let Some(ref lang) = track.language {
            properties.insert("language".to_string(), lang.clone());
        }

        if let Some(tb) = track.time_base {
            properties.insert("time_base".to_string(), format!("{}/{}", tb.numer.get(), tb.denom.get()));
        }

        if let (Some(tb), Some(n_frames)) = (track.time_base, track.num_frames) {
            let dur = (n_frames as f64) * (tb.numer.get() as f64) / (tb.denom.get() as f64);
            if dur > 0.0 {
                properties.insert(
                    "duration".to_string(),
                    format!("{} ({})", format_duration_friendly(dur), format_duration(dur)),
                );
                max_dur_secs = Some(max_dur_secs.map_or(dur, |m| m.max(dur)));
            }
        }

        let (stream_type, codec_name, codec_long_name) = match &track.codec_params {
            Some(symphonia::core::codecs::CodecParameters::Audio(p)) => {
                let decoder = symphonia::default::get_codecs().get_audio_decoder(p.codec);
                let (short, long) = if let Some(dec) = decoder {
                    (dec.codec.info.short_name.to_string(), dec.codec.info.long_name.to_string())
                } else {
                    (format!("{:?}", p.codec), "Audio Codec".to_string())
                };
                if let Some(sr) = p.sample_rate {
                    properties.insert("sample_rate".to_string(), format_sample_rate(sr as f64));
                }
                if let Some(ref channels) = p.channels {
                    properties.insert("channels".to_string(), format_channels(channels.count()));
                }
                if let Some(bps) = p.bits_per_sample {
                    properties.insert("bits_per_sample".to_string(), format!("{}-bit", bps));
                }
                if let Some(br) = p.bits_per_coded_sample {
                    properties.insert("bits_per_coded_sample".to_string(), format!("{}-bit", br));
                }
                (StreamKind::Audio, short, long)
            }
            Some(symphonia::core::codecs::CodecParameters::Video(p)) => {
                (StreamKind::Video, format!("{:?}", p.codec), "Video Codec".to_string())
            }
            Some(symphonia::core::codecs::CodecParameters::Subtitle(p)) => {
                (StreamKind::Subtitle, format!("{:?}", p.codec), "Subtitle Codec".to_string())
            }
            _ => (StreamKind::Other, "unknown".to_string(), "Unknown Codec".to_string()),
        };

        streams.push(MediaStream {
            index: idx,
            stream_type,
            codec_name,
            codec_long_name,
            properties,
            tags: BTreeMap::new(),
        });
    }

    let duration_seconds = max_dur_secs;
    let duration_human = duration_seconds.map(format_duration);
    let overall_bitrate_kbps = duration_seconds.and_then(|d| {
        if d > 0.0 {
            Some((file_size as f64 * 8.0) / (d * 1000.0))
        } else {
            None
        }
    });

    let format_ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("media")
        .to_ascii_lowercase();

    let (format_name, format_long_name) = match format_ext.as_str() {
        "mp3" => ("mp3", "MP3 (MPEG Audio Layer III)"),
        "flac" => ("flac", "FLAC (Free Lossless Audio Codec)"),
        "wav" => ("wav", "WAV (Waveform Audio File Format)"),
        "ogg" => ("ogg", "Ogg Multimedia Container"),
        "opus" => ("opus", "Ogg Opus Audio"),
        "aiff" | "aif" => ("aiff", "AIFF (Audio Interchange File Format)"),
        "caf" => ("caf", "Apple Core Audio Format (CAF)"),
        "aac" => ("aac", "ADTS AAC (Advanced Audio Coding)"),
        other => (other, "Multimedia Container / Audio Stream"),
    };

    Ok(MediaProbe {
        file_name: file_name.to_string(),
        file_path: file_path.to_string(),
        file_size,
        file_size_human: file_size_human.to_string(),
        format_name: format_name.to_string(),
        format_long_name: format_long_name.to_string(),
        duration_seconds,
        duration_human,
        overall_bitrate_kbps,
        container_tags,
        streams,
    })
}

fn probe_image(
    path: &Path,
    file_name: &str,
    file_path: &str,
    file_size: u64,
    file_size_human: &str,
    ext: &str,
) -> Result<MediaProbe, String> {
    use imagesize::size;

    let dim = size(path).map_err(|e| format!("Image probe error: {:?}", e))?;
    let w = dim.width as u32;
    let h = dim.height as u32;

    let (fmt_name, fmt_long) = match ext {
        "png" => ("png_pipe", "PNG (Portable Network Graphics)"),
        "jpg" | "jpeg" => ("image2 / jpeg_pipe", "JPEG / JFIF Image"),
        "webp" => ("webp_pipe", "WebP Image"),
        "gif" => ("gif", "GIF (Graphics Interchange Format)"),
        "bmp" => ("bmp_pipe", "BMP (Windows Bitmap)"),
        "tiff" | "tif" => ("tiff_pipe", "TIFF (Tagged Image File Format)"),
        "avif" => ("avif_pipe", "AVIF Image"),
        "ico" => ("ico", "Windows Icon"),
        _ => (ext, "Still Image"),
    };

    let mut properties = BTreeMap::new();
    properties.insert("resolution".to_string(), format!("{} × {}", w, h));
    properties.insert("aspect_ratio".to_string(), calculate_aspect_ratio(w, h));

    let stream = MediaStream {
        index: 0,
        stream_type: StreamKind::Image,
        codec_name: ext.to_ascii_uppercase(),
        codec_long_name: fmt_long.to_string(),
        properties,
        tags: BTreeMap::new(),
    };

    Ok(MediaProbe {
        file_name: file_name.to_string(),
        file_path: file_path.to_string(),
        file_size,
        file_size_human: file_size_human.to_string(),
        format_name: fmt_name.to_string(),
        format_long_name: fmt_long.to_string(),
        duration_seconds: None,
        duration_human: None,
        overall_bitrate_kbps: None,
        container_tags: BTreeMap::new(),
        streams: vec![stream],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(0.0), "00:00:00.000");
        assert_eq!(format_duration(65.5), "00:01:05.500");
        assert_eq!(format_duration(3661.123), "01:01:01.123");
        assert_eq!(format_duration_friendly(65.5), "1m 06s");
        assert_eq!(format_duration_friendly(3661.0), "1h 01m 01s");
    }

    #[test]
    fn test_aspect_ratio() {
        assert_eq!(calculate_aspect_ratio(1920, 1080), "16:9");
        assert_eq!(calculate_aspect_ratio(3840, 2160), "16:9");
        assert_eq!(calculate_aspect_ratio(1280, 720), "16:9");
        assert_eq!(calculate_aspect_ratio(640, 480), "4:3");
        assert_eq!(calculate_aspect_ratio(1080, 1080), "1:1");
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(500), "500 bytes");
        assert_eq!(format_bytes(2048), "2.0 KB (2048 bytes)");
        assert_eq!(format_bytes_short(2048), "2.0 KB");
    }

    #[test]
    fn test_probe_wav() {
        let mut wav_bytes = Vec::new();
        wav_bytes.extend_from_slice(b"RIFF");
        let chunk_size: u32 = 36;
        wav_bytes.extend_from_slice(&chunk_size.to_le_bytes());
        wav_bytes.extend_from_slice(b"WAVE");
        wav_bytes.extend_from_slice(b"fmt ");
        let subchunk1_size: u32 = 16;
        wav_bytes.extend_from_slice(&subchunk1_size.to_le_bytes());
        let audio_format: u16 = 1;
        wav_bytes.extend_from_slice(&audio_format.to_le_bytes());
        let num_channels: u16 = 2;
        wav_bytes.extend_from_slice(&num_channels.to_le_bytes());
        let sample_rate: u32 = 44100;
        wav_bytes.extend_from_slice(&sample_rate.to_le_bytes());
        let byte_rate: u32 = sample_rate * 2 * 2;
        wav_bytes.extend_from_slice(&byte_rate.to_le_bytes());
        let block_align: u16 = 4;
        wav_bytes.extend_from_slice(&block_align.to_le_bytes());
        let bits_per_sample: u16 = 16;
        wav_bytes.extend_from_slice(&bits_per_sample.to_le_bytes());
        wav_bytes.extend_from_slice(b"data");
        let subchunk2_size: u32 = 0;
        wav_bytes.extend_from_slice(&subchunk2_size.to_le_bytes());

        let temp_dir = std::env::temp_dir();
        let temp_path = temp_dir.join("test_synthetic_sample.wav");
        std::fs::write(&temp_path, &wav_bytes).unwrap();

        let probe = probe_file(&temp_path).expect("Should probe synthetic WAV");
        assert_eq!(probe.format_name, "wav");
        assert_eq!(probe.streams.len(), 1);
        let stream = &probe.streams[0];
        assert_eq!(stream.stream_type, StreamKind::Audio);
        assert_eq!(stream.properties.get("sample_rate").unwrap(), "44.1 kHz");
        assert_eq!(stream.properties.get("channels").unwrap(), "Stereo (2 ch)");

        let _ = std::fs::remove_file(temp_path);
    }
}
