#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod probe;

use gpui::{
    div, img, prelude::*, px, rgb, size, AnyElement, App, AppContext, Application, AsyncApp,
    Bounds, ClipboardItem, Context, Entity, ExternalPaths, FontWeight, Image, ImageFormat, Window,
    WindowBounds, WindowOptions,
};
use probe::{
    format_bitrate_human, format_bytes_short, format_duration_friendly, probe_file, MediaProbe,
    StreamKind,
};
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::Arc;

const APP_NAME: &str = "MediaProbe";
const ICON_PNG: &[u8] = include_bytes!("../assets/icon.png");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ActiveTab {
    Overview,
    Streams,
    Metadata,
    Json,
}

struct AppState {
    current_probe: Option<MediaProbe>,
    error_message: Option<String>,
    active_tab: ActiveTab,
    copied_status: Option<&'static str>,
    app_icon: Arc<Image>,
}

impl AppState {
    fn new(initial_path: Option<PathBuf>) -> Self {
        let mut state = Self {
            current_probe: None,
            error_message: None,
            active_tab: ActiveTab::Overview,
            copied_status: None,
            app_icon: Arc::new(Image::from_bytes(ImageFormat::Png, ICON_PNG.to_vec())),
        };
        if let Some(p) = initial_path {
            state.load_file(&p);
        }
        state
    }

    fn load_file(&mut self, path: &Path) {
        self.copied_status = None;
        match probe_file(path) {
            Ok(probe) => {
                self.current_probe = Some(probe);
                self.error_message = None;
            }
            Err(err) => {
                self.error_message = Some(err);
            }
        }
    }
}

// Modern web-inspired neutral dark palette (Zinc + White)
const BG_PAGE: u32 = 0x09090b;
const BG_CARD: u32 = 0x111114;
const BG_SUBTLE: u32 = 0x18181b;
const BG_HOVER: u32 = 0x27272a;
const BG_CODE: u32 = 0x0c0c0e;
const BORDER_SUBTLE: u32 = 0x27272a;
const BORDER_STRONG: u32 = 0x3f3f46;
const TEXT_WHITE: u32 = 0xffffff;
const TEXT_PRIMARY: u32 = 0xf4f4f5;
const TEXT_MUTED: u32 = 0xa1a1aa;
const TEXT_DIM: u32 = 0x71717a;
const BTN_PRIMARY_BG: u32 = 0xfafafa;
const BTN_PRIMARY_HOVER: u32 = 0xe4e4e7;
const BTN_PRIMARY_TEXT: u32 = 0x09090b;
const COLOR_SUCCESS: u32 = 0x4ade80;
const COLOR_ERROR: u32 = 0xf87171;

impl Render for AppState {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let has_probe = self.current_probe.is_some();

        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(rgb(BG_PAGE))
            .text_color(rgb(TEXT_PRIMARY))
            .id("root-container")
            .on_drop(cx.listener(|this, external_paths: &ExternalPaths, _, cx| {
                if let Some(path) = external_paths.paths().first() {
                    this.load_file(path);
                    cx.notify();
                }
            }))
            .child(self.render_header(cx))
            .child(
                div()
                    .flex_1()
                    .w_full()
                    .id("main-scroll-pane")
                    .overflow_y_scroll()
                    .child(
                        div()
                            .w_full()
                            .flex()
                            .flex_col()
                            .items_center()
                            .px_6()
                            .py_6()
                            .child(
                                div()
                                    .w_full()
                                    .max_w(px(960.0))
                                    .flex()
                                    .flex_col()
                                    .gap_5()
                                    .when_some(self.error_message.as_ref(), |this, err| {
                                        this.child(
                                            div()
                                                .p_4()
                                                .bg(rgb(0x1f1215))
                                                .border_1()
                                                .border_color(rgb(0x4c1d24))
                                                .rounded_lg()
                                                .text_sm()
                                                .text_color(rgb(COLOR_ERROR))
                                                .child(format!("Could not read file: {}", err)),
                                        )
                                    })
                                    .child(if has_probe {
                                        self.render_probe_view(cx)
                                    } else {
                                        self.render_empty_state(cx)
                                    }),
                            ),
                    ),
            )
    }
}

