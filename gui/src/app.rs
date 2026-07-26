use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;

use wrappe::{PackConfig, PackProgress, PackStage};

#[derive(Debug, Clone, PartialEq)]
enum Tab {
    Simple,
    Basic,
    Advanced,
}

#[derive(Debug, Clone, PartialEq)]
enum UnpackTarget {
    Temp,
    Local,
    Cwd,
}

impl UnpackTarget {
    fn as_str(&self) -> &str {
        match self {
            UnpackTarget::Temp => "temp",
            UnpackTarget::Local => "local",
            UnpackTarget::Cwd => "cwd",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Versioning {
    SideBySide,
    Replace,
    None,
}

impl Versioning {
    fn as_str(&self) -> &str {
        match self {
            Versioning::SideBySide => "sidebyside",
            Versioning::Replace => "replace",
            Versioning::None => "none",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Verification {
    Existence,
    Checksum,
    None,
}

impl Verification {
    fn as_str(&self) -> &str {
        match self {
            Verification::Existence => "existence",
            Verification::Checksum => "checksum",
            Verification::None => "none",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum ShowInfo {
    Title,
    Verbose,
    None,
}

impl ShowInfo {
    fn as_str(&self) -> &str {
        match self {
            ShowInfo::Title => "title",
            ShowInfo::Verbose => "verbose",
            ShowInfo::None => "none",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum ConsoleMode {
    Auto,
    Always,
    Never,
    Attach,
}

impl ConsoleMode {
    fn as_str(&self) -> &str {
        match self {
            ConsoleMode::Auto => "auto",
            ConsoleMode::Always => "always",
            ConsoleMode::Never => "never",
            ConsoleMode::Attach => "attach",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum CurrentDir {
    Inherit,
    Unpack,
    Runner,
    Command,
}

impl CurrentDir {
    fn as_str(&self) -> &str {
        match self {
            CurrentDir::Inherit => "inherit",
            CurrentDir::Unpack => "unpack",
            CurrentDir::Runner => "runner",
            CurrentDir::Command => "command",
        }
    }
}

pub struct WrappeApp {
    // Input/Output paths
    input_path: String,
    main_exe: String,
    command_path: String,
    output_path: String,
    output_folder: String,
    output_name: String,

    // Options
    compression: u32,
    runner: String,
    runner_index: usize,
    available_runners: Vec<String>,
    unpack_target: UnpackTarget,
    versioning: Versioning,
    verification: Verification,
    version_string: String,
    show_information: ShowInfo,
    console: ConsoleMode,
    current_dir: CurrentDir,
    env_vars: String,
    icon_path: String,
    cleanup: bool,
    once: bool,
    build_dictionary: bool,
    exclude_patterns: String,

    // Packing state
    packing: bool,
    progress: f32,
    progress_total: u64,
    progress_current: u64,
    progress_message: String,
    result_message: Option<String>,
    result_error: bool,
    pack_thread: Option<thread::JoinHandle<()>>,
    progress_receiver: Option<mpsc::Receiver<PackProgress>>,

    // UI state
    selected_tab: Tab,
    dark_mode: bool,
    dragging: bool,
    drag_offset: Option<egui::Pos2>,
}

impl Default for WrappeApp {
    fn default() -> Self {
        let runners: Vec<String> = wrappe::get_available_runners()
            .iter()
            .map(|s| s.to_string())
            .collect();

        WrappeApp {
            input_path: String::new(),
            main_exe: String::new(),
            command_path: String::new(),
            output_path: String::new(),
            output_folder: String::new(),
            output_name: String::new(),
            compression: 8,
            runner: if runners.is_empty() {
                String::new()
            } else {
                runners[0].clone()
            },
            runner_index: 0,
            available_runners: runners,
            unpack_target: UnpackTarget::Temp,
            versioning: Versioning::SideBySide,
            verification: Verification::Existence,
            version_string: String::new(),
            show_information: ShowInfo::Title,
            console: ConsoleMode::Auto,
            current_dir: CurrentDir::Inherit,
            env_vars: String::new(),
            icon_path: String::new(),
            cleanup: false,
            once: false,
            build_dictionary: false,
            exclude_patterns: String::new(),
            packing: false,
            progress: 0.0,
            progress_total: 0,
            progress_current: 0,
            progress_message: String::new(),
            result_message: None,
            result_error: false,
            pack_thread: None,
            progress_receiver: None,
            selected_tab: Tab::Simple,
            dark_mode: true,
            dragging: false,
            drag_offset: None,
        }
    }
}

impl eframe::App for WrappeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check for progress updates from packing thread
        let mut should_clear_receiver = false;
        if let Some(ref receiver) = self.progress_receiver {
            while let Ok(progress) = receiver.try_recv() {
                let msg = progress.message.clone();
                let stage = progress.stage;
                let is_err = progress.is_error;

                self.progress_message = msg.clone();
                self.progress_current = progress.current;
                self.progress_total = progress.total;
                if progress.total > 0 {
                    self.progress = progress.current as f32 / progress.total as f32;
                }

                if stage == PackStage::Done {
                    self.packing = false;
                    self.result_message = Some(msg.clone());
                    self.result_error = is_err;
                    should_clear_receiver = true;
                } else if is_err {
                    self.result_message = Some(format!("Error: {}", msg));
                    self.result_error = true;
                }
            }
        }
        if should_clear_receiver {
            self.progress_receiver = None;
        }

        // Custom title bar
        let title_bar_height = 36.0;
        egui::TopBottomPanel::top("custom_title_bar")
            .exact_height(title_bar_height)
            .show(ctx, |ui| {
                let bg = if self.dark_mode {
                    egui::Color32::from_rgb(25, 25, 30)
                } else {
                    egui::Color32::from_rgb(230, 230, 235)
                };
                ui.painter().rect_filled(ui.max_rect(), egui::CornerRadius::ZERO, bg);

                // Drag detection
                let resp = ui.interact(
                    ui.max_rect(),
                    ui.next_auto_id(),
                    egui::Sense::click_and_drag(),
                );
                if resp.drag_started() {
                    self.drag_offset = resp.hover_pos();
                }
                if resp.dragged() {
                    if let Some(start) = self.drag_offset {
                        let delta = resp.hover_pos().unwrap() - start;
                        if let Some(pos) = ctx.input(|i| i.viewport().inner_rect) {
                            ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(
                                egui::Pos2::new(pos.min.x + delta.x, pos.min.y + delta.y),
                            ));
                        }
                    }
                }
                if resp.drag_stopped() {
                    self.drag_offset = None;
                }

                ui.horizontal(|ui| {
                    ui.add_space(12.0);
                    let text_color = if self.dark_mode {
                        egui::Color32::from_rgb(200, 200, 210)
                    } else {
                        egui::Color32::from_rgb(40, 40, 50)
                    };
                    ui.label(
                        egui::RichText::new("wrappe GUI")
                            .size(14.0)
                            .color(text_color),
                    );
                    ui.add_space(8.0);
                    ui.label(
                        egui::RichText::new("pack your app into one file")
                            .size(11.0)
                            .color(if self.dark_mode {
                                egui::Color32::from_rgb(120, 120, 130)
                            } else {
                                egui::Color32::from_rgb(140, 140, 150)
                            }),
                    );

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Close button
                        let close_btn = egui::Button::new(
                            egui::RichText::new("\u{2715}").size(14.0),
                        )
                        .fill(egui::Color32::TRANSPARENT)
                        .min_size(egui::vec2(46.0, title_bar_height));
                        if ui.add(close_btn).clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }

                        // Minimize button
                        let min_btn = egui::Button::new(
                            egui::RichText::new("\u{2500}").size(14.0),
                        )
                        .fill(egui::Color32::TRANSPARENT)
                        .min_size(egui::vec2(46.0, title_bar_height));
                        if ui.add(min_btn).clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                        }

                        // Theme toggle
                        if ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new(if self.dark_mode {
                                        "\u{2600}"
                                    } else {
                                        "\u{1F319}"
                                    })
                                    .size(14.0),
                                )
                                .fill(egui::Color32::TRANSPARENT)
                                .min_size(egui::vec2(40.0, title_bar_height)),
                            )
                            .clicked()
                        {
                            self.dark_mode = !self.dark_mode;
                            ctx.set_visuals(if self.dark_mode {
                                egui::Visuals::dark()
                            } else {
                                egui::Visuals::light()
                            });
                        }
                    });
                });
            });

        // Bottom panel - progress and results
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            if self.packing {
                ui.add(
                    egui::ProgressBar::new(self.progress)
                        .text(&self.progress_message)
                        .animate(true),
                );
            }

            if let Some(ref msg) = self.result_message {
                if self.result_error {
                    ui.colored_label(egui::Color32::RED, msg);
                } else {
                    ui.colored_label(egui::Color32::GREEN, msg);
                }
                if ui.button("Clear").clicked() {
                    self.result_message = None;
                    self.result_error = false;
                }
            }
        });

        // Central panel
        egui::CentralPanel::default().show(ctx, |ui| {
            // Tab bar
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.selected_tab, Tab::Simple, "Simple");
                ui.selectable_value(&mut self.selected_tab, Tab::Basic, "Basic");
                ui.selectable_value(&mut self.selected_tab, Tab::Advanced, "Advanced");
            });
            ui.separator();

            match self.selected_tab {
                Tab::Simple => self.show_simple_tab(ui),
                Tab::Basic => self.show_basic_tab(ui),
                Tab::Advanced => self.show_advanced_tab(ui),
            }

            ui.separator();

            // Pack button (hidden in Simple mode which has its own GO button)
            if self.selected_tab != Tab::Simple {
                ui.add_enabled_ui(!self.packing && !self.input_path.is_empty(), |ui| {
                    let pack_btn = egui::Button::new(
                        egui::RichText::new("Pack!")
                            .size(24.0)
                            .color(egui::Color32::WHITE),
                    )
                    .fill(egui::Color32::from_rgb(0, 150, 50))
                    .min_size(egui::vec2(200.0, 50.0));

                    if ui.add(pack_btn).clicked() {
                        self.start_packing();
                    }
                });
            }

            if self.packing {
                ui.label("Packing in progress...");
            }
        });

        if self.packing {
            ctx.request_repaint();
        }
    }
}

impl WrappeApp {
    fn show_simple_tab(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);

