use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;

use wrappe::{PackConfig, PackProgress, PackStage};

#[derive(Debug, Clone, PartialEq)]
enum Tab {
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
    command_path: String,
    output_path: String,

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
}

impl Default for WrappeApp {
    fn default() -> Self {
        let runners: Vec<String> = wrappe::get_available_runners()
            .iter()
            .map(|s| s.to_string())
            .collect();

        WrappeApp {
            input_path: String::new(),
            command_path: String::new(),
            output_path: String::new(),
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
            selected_tab: Tab::Basic,
        }
    }
}

impl eframe::App for WrappeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check for progress updates from packing thread
        if let Some(ref receiver) = self.progress_receiver {
            while let Ok(progress) = receiver.try_recv() {
                self.progress_message = progress.message.clone();
                self.progress_current = progress.current;
                self.progress_total = progress.total;
                if progress.total > 0 {
                    self.progress = progress.current as f32 / progress.total as f32;
                }

                if progress.stage == PackStage::Done {
                    self.packing = false;
                    self.result_message = Some(progress.message);
                    self.result_error = progress.is_error;
                    self.progress_receiver = None;
                }

                if progress.is_error && progress.stage != PackStage::Done {
                    self.result_message = Some(format!("Error: {}", progress.message));
                    self.result_error = true;
                }
            }
        }

        // Top panel - title
        egui::TopBottomPanel::top("title_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("wrappe GUI");
                ui.separator();
                ui.label("Pack executables into self-contained single binaries");
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

        // Central panel - tabs
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.selected_tab, Tab::Basic, "Basic");
                ui.selectable_value(&mut self.selected_tab, Tab::Advanced, "Advanced");
            });
            ui.separator();

            match self.selected_tab {
                Tab::Basic => self.show_basic_tab(ui),
                Tab::Advanced => self.show_advanced_tab(ui),
            }

            ui.separator();

            // Pack button
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

            if self.packing {
                ui.label("Packing in progress...");
            }
        });

        // Request repaint while packing for progress updates
        if self.packing {
            ctx.request_repaint();
        }
    }
}

impl WrappeApp {
    fn show_basic_tab(&mut self, ui: &mut egui::Ui) {
        egui::Grid::new("basic_grid")
            .num_columns(3)
            .spacing([10.0, 8.0])
            .striped(true)
            .show(ui, |ui| {
                // Input path
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

                // Command path
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

                // Output path
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

                // Compression level
                ui.label("Compression:");
                ui.add(egui::Slider::new(&mut self.compression, 0..=22).text("level"));
                ui.label(format!("Level: {}", self.compression));
                ui.end_row();

                // Runner
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
                    // Unpack target
                    ui.label("Unpack Target:");
                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut self.unpack_target, UnpackTarget::Temp, "Temp");
                        ui.selectable_value(&mut self.unpack_target, UnpackTarget::Local, "Local");
                        ui.selectable_value(&mut self.unpack_target, UnpackTarget::Cwd, "CWD");
                    });
                    ui.end_row();

                    // Versioning
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

                    // Verification
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

                    // Version string
                    ui.label("Version String:");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.version_string)
                            .hint_text("Custom version (max 16 chars, optional)...")
                            .desired_width(300.0),
                    );
                    ui.end_row();

                    // Show information
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

                    // Console
                    ui.label("Console:");
                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut self.console, ConsoleMode::Auto, "Auto");
                        ui.selectable_value(&mut self.console, ConsoleMode::Always, "Always");
                        ui.selectable_value(&mut self.console, ConsoleMode::Never, "Never");
                        ui.selectable_value(&mut self.console, ConsoleMode::Attach, "Attach");
                    });
                    ui.end_row();

                    // Current directory
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

                    // Environment variables
                    ui.label("Env Vars:");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.env_vars)
                            .hint_text("KEY1=value1 KEY2=value2 ...")
                            .desired_width(400.0),
                    );
                    ui.end_row();

                    // Icon path
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

                    // Exclude patterns
                    ui.label("Exclude:");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.exclude_patterns)
                            .hint_text("*.log node_modules/** .git/** ...")
                            .desired_width(400.0),
                    );
                    ui.end_row();

                    // Checkboxes
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
            let _ = wrappe::pack(config, |progress| {
                let _ = sender.send(progress);
            });
        });

        self.pack_thread = Some(handle);
    }
}
