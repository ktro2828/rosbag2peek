use crossbeam_channel as channel;
use egui::{RichText, collapsing_header::CollapsingState};
use rfd::FileDialog;
use rospeek_core::{CdrDecoder, MessageSchema, RawMessage, Topic, ns_to_iso, try_decode_binary};
use std::{f32, path::PathBuf, sync::Arc};

use crate::backend::Backend;

#[derive(Debug)]
enum Command {
    LoadTopic { name: String },
    PageNext,
}

#[derive(Debug)]
enum Event {
    Topics(Vec<Topic>),
    Page {
        topic: String,
        msgs: Vec<RawMessage>,
    },
    Error(String),
}

#[derive(Debug, PartialEq, Eq)]
enum ViewMode {
    Auto,
    Bytes,
    Json,
}

pub struct RospeekApp<B: Backend + 'static> {
    backend: Option<Arc<B>>,
    bag_path: Option<PathBuf>,
    topics: Vec<Topic>,
    topic_filter: String,
    current_schema: Option<MessageSchema>,
    current_topic: Option<String>,
    current_page: usize,
    page: Vec<RawMessage>,
    view_mode: ViewMode,
    // backend workers
    tx: channel::Sender<Command>,
    rx: channel::Receiver<Event>,
}

impl<B: Backend + 'static> RospeekApp<B> {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let (txc, rxc) = channel::unbounded::<Command>();
        let (txe, rxe) = channel::unbounded::<Event>();

        // worker thread starts empty; will be (re)created when a bag is opened
        std::thread::spawn(move || {
            // idle loop; wait for commands until a real backend is provided after open
            while let Ok(_cmd) = rxc.recv() {
                let _ = txe.send(Event::Error("No bag opened".into()));
            }
        });

        Self {
            backend: None,
            bag_path: None,
            topics: Vec::new(),
            topic_filter: String::new(),
            current_schema: None,
            current_topic: None,
            current_page: 0,
            page: Vec::new(),
            view_mode: ViewMode::Auto,
            tx: txc,
            rx: rxe,
        }
    }

    /// Open a ROS 2 bag file.
    fn open(&mut self) {
        if let Some(path) = FileDialog::new()
            .add_filter("ROS 2 bag", &["db3"])
            .pick_file()
        {
            match B::open(&path) {
                Ok(backend) => {
                    let backend = Arc::new(backend);
                    let topics = backend.topics().unwrap_or_default();

                    // Rebuild worker bound to this backend
                    let (txc, rxc) = channel::unbounded::<Command>();
                    let (txe, rxe) = channel::unbounded::<Event>();

                    // let worker bound to this backend
                    let bend = backend.clone();
                    let tmp_topics = topics.clone().into_iter().collect::<Vec<_>>();
                    std::thread::spawn(move || {
                        let _ = txe.send(Event::Topics(tmp_topics));
                        while let Ok(cmd) = rxc.recv() {
                            match cmd {
                                Command::LoadTopic { name } => {
                                    match bend.read_messages(&name, None, 200) {
                                        Ok(msgs) => {
                                            let _ = txe.send(Event::Page { topic: name, msgs });
                                        }
                                        Err(e) => {
                                            let _ = txe.send(Event::Error(e.to_string()));
                                        }
                                    }
                                }
                                Command::PageNext => {
                                    // TODO: keep cursor; request next page
                                }
                            }
                        }
                    });

                    self.backend = Some(backend);
                    self.bag_path = Some(path);
                    self.topics = topics;
                    self.current_schema = None;
                    self.current_topic = None;
                    self.current_page = 0;
                    self.page.clear();
                    self.tx = txc;
                    self.rx = rxe;
                }
                Err(e) => {
                    self.backend = None;
                    self.bag_path = None;
                    self.topics.clear();
                    self.current_schema = None;
                    self.current_topic = None;
                    self.current_page = 0;
                    egui::PopupCloseBehavior::default();
                    eprintln!("Open failed: {e:?}")
                }
            }
        }
    }

    /// Performs UI operations related to topics.
    fn ui_topics(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Topic Filter");
            ui.text_edit_singleline(&mut self.topic_filter);
        });
        ui.separator();

        let filter = self.topic_filter.to_lowercase();
        egui::ScrollArea::vertical().show(ui, |ui| {
            for (i, topic) in self.topics.iter().enumerate() {
                if !filter.is_empty() && !topic.name.to_lowercase().contains(&filter) {
                    continue;
                }
                let select = Some(topic.name.clone()) == self.current_topic;
                if ui
                    .selectable_label(
                        select,
                        to_rich_text(&format!("{} [{}]", topic.name, &topic.type_name)),
                    )
                    .clicked()
                {
                    self.current_schema = MessageSchema::try_from(topic.type_name.as_ref()).ok();
                    self.current_topic = Some(topic.name.clone());
                    self.current_page += i;
                    let _ = self.tx.send(Command::LoadTopic {
                        name: topic.name.clone(),
                    });
                }
            }
        });
    }

    fn ui_center(&mut self, ui: &mut egui::Ui) {
        ui.heading("Message Inspector");
        ui.separator();

        ui.horizontal(|ui| {
            egui::ComboBox::from_label("View Mode")
                .selected_text(match self.view_mode {
                    ViewMode::Auto => "Auto",
                    ViewMode::Bytes => "Bytes",
                    ViewMode::Json => "Json",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.view_mode, ViewMode::Auto, "Auto");
                    ui.selectable_value(&mut self.view_mode, ViewMode::Bytes, "Bytes");
                    ui.selectable_value(&mut self.view_mode, ViewMode::Json, "Json");
                });
        });

        if let (Some(topic), Some(schema)) = (&self.current_topic, &self.current_schema) {
            ui.monospace(to_rich_text(&format!("Topic: {}", topic)).strong());
            ui.add_space(4.0);
            egui::ScrollArea::vertical().show(ui, |ui| {
                let mut decoder = CdrDecoder::from_schema(&schema);
                for (idx, msg) in self.page.iter().enumerate() {
                    let id = ui.make_persistent_id(("msg_row", msg.topic_id, msg.timestamp, idx));
                    let header = CollapsingState::load_with_default_open(ui.ctx(), id, false)
                        .show_header(ui, |ui| {
                            ui.label(format!(
                                "[#{idx}] @{} ({} bytes)",
                                ns_to_iso(msg.timestamp),
                                msg.data.len()
                            ))
                        });

                    // display decoded message if the header is unindented
                    header.body_unindented(|ui| {
                        let mut body = self.display_message(&mut decoder, msg);
                        egui::TextEdit::multiline(&mut body)
                            .code_editor()
                            .interactive(false)
                            .desired_width(f32::INFINITY)
                            .show(ui);
                    });
                }
            });
        } else {
            ui.label("Select a topic on the left.");
        }
    }

    fn display_message<'a>(&self, decoder: &mut CdrDecoder<'a>, msg: &'a RawMessage) -> String {
        match self.view_mode {
            ViewMode::Bytes => dump_bytes(&msg.data, 64),
            _ => self.current_schema.as_ref().map_or_else(
                || "Failed to decode binary: no schema".to_string(),
                |schema| {
                    try_decode_binary(decoder, schema, &msg.data).unwrap_or_else(|e| e.to_string())
                },
            ),
        }
    }

    /// Performs UI operations related to the timeline.
    fn ui_timeline(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("▶").clicked() { /* TODO(ktro2828): Implement timeline playback */ }
            if ui.button("⏸").clicked() { /* TODO(ktro2828): Implement timeline pause */ }
            if ui.button("⏹").clicked() { /* TODO(ktro2828): Implement timeline stop */ }
            if ui.button("⏭").clicked() {
                let _ = self.tx.send(Command::PageNext);
            }
            ui.add_space(8.0);
            ui.label(to_rich_text("Timeline"));
        });
    }
}