impl AppState {
    fn render_header(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let has_probe = self.current_probe.is_some();
        let status_msg = self.copied_status;

        div()
            .w_full()
            .h(px(60.0))
            .px_6()
            .flex()
            .flex_row()
            .items_center()
            .justify_center()
            .bg(rgb(BG_PAGE))
            .border_b_1()
            .border_color(rgb(BORDER_SUBTLE))
            .child(
                div()
                    .w_full()
                    .max_w(px(960.0))
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_3()
                            .child(img(self.app_icon.clone()).w(px(28.0)).h(px(28.0)))
                            .child(
                                div()
                                    .text_base()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(rgb(TEXT_WHITE))
                                    .child(APP_NAME),
                            )
                            .child(
                                div()
                                    .px_2()
                                    .py(px(2.0))
                                    .bg(rgb(BG_SUBTLE))
                                    .border_1()
                                    .border_color(rgb(BORDER_SUBTLE))
                                    .rounded_full()
                                    .text_xs()
                                    .text_color(rgb(TEXT_MUTED))
                                    .child("Media Inspector"),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_2()
                            .when_some(status_msg, |this, msg| {
                                this.child(
                                    div()
                                        .mr_2()
                                        .text_xs()
                                        .text_color(rgb(COLOR_SUCCESS))
                                        .child(msg),
                                )
                            })
                            .when(has_probe, |this| {
                                this.child(
                                    div()
                                        .px_3()
                                        .py(px(6.0))
                                        .bg(rgb(BG_SUBTLE))
                                        .border_1()
                                        .border_color(rgb(BORDER_SUBTLE))
                                        .rounded_md()
                                        .cursor_pointer()
                                        .hover(|s| {
                                            s.bg(rgb(BG_HOVER))
                                                .border_color(rgb(BORDER_STRONG))
                                        })
                                        .text_sm()
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(rgb(TEXT_WHITE))
                                        .id("copy-json-btn")
                                        .child("Copy JSON")
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            if let Some(ref probe) = this.current_probe {
                                                if let Ok(json) =
                                                    serde_json::to_string_pretty(probe)
                                                {
                                                    cx.write_to_clipboard(
                                                        ClipboardItem::new_string(json),
                                                    );
                                                    this.copied_status = Some("Copied JSON");
                                                    cx.notify();
                                                }
                                            }
                                        })),
                                )
                            })
                            .child(
                                div()
                                    .px_3()
                                    .py(px(6.0))
                                    .bg(rgb(BTN_PRIMARY_BG))
                                    .rounded_md()
                                    .cursor_pointer()
                                    .hover(|s| s.bg(rgb(BTN_PRIMARY_HOVER)))
                                    .text_sm()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(rgb(BTN_PRIMARY_TEXT))
                                    .id("open-file-btn")
                                    .child("Open File")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        if let Some(path) = rfd::FileDialog::new().pick_file() {
                                            this.load_file(&path);
                                            cx.notify();
                                        }
                                    })),
                            ),
                    ),
            )
    }

    fn render_empty_state(&self, cx: &mut Context<Self>) -> AnyElement {
        div()
            .w_full()
            .py_16()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .child(
                div()
                    .w_full()
                    .max_w(px(560.0))
                    .p_10()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .border_1()
                    .border_dashed()
                    .border_color(rgb(BORDER_STRONG))
                    .rounded_xl()
                    .bg(rgb(BG_CARD))
                    .hover(|s| s.bg(rgb(BG_SUBTLE)).border_color(rgb(TEXT_MUTED)))
                    .cursor_pointer()
                    .id("drop-box")
                    .on_click(cx.listener(|this, _, _, cx| {
                        if let Some(path) = rfd::FileDialog::new().pick_file() {
                            this.load_file(&path);
                            cx.notify();
                        }
                    }))
                    .child(img(self.app_icon.clone()).w(px(52.0)).h(px(52.0)).mb_4())
                    .child(
                        div()
                            .text_lg()
                            .font_weight(FontWeight::SEMIBOLD)
                            .mb_1()
                            .text_color(rgb(TEXT_WHITE))
                            .child("Inspect any media file"),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(TEXT_MUTED))
                            .mb_6()
                            .child("Drop a video, audio, or image file here — or click to browse"),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .flex_wrap()
                            .justify_center()
                            .gap_2()
                            .child(pill("MP4"))
                            .child(pill("MKV"))
                            .child(pill("MOV"))
                            .child(pill("WebM"))
                            .child(pill("MP3"))
                            .child(pill("FLAC"))
                            .child(pill("WAV"))
                            .child(pill("OGG"))
                            .child(pill("Images")),
                    ),
            )
            .into_any_element()
    }

    fn render_probe_view(&self, cx: &mut Context<Self>) -> AnyElement {
        let probe = match &self.current_probe {
            Some(p) => p,
            None => return div().child("No media loaded").into_any_element(),
        };

        div()
            .flex()
            .flex_col()
            .gap_5()
            .child(self.render_summary_card(probe))
            .child(self.render_tabs_bar(probe, cx))
            .child(match self.active_tab {
                ActiveTab::Overview => self.render_overview_tab(probe),
                ActiveTab::Streams => self.render_streams_tab(probe),
                ActiveTab::Metadata => self.render_metadata_tab(probe),
                ActiveTab::Json => self.render_json_tab(probe),
            })
            .into_any_element()
    }

    fn render_summary_card(&self, probe: &MediaProbe) -> impl IntoElement {
        let video_count = probe
            .streams
            .iter()
            .filter(|s| s.stream_type == StreamKind::Video)
            .count();
        let audio_count = probe
            .streams
            .iter()
            .filter(|s| s.stream_type == StreamKind::Audio)
            .count();
        let sub_count = probe
            .streams
            .iter()
            .filter(|s| s.stream_type == StreamKind::Subtitle)
            .count();
        let img_count = probe
            .streams
            .iter()
            .filter(|s| s.stream_type == StreamKind::Image)
            .count();

        let duration_label = probe
            .duration_seconds
            .map(format_duration_friendly)
            .unwrap_or_else(|| "—".to_string());
        let bitrate_label = probe
            .overall_bitrate_kbps
            .map(format_bitrate_human)
            .unwrap_or_else(|| "—".to_string());

        div()
            .p_6()
            .bg(rgb(BG_CARD))
            .border_1()
            .border_color(rgb(BORDER_SUBTLE))
            .rounded_xl()
            .flex()
            .flex_col()
            .gap_5()
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_start()
                    .justify_between()
                    .gap_4()
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(
                                div()
                                    .text_xl()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(rgb(TEXT_WHITE))
                                    .child(probe.file_name.clone()),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(rgb(TEXT_PRIMARY))
                                    .child(probe.human_summary()),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(TEXT_DIM))
                                    .child(probe.file_path.clone()),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .gap_2()
                            .when(video_count > 0, |this| {
                                this.child(pill(&pluralize(video_count, "Video")))
                            })
                            .when(audio_count > 0, |this| {
                                this.child(pill(&pluralize(audio_count, "Audio")))
                            })
                            .when(sub_count > 0, |this| {
                                this.child(pill(&pluralize(sub_count, "Subtitle")))
                            })
                            .when(img_count > 0, |this| {
                                this.child(pill(&pluralize(img_count, "Image")))
                            }),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap_3()
                    .child(stat_tile("FORMAT", &probe.short_format()))
                    .child(stat_tile("FILE SIZE", &format_bytes_short(probe.file_size)))
                    .child(stat_tile("DURATION", &duration_label))
                    .child(stat_tile("BITRATE", &bitrate_label))
                    .child(stat_tile("TRACKS", &probe.streams.len().to_string())),
            )
    }

    fn render_tabs_bar(&self, probe: &MediaProbe, cx: &mut Context<Self>) -> impl IntoElement {
        let current = self.active_tab;
        let streams_label = format!("Streams ({})", probe.streams.len());
        let meta_label = format!("Metadata ({})", probe.container_tags.len());

        div()
            .flex()
            .flex_row()
            .child(
                div()
                    .p_1()
                    .bg(rgb(BG_SUBTLE))
                    .border_1()
                    .border_color(rgb(BORDER_SUBTLE))
                    .rounded_lg()
                    .flex()
                    .flex_row()
                    .gap_1()
                    .child(tab_button(
                        "tab-overview",
                        "Overview",
                        current == ActiveTab::Overview,
                        cx.listener(|this, _, _, cx| {
                            this.active_tab = ActiveTab::Overview;
                            cx.notify();
                        }),
                    ))
                    .child(tab_button(
                        "tab-streams",
                        &streams_label,
                        current == ActiveTab::Streams,
                        cx.listener(|this, _, _, cx| {
                            this.active_tab = ActiveTab::Streams;
                            cx.notify();
                        }),
                    ))
                    .child(tab_button(
                        "tab-metadata",
                        &meta_label,
                        current == ActiveTab::Metadata,
                        cx.listener(|this, _, _, cx| {
                            this.active_tab = ActiveTab::Metadata;
                            cx.notify();
                        }),
                    ))
                    .child(tab_button(
                        "tab-json",
                        "Raw JSON",
                        current == ActiveTab::Json,
                        cx.listener(|this, _, _, cx| {
                            this.active_tab = ActiveTab::Json;
                            cx.notify();
                        }),
                    )),
            )
    }

    fn render_overview_tab(&self, probe: &MediaProbe) -> AnyElement {
        let duration_text = match (probe.duration_seconds, &probe.duration_human) {
            (Some(secs), Some(ts)) => format!("{} ({})", format_duration_friendly(secs), ts),
            _ => "Not applicable".to_string(),
        };
        let bitrate_text = probe
            .overall_bitrate_kbps
            .map(format_bitrate_human)
            .unwrap_or_else(|| "Not applicable".to_string());

        div()
            .flex()
            .flex_col()
            .gap_4()
            .child(
                section_card("File & Container Details").child(
                    div()
                        .flex()
                        .flex_col()
                        .child(key_val_row("Format", &probe.format_long_name, true))
                        .child(key_val_row("Technical ID", &probe.format_name, true))
                        .child(key_val_row("File Size", &probe.file_size_human, true))
                        .child(key_val_row("Duration", &duration_text, true))
                        .child(key_val_row("Overall Bitrate", &bitrate_text, false)),
                ),
            )
            .child(
                section_card("Tracks & Streams").child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .children(probe.streams.iter().map(|s| {
                            div()
                                .px_4()
                                .py_3()
                                .bg(rgb(BG_SUBTLE))
                                .border_1()
                                .border_color(rgb(BORDER_SUBTLE))
                                .rounded_lg()
                                .flex()
                                .flex_row()
                                .items_center()
                                .justify_between()
                                .gap_4()
                                .child(
                                    div()
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .gap_3()
                                        .child(pill(&format!("#{}", s.index + 1)))
                                        .child(pill(s.stream_type.as_str()))
                                        .child(
                                            div()
                                                .text_sm()
                                                .font_weight(FontWeight::SEMIBOLD)
                                                .text_color(rgb(TEXT_WHITE))
                                                .child(s.codec_long_name.clone()),
                                        ),
                                )
                                .child(
                                    div()
                                        .text_sm()
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(rgb(TEXT_WHITE))
                                        .child(s.human_summary()),
                                )
                        })),
                ),
            )
            .into_any_element()
    }

    fn render_streams_tab(&self, probe: &MediaProbe) -> AnyElement {
        div()
            .flex()
            .flex_col()
            .gap_4()
            .children(probe.streams.iter().map(|stream| {
                let total = stream.properties.len();
                let summary = stream.human_summary();
                div()
                    .p_5()
                    .bg(rgb(BG_CARD))
                    .border_1()
                    .border_color(rgb(BORDER_SUBTLE))
                    .rounded_xl()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap_2()
                                    .child(pill(&format!(
                                        "Track #{} • {}",
                                        stream.index + 1,
                                        stream.stream_type.as_str()
                                    )))
                                    .child(
                                        div()
                                            .ml_1()
                                            .text_base()
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .text_color(rgb(TEXT_WHITE))
                                            .child(format!(
                                                "{} ({})",
                                                stream.codec_long_name, stream.codec_name
                                            )),
                                    ),
                            )
                            .when(!summary.is_empty(), |this| {
                                this.child(
                                    div()
                                        .text_sm()
                                        .text_color(rgb(TEXT_MUTED))
                                        .child(summary),
                                )
                            }),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .px_4()
                            .py_1()
                            .bg(rgb(BG_SUBTLE))
                            .border_1()
                            .border_color(rgb(BORDER_SUBTLE))
                            .rounded_lg()
                            .children(stream.properties.iter().enumerate().map(|(i, (k, v))| {
                                key_val_row(&humanize_key(k), v, i + 1 < total)
                            })),
                    )
            }))
            .into_any_element()
    }

    fn render_metadata_tab(&self, probe: &MediaProbe) -> AnyElement {
        let total = probe.container_tags.len();
        section_card("Tags & Metadata")
            .child(if probe.container_tags.is_empty() {
                div()
                    .py_4()
                    .text_sm()
                    .text_color(rgb(TEXT_MUTED))
                    .child("This file does not contain embedded metadata tags.")
                    .into_any_element()
            } else {
                div()
                    .flex()
                    .flex_col()
                    .children(probe.container_tags.iter().enumerate().map(|(i, (k, v))| {
                        key_val_row(&humanize_key(k), v, i + 1 < total)
                    }))
                    .into_any_element()
            })
            .into_any_element()
    }

    fn render_json_tab(&self, probe: &MediaProbe) -> AnyElement {
        let json_str = serde_json::to_string_pretty(probe).unwrap_or_else(|_| "{}".to_string());

        section_card("JSON Output")
            .child(
                div()
                    .p_4()
                    .bg(rgb(BG_CODE))
                    .border_1()
                    .border_color(rgb(BORDER_SUBTLE))
                    .rounded_lg()
                    .text_xs()
                    .text_color(rgb(TEXT_PRIMARY))
                    .child(json_str),
            )
            .into_any_element()
    }
}