            // Title
            ui.label(
                egui::RichText::new("Super Simple Mode")
                    .size(28.0)
                    .strong(),
            );
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new("Just pick a folder, choose the main exe, save it. Done.")
                    .size(13.0),
            );
            ui.add_space(24.0);

            // Card
            let card = egui::Frame::new()
                .fill(if self.dark_mode {
                    egui::Color32::from_rgb(35, 35, 42)
                } else {
                    egui::Color32::from_rgb(248, 248, 250)
                })
                .stroke(egui::Stroke::new(
                    1.5_f32,
                    if self.dark_mode {
                        egui::Color32::from_rgb(60, 60, 70)
                    } else {
                        egui::Color32::from_rgb(200, 200, 210)
                    },
                ))
                .corner_radius(12)
                .inner_margin(egui::Margin::symmetric(32, 28));

            card.show(ui, |ui| {
                ui.set_width(520.0);

                // Input folder
                self.field_label(ui, "\u{1F4C1}  Input Folder");
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut self.input_path)
                            .hint_text("Select the folder containing your app...")
                            .desired_width(400.0),
                    );
                    if ui.button("Browse").clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_folder() {
                            self.input_path = path.display().to_string();
                            if self.output_name.is_empty() {
                                self.output_name = path
                                    .file_name()
                                    .map(|n| n.to_string_lossy().to_string())
                                    .unwrap_or_default();
                            }
                        }
                    }
                });
                ui.add_space(18.0);

                // Main executable (NEW)
                self.field_label(ui, "\u{1F4BE}  Main Executable");
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut self.main_exe)
                            .hint_text("e.g. myapp.exe (relative to input folder)")
                            .desired_width(400.0),
                    );
                    if ui.button("Browse").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("Executables", &["exe", ""])
                            .pick_file()
                        {
                            self.main_exe = path
                                .file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_default();
                        }
                    }
                });
                ui.add_space(18.0);

                // Output folder
                self.field_label(ui, "\u{1F4C2}  Output Folder");
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut self.output_folder)
                            .hint_text("Where to save the packed file...")
                            .desired_width(400.0),
                    );
                    if ui.button("Browse").clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_folder() {
                            self.output_folder = path.display().to_string();
                        }
                    }
                });
                ui.add_space(18.0);

                // Output filename
                self.field_label(ui, "\u{1F4DD}  Output Filename");
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut self.output_name)
                            .hint_text("e.g. MyApp")
                            .desired_width(300.0),
                    );
                    ui.label(".exe");
                });

                // Preview
                if !self.output_folder.is_empty() && !self.output_name.is_empty() {
                    ui.add_space(10.0);
                    let preview = format!(
                        "{}/{}.exe",
                        self.output_folder.trim_end_matches('/').trim_end_matches('\\'),
                        self.output_name.trim(),
                    );
                    ui.label(
                        egui::RichText::new(format!("Saved to: {}", preview))
                            .size(12.0),
                    );
                    self.output_path = preview;
                }
            });

            ui.add_space(24.0);

            // Big GO button
            let ready = !self.input_path.is_empty()
                && !self.output_folder.is_empty()
                && !self.output_name.is_empty();

            ui.add_enabled_ui(!self.packing && ready, |ui| {
                let btn = egui::Button::new(
                    egui::RichText::new("GO!")
                        .size(42.0)
                        .color(egui::Color32::WHITE),
                )
                .fill(egui::Color32::from_rgb(46, 204, 113))
                .min_size(egui::vec2(320.0, 85.0))
                .corner_radius(16);

                if ui.add(btn).clicked() {
                    self.command_path = self.main_exe.clone();
                    self.runner_index = 0;
                    self.runner = self.available_runners.first().cloned().unwrap_or_default();
                    self.compression = 8;
                    self.unpack_target = UnpackTarget::Temp;
                    self.versioning = Versioning::SideBySide;
                    self.verification = Verification::Existence;
                    self.console = ConsoleMode::Never;
                    self.current_dir = CurrentDir::Inherit;
                    self.cleanup = false;
                    self.once = false;
                    self.build_dictionary = false;
                    self.version_string.clear();
                    self.env_vars.clear();
                    self.icon_path.clear();
                    self.exclude_patterns.clear();
                    self.start_packing();
                }
            });

            if !ready && !self.packing {
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new("Fill in all fields above to start")
                        .size(13.0),
                );
            }
        });
    }

    fn field_label(&self, ui: &mut egui::Ui, text: &str) {
        ui.label(
            egui::RichText::new(text)
                .size(14.0)
                .strong(),
        );
    }

    fn show_basic_tab(&mut self, ui: &mut egui::Ui) {
        egui::Grid::new("basic_grid")
            .num_columns(3)
            .spacing([10.0, 8.0])
            .striped(true)
            .show(ui, |ui| {
                ui.label("Input:");
                ui.add(
                    egui::TextEdit::singleline(&mut self.input_path)
                        .hint_text("Directory or executable to pack...")
                        .desired_width(400.0),
                );
                if ui.button("Browse...").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        self.input_path = path.display().to_string();
                    }
                }
                ui.end_row();

                ui.label("Command:");
                ui.add(
                    egui::TextEdit::singleline(&mut self.command_path)
                        .hint_text("Executable relative to input (for directories)...")
                        .desired_width(400.0),
                );
                if ui.button("Browse...").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_file() {
                        self.command_path = path.display().to_string();
                    }
                }
                ui.end_row();

                ui.label("Output:");
                ui.add(
                    egui::TextEdit::singleline(&mut self.output_path)
                        .hint_text("Output executable path...")
                        .desired_width(400.0),
                );
                if ui.button("Save as...").clicked() {
                    if let Some(path) = rfd::FileDialog::new().save_file() {
                        self.output_path = path.display().to_string();
                    }
                }
                ui.end_row();

                ui.label("Compression:");
                ui.add(egui::Slider::new(&mut self.compression, 0..=22).text("level"));
                ui.label(format!("Level: {}", self.compression));
                ui.end_row();

                ui.label("Runner:");
                egui::ComboBox::from_id_salt("runner_combo")
                    .selected_text(&self.runner)
                    .show_ui(ui, |ui| {
                        for (i, r) in self.available_runners.iter().enumerate() {
                            if ui.selectable_value(&mut self.runner_index, i, r).clicked() {
                                self.runner = r.clone();
                            }
                        }
                    });
                ui.end_row();
            });
    }

    fn show_advanced_tab(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("advanced_grid")
                .num_columns(2)
                .spacing([10.0, 6.0])
                .striped(true)
                .show(ui, |ui| {
                    ui.label("Unpack Target:");
                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut self.unpack_target, UnpackTarget::Temp, "Temp");
                        ui.selectable_value(&mut self.unpack_target, UnpackTarget::Local, "Local");
                        ui.selectable_value(&mut self.unpack_target, UnpackTarget::Cwd, "CWD");
                    });
                    ui.end_row();

                    ui.label("Versioning:");
                    ui.horizontal(|ui| {
                        ui.selectable_value(
                            &mut self.versioning,
                            Versioning::SideBySide,
                            "Side-by-side",
                        );
                        ui.selectable_value(&mut self.versioning, Versioning::Replace, "Replace");
                        ui.selectable_value(&mut self.versioning, Versioning::None, "None");
                    });
                    ui.end_row();

                    ui.label("Verification:");
                    ui.horizontal(|ui| {
                        ui.selectable_value(
                            &mut self.verification,
                            Verification::Existence,
                            "Existence",
                        );
                        ui.selectable_value(
                            &mut self.verification,
                            Verification::Checksum,
                            "Checksum",
                        );
                        ui.selectable_value(&mut self.verification, Verification::None, "None");
                    });
                    ui.end_row();

                    ui.label("Version String:");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.version_string)
                            .hint_text("Custom version (max 16 chars, optional)...")
                            .desired_width(300.0),
                    );
                    ui.end_row();

                    ui.label("Show Info:");
                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut self.show_information, ShowInfo::Title, "Title");
                        ui.selectable_value(
                            &mut self.show_information,
                            ShowInfo::Verbose,
                            "Verbose",
                        );
                        ui.selectable_value(&mut self.show_information, ShowInfo::None, "None");
                    });
                    ui.end_row();

                    ui.label("Console:");
                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut self.console, ConsoleMode::Auto, "Auto");
                        ui.selectable_value(&mut self.console, ConsoleMode::Always, "Always");
                        ui.selectable_value(&mut self.console, ConsoleMode::Never, "Never");
                        ui.selectable_value(&mut self.console, ConsoleMode::Attach, "Attach");
                    });
                    ui.end_row();

                    ui.label("Working Dir:");
                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut self.current_dir, CurrentDir::Inherit, "Inherit");
                        ui.selectable_value(&mut self.current_dir, CurrentDir::Unpack, "Unpack");
                        ui.selectable_value(&mut self.current_dir, CurrentDir::Runner, "Runner");
                        ui.selectable_value(
                            &mut self.current_dir,
                            CurrentDir::Command,
                            "Command",
                        );
                    });
                    ui.end_row();

                    ui.label("Env Vars:");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.env_vars)
                            .hint_text("KEY1=value1 KEY2=value2 ...")
                            .desired_width(400.0),
                    );
                    ui.end_row();

                    ui.label("Icon:");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.icon_path)
                            .hint_text("Path to .ico file (Windows only)...")
                            .desired_width(300.0),
                    );
                    if ui.button("Browse...").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("Icons", &["ico", "png", "jpg"])
                            .pick_file()
                        {
                            self.icon_path = path.display().to_string();
                        }
                    }
                    ui.end_row();

                    ui.label("Exclude:");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.exclude_patterns)
                            .hint_text("*.log node_modules/** .git/** ...")
                            .desired_width(400.0),
                    );
                    ui.end_row();

                    ui.label("Options:");
                    ui.vertical(|ui| {
                        ui.checkbox(&mut self.cleanup, "Cleanup after exit");
                        ui.checkbox(&mut self.once, "Single instance only");
                        ui.checkbox(&mut self.build_dictionary, "Build compression dictionary");
                    });
                    ui.end_row();
                });
        });
    }

    fn build_config(&self) -> PackConfig {
        let env_vars: Vec<String> = self
            .env_vars
            .split_whitespace()
            .map(|s| s.to_string())
            .filter(|s| s.contains('='))
            .collect();

        let exclude_patterns: Vec<String> = self
            .exclude_patterns
            .split_whitespace()
            .map(|s| s.to_string())
            .filter(|s| !s.is_empty())
            .collect();

        PackConfig {
            runner: self.runner.clone(),
            compression: self.compression,
            unpack_target: self.unpack_target.as_str().to_string(),
            unpack_directory: None,
            versioning: self.versioning.as_str().to_string(),
            verification: self.verification.as_str().to_string(),
            version_string: if self.version_string.is_empty() {
                None
            } else {
                Some(self.version_string.clone())
            },
            show_information: self.show_information.as_str().to_string(),
            console: self.console.as_str().to_string(),
            current_dir: self.current_dir.as_str().to_string(),
            env: env_vars,
            icon: if self.icon_path.is_empty() {
                None
            } else {
                Some(PathBuf::from(&self.icon_path))
            },
            cleanup: self.cleanup,
            once: self.once,
            build_dictionary: self.build_dictionary,
            input: PathBuf::from(&self.input_path),
            command: if self.command_path.is_empty() {
                None
            } else {
                Some(PathBuf::from(&self.command_path))
            },
            output: if self.output_path.is_empty() {
                None
            } else {
                Some(PathBuf::from(&self.output_path))
            },
            arguments: Vec::new(),
            exclude_patterns,
        }
    }

    fn start_packing(&mut self) {
        self.packing = true;
        self.progress = 0.0;
        self.progress_message = "Starting...".to_string();
        self.result_message = None;
        self.result_error = false;

        let config = self.build_config();
        let (sender, receiver) = mpsc::channel();

        self.progress_receiver = Some(receiver);

        let handle = thread::spawn(move || {
            let _ = wrappe::pack(config, move |progress| {
                let _ = sender.send(progress);
            });
        });

        self.pack_thread = Some(handle);
    }
}