impl<B: Backend + 'static> eframe::App for RospeekApp<B> {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        while let Ok(ev) = self.rx.try_recv() {
            match ev {
                Event::Topics(ts) => {
                    self.topics = ts;
                }
                Event::Page { topic, msgs } => {
                    if Some(topic.clone()) == self.current_topic {
                        self.page = msgs;
                    }
                }
                Event::Error(e) => {
                    egui::Window::new("Error").show(ctx, |ui| {
                        ui.label(format!("Error: {}", e));
                    });
                }
            }
        }

        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Open bag...").clicked() {
                    self.open();
                }
                if let Some(p) = &self.bag_path {
                    ui.label(to_rich_text(&p.display().to_string()));
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(to_rich_text("rospeek-gui"));
                });
            })
        });

        egui::SidePanel::left("left")
            .resizable(true)
            .default_width(280.0)
            .show(ctx, |ui| {
                self.ui_topics(ui);
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.ui_center(ui);
        });

        egui::TopBottomPanel::bottom("bottom").show(ctx, |ui| {
            self.ui_timeline(ui);
        });
    }
}

/// Converts a string to rich text with a gray color.
fn to_rich_text(s: &str) -> egui::RichText {
    RichText::new(s).color(egui::Color32::from_gray(150))
}

/// Converts bytes to string representation.
fn dump_bytes(bytes: &[u8], max_line: usize) -> String {
    const CHUNK_SIZE: usize = 16;
    let mut out = String::new();
    for (line_idx, chunk) in bytes.chunks(CHUNK_SIZE).enumerate() {
        if line_idx >= max_line {
            out.push_str("...");
            break;
        }

        // offset
        out.push_str(&format!("{:08x}: ", line_idx * CHUNK_SIZE));

        // hex (push separator per 4-bytes)
        for i in 0..CHUNK_SIZE {
            if i > 0 && i % 4 == 0 {
                out.push_str("| ");
            }

            if let Some(byte) = chunk.get(i) {
                out.push_str(&format!("{:02x} ", byte));
            } else {
                out.push_str("   ");
            }
        }

        // ascii
        out.push_str(" ");
        for b in chunk {
            let c = if b.is_ascii_graphic() {
                *b as char
            } else {
                '.'
            };
            out.push(c);
        }
        out.push('\n');
    }
    out
}