fn pluralize(count: usize, noun: &str) -> String {
    if count == 1 || noun == "Audio" {
        format!("{} {}", count, noun)
    } else {
        format!("{} {}s", count, noun)
    }
}

pub fn humanize_key(key: &str) -> String {
    match key {
        "track_id" => return "Track ID".to_string(),
        "track_number" => return "Track Number".to_string(),
        "sample_rate" => return "Sample Rate".to_string(),
        "frame_rate" => return "Frame Rate".to_string(),
        "aspect_ratio" => return "Aspect Ratio".to_string(),
        "sample_count" => return "Frames / Samples".to_string(),
        "timescale" => return "Timescale".to_string(),
        "bits_per_sample" | "bit_depth" => return "Bit Depth".to_string(),
        "bits_per_coded_sample" => return "Coded Bit Depth".to_string(),
        "time_base" => return "Time Base".to_string(),
        "display_resolution" => return "Display Size".to_string(),
        "default_flag" => return "Default Track".to_string(),
        "forced_flag" => return "Forced Track".to_string(),
        "major_brand" => return "Container Brand".to_string(),
        "minor_version" => return "Format Version".to_string(),
        "compatible_brands" => return "Compatible Brands".to_string(),
        "muxing_app" => return "Muxer".to_string(),
        "writing_app" => return "Encoder / App".to_string(),
        "date_utc" => return "Created (UTC)".to_string(),
        _ => {}
    }

    // Convert snake_case or PascalCase/camelCase into Title Case words
    let mut out = String::with_capacity(key.len() + 4);
    let mut new_word = true;
    for ch in key.chars() {
        if ch == '_' || ch == '-' {
            if !out.ends_with(' ') && !out.is_empty() {
                out.push(' ');
            }
            new_word = true;
        } else if ch.is_uppercase() && !new_word {
            out.push(' ');
            out.push(ch);
            new_word = false;
        } else if new_word {
            for upper in ch.to_uppercase() {
                out.push(upper);
            }
            new_word = false;
        } else {
            out.push(ch);
        }
    }
    out
}

fn section_card(title: &str) -> gpui::Div {
    div()
        .p_5()
        .bg(rgb(BG_CARD))
        .border_1()
        .border_color(rgb(BORDER_SUBTLE))
        .rounded_xl()
        .flex()
        .flex_col()
        .gap_3()
        .child(
            div()
                .text_sm()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(rgb(TEXT_WHITE))
                .child(title.to_string()),
        )
}

fn stat_tile(label: &str, value: &str) -> impl IntoElement {
    div()
        .flex_1()
        .p_3()
        .bg(rgb(BG_SUBTLE))
        .border_1()
        .border_color(rgb(BORDER_SUBTLE))
        .rounded_lg()
        .flex()
        .flex_col()
        .gap_1()
        .child(
            div()
                .text_xs()
                .font_weight(FontWeight::MEDIUM)
                .text_color(rgb(TEXT_DIM))
                .child(label.to_string()),
        )
        .child(
            div()
                .text_sm()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(rgb(TEXT_WHITE))
                .child(value.to_string()),
        )
}

fn key_val_row(key: &str, value: &str, border_bottom: bool) -> impl IntoElement {
    let row = div()
        .flex()
        .flex_row()
        .justify_between()
        .items_center()
        .gap_4()
        .py(px(10.0))
        .px_2()
        .hover(|s| s.bg(rgb(BG_HOVER)))
        .child(
            div()
                .text_sm()
                .text_color(rgb(TEXT_MUTED))
                .child(key.to_string()),
        )
        .child(
            div()
                .text_sm()
                .font_weight(FontWeight::MEDIUM)
                .text_color(rgb(TEXT_WHITE))
                .child(value.to_string()),
        );

    if border_bottom {
        row.border_b_1().border_color(rgb(BORDER_SUBTLE))
    } else {
        row
    }
}

fn pill(text: &str) -> impl IntoElement {
    div()
        .px(px(10.0))
        .py(px(3.0))
        .rounded_full()
        .bg(rgb(BG_SUBTLE))
        .border_1()
        .border_color(rgb(BORDER_STRONG))
        .text_xs()
        .font_weight(FontWeight::MEDIUM)
        .text_color(rgb(TEXT_WHITE))
        .child(text.to_string())
}

fn tab_button(
    id_str: &'static str,
    label: &str,
    is_active: bool,
    on_click_handler: impl Fn(&gpui::ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    let (bg_c, text_c) = if is_active {
        (BG_HOVER, TEXT_WHITE)
    } else {
        (BG_SUBTLE, TEXT_MUTED)
    };

    div()
        .px_4()
        .py(px(6.0))
        .bg(rgb(bg_c))
        .text_color(rgb(text_c))
        .rounded_md()
        .cursor_pointer()
        .hover(|s| s.text_color(rgb(TEXT_WHITE)))
        .text_sm()
        .font_weight(FontWeight::MEDIUM)
        .id(id_str)
        .child(label.to_string())
        .on_click(on_click_handler)
}

fn file_url_to_path(url: &str) -> Option<PathBuf> {
    let raw = url.strip_prefix("file://")?;
    let mut bytes_out = Vec::with_capacity(raw.len());
    let bytes = raw.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(hex) = std::str::from_utf8(&bytes[i + 1..i + 3]) {
                if let Ok(b) = u8::from_str_radix(hex, 16) {
                    bytes_out.push(b);
                    i += 3;
                    continue;
                }
            }
        }
        bytes_out.push(bytes[i]);
        i += 1;
    }
    Some(PathBuf::from(String::from_utf8_lossy(&bytes_out).into_owned()))
}

fn main() {
    let initial_path: Rc<RefCell<Option<PathBuf>>> =
        Rc::new(RefCell::new(std::env::args().nth(1).map(PathBuf::from)));
    let open_target: Rc<RefCell<Option<(Entity<AppState>, AsyncApp)>>> =
        Rc::new(RefCell::new(None));

    let pending_open = initial_path.clone();
    let open_target_cb = open_target.clone();

    let app = Application::new();
    app.on_open_urls(move |urls| {
        if let Some(path) = urls.first().and_then(|u| file_url_to_path(u)) {
            if let Some((ref view, ref mut async_cx)) = *open_target_cb.borrow_mut() {
                let _ = async_cx.update_entity(view, |state, cx| {
                    state.load_file(&path);
                    cx.notify();
                });
            } else {
                *pending_open.borrow_mut() = Some(path);
            }
        }
    });

    app.run(move |cx: &mut App| {
        let start_file = initial_path.borrow_mut().take();
        let bounds = Bounds::centered(None, size(px(1000.0), px(740.0)), cx);
        let window = cx
            .open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    titlebar: Some(gpui::TitlebarOptions {
                        title: Some(APP_NAME.into()),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                |_, cx| cx.new(|_| AppState::new(start_file)),
            )
            .unwrap();

        if let Ok(view) = window.entity(cx) {
            *open_target.borrow_mut() = Some((view, cx.to_async()));
        }

        cx.activate(true);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_humanize_key() {
        assert_eq!(humanize_key("sample_rate"), "Sample Rate");
        assert_eq!(humanize_key("frame_rate"), "Frame Rate");
        assert_eq!(humanize_key("TrackTitle"), "Track Title");
        assert_eq!(humanize_key("custom_field"), "Custom Field");
    }

    #[test]
    fn test_file_url_to_path() {
        assert_eq!(
            file_url_to_path("file:///Users/test/My%20Video.mp4"),
            Some(PathBuf::from("/Users/test/My Video.mp4"))
        );
    }
}
