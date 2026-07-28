use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;

use wrappe::{PackConfig, PackProgress, PackStage};

#[derive(Debug, Clone, PartialEq)]
enum Tab { Simple, Basic, Advanced, About }

#[derive(Debug, Clone, PartialEq)]
enum UnpackTarget { Temp, Local, Cwd }
impl UnpackTarget { fn as_str(&self) -> &str { match self { Self::Temp => "temp", Self::Local => "local", Self::Cwd => "cwd" } } }

#[derive(Debug, Clone, PartialEq)]
enum Versioning { SideBySide, Replace, None }
impl Versioning { fn as_str(&self) -> &str { match self { Self::SideBySide => "sidebyside", Self::Replace => "replace", Self::None => "none" } } }

#[derive(Debug, Clone, PartialEq)]
enum Verification { Existence, Checksum, None }
impl Verification { fn as_str(&self) -> &str { match self { Self::Existence => "existence", Self::Checksum => "checksum", Self::None => "none" } } }

#[derive(Debug, Clone, PartialEq)]
enum ShowInfo { Title, Verbose, None }
impl ShowInfo { fn as_str(&self) -> &str { match self { Self::Title => "title", Self::Verbose => "verbose", Self::None => "none" } } }

#[derive(Debug, Clone, PartialEq)]
enum ConsoleMode { Auto, Always, Never, Attach }
impl ConsoleMode { fn as_str(&self) -> &str { match self { Self::Auto => "auto", Self::Always => "always", Self::Never => "never", Self::Attach => "attach" } } }

#[derive(Debug, Clone, PartialEq)]
enum CurrentDir { Inherit, Unpack, Runner, Command }
impl CurrentDir { fn as_str(&self) -> &str { match self { Self::Inherit => "inherit", Self::Unpack => "unpack", Self::Runner => "runner", Self::Command => "command" } } }

#[derive(Debug, Clone, PartialEq)]
enum Lang { En, Zh, Ja, Ko, Ru }
impl Lang {
    fn name(&self) -> &str { match self { Self::En => "English", Self::Zh => "中文", Self::Ja => "日本語", Self::Ko => "한국어", Self::Ru => "Русский" } }
    fn flag(&self) -> &str { match self { Self::En => "EN", Self::Zh => "中", Self::Ja => "日", Self::Ko => "한", Self::Ru => "RU" } }
}

fn tr<'a>(lang: &Lang, en: &'a str, zh: &'a str, ja: &'a str, ko: &'a str, ru: &'a str) -> &'a str {
    match lang { Lang::En => en, Lang::Zh => zh, Lang::Ja => ja, Lang::Ko => ko, Lang::Ru => ru }
}

pub struct WrappeApp {
    input_path: String, main_exe: String, command_path: String,
    output_path: String, output_folder: String, output_name: String,
    compression: u32, runner: String, runner_index: usize, available_runners: Vec<String>,
    unpack_target: UnpackTarget, versioning: Versioning, verification: Verification,
    version_string: String, show_information: ShowInfo, console: ConsoleMode, current_dir: CurrentDir,
    env_vars: String, icon_path: String, cleanup: bool, once: bool, build_dictionary: bool,
    exclude_patterns: String, memory_mode: bool, admin_mode: bool,
    packing: bool, progress: f32, progress_total: u64, progress_current: u64,
    progress_message: String, result_message: Option<String>, result_error: bool,
    pack_thread: Option<thread::JoinHandle<()>>,
    progress_receiver: Option<mpsc::Receiver<PackProgress>>,
    cancel_flag: Option<std::sync::Arc<std::sync::atomic::AtomicBool>>,
    selected_tab: Tab, dark_mode: bool, lang: Lang,
}

impl Default for WrappeApp {
    fn default() -> Self {
        let runners: Vec<String> = wrappe::get_available_runners().iter().map(|s| s.to_string()).collect();
        WrappeApp {
            input_path: String::new(), main_exe: String::new(), command_path: String::new(),
            output_path: String::new(), output_folder: String::new(), output_name: String::new(),
            compression: 8, runner: runners.first().cloned().unwrap_or_default(), runner_index: 0,
            available_runners: runners,
            unpack_target: UnpackTarget::Temp, versioning: Versioning::SideBySide,
            verification: Verification::Existence, version_string: String::new(),
            show_information: ShowInfo::Title, console: ConsoleMode::Auto,
            current_dir: CurrentDir::Inherit, env_vars: String::new(), icon_path: String::new(),
            cleanup: false, once: false, build_dictionary: false, exclude_patterns: String::new(),
            memory_mode: true, admin_mode: false,
            packing: false, progress: 0.0, progress_total: 0, progress_current: 0,
            progress_message: String::new(), result_message: None, result_error: false,
            pack_thread: None, progress_receiver: None, cancel_flag: None,
            selected_tab: Tab::Simple, dark_mode: true, lang: Lang::Zh,
        }
    }
}

impl eframe::App for WrappeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut should_clear_receiver = false;
        if let Some(ref receiver) = self.progress_receiver {
            while let Ok(progress) = receiver.try_recv() {
                let msg = progress.message.clone(); let stage = progress.stage; let is_err = progress.is_error;
                self.progress_message = msg.clone(); self.progress_current = progress.current; self.progress_total = progress.total;
                if progress.total > 0 { self.progress = progress.current as f32 / progress.total as f32; }
                if stage == PackStage::Done {
                    self.packing = false; self.result_message = Some(msg.clone()); self.result_error = is_err; should_clear_receiver = true;
                } else if stage == PackStage::Cancelled {
                    self.packing = false; self.result_message = Some("Cancelled".to_string()); self.result_error = true; should_clear_receiver = true;
                } else if is_err {
                    self.result_message = Some(format!("Error: {}", msg)); self.result_error = true;
                }
            }
        }
        if should_clear_receiver { self.progress_receiver = None; }

        // Title bar
        egui::TopBottomPanel::top("title_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("wrappe GUI");
                ui.add_space(8.0);
                ui.label(egui::RichText::new(tr(&self.lang, "pack your app into one file", "打包你的应用到一个文件", "アプリを1つのファイルに", "앱을 하나의 파일로 패킹", "упакуйте приложение в один файл")).size(11.0).weak());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    egui::ComboBox::from_id_salt("lang_selector").selected_text(self.lang.flag()).width(50.0).show_ui(ui, |ui| {
                        for lang in &[Lang::En, Lang::Zh, Lang::Ja, Lang::Ko, Lang::Ru] {
                            if ui.selectable_label(self.lang == *lang, lang.name()).clicked() { self.lang = lang.clone(); }
                        }
                    });
                    if ui.button(if self.dark_mode { "\u{2600}" } else { "\u{1F319}" }).clicked() {
                        self.dark_mode = !self.dark_mode;
                        ctx.set_visuals(if self.dark_mode { egui::Visuals::dark() } else { egui::Visuals::light() });
                    }
                });
            });
        });

        // Bottom panel
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            if self.packing {
                ui.add(egui::ProgressBar::new(self.progress).text(&self.progress_message).animate(true));
            }
            if let Some(ref msg) = self.result_message {
                if self.result_error { ui.colored_label(egui::Color32::RED, msg); }
                else { ui.colored_label(egui::Color32::GREEN, msg); }
                if !self.result_error && !self.output_folder.is_empty() {
                    if ui.button(tr(&self.lang, "Open Folder", "打开文件夹", "フォルダを開く", "폴더 열기", "Открыть папку")).clicked() { let _ = open::that(&self.output_folder); }
                }
                if ui.button(tr(&self.lang, "Clear", "清除", "クリア", "지우기", "Очистить")).clicked() { self.result_message = None; self.result_error = false; }
            }
        });

        // Central panel
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.selected_tab, Tab::Simple, tr(&self.lang, "Simple", "简单", "簡単", "간단", "Просто"));
                ui.selectable_value(&mut self.selected_tab, Tab::Basic, tr(&self.lang, "Basic", "基础", "基本", "기본", "Базовый"));
                ui.selectable_value(&mut self.selected_tab, Tab::Advanced, tr(&self.lang, "Advanced", "高级", "詳細", "고급", "Продвинутый"));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.selectable_value(&mut self.selected_tab, Tab::About, tr(&self.lang, "About", "关于", "情報", "정보", "О программе"));
                });
            });
            ui.separator();
            match self.selected_tab {
                Tab::Simple => self.show_simple_tab(ui),
                Tab::Basic => self.show_basic_tab(ui),
                Tab::Advanced => self.show_advanced_tab(ui),
                Tab::About => self.show_about_tab(ui),
            }
            ui.separator();
            if self.selected_tab != Tab::Simple {
                ui.add_enabled_ui(!self.packing && !self.input_path.is_empty(), |ui| {
                    if ui.add(egui::Button::new(egui::RichText::new("Pack!").size(24.0).color(egui::Color32::WHITE)).fill(egui::Color32::from_rgb(0, 150, 50)).min_size(egui::vec2(200.0, 50.0))).clicked() { self.start_packing(); }
                });
            }
            if self.packing {
                ui.horizontal(|ui| {
                    ui.label(tr(&self.lang, "Packing in progress...", "打包中...", "パック中...", "패킹 중...", "Упаковка..."));
                    if ui.button("\u{2716} Cancel").clicked() { if let Some(ref flag) = self.cancel_flag { flag.store(true, std::sync::atomic::Ordering::Relaxed); } }
                });
            }
        });
        if self.packing { ctx.request_repaint(); }
    }
}

impl WrappeApp {
    fn show_simple_tab(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            ui.label(egui::RichText::new(tr(&self.lang, "Super Simple Mode", "超级简单模式", "超簡単モード", "초간단 모드", "Супер простой режим")).size(28.0).strong());
            ui.add_space(4.0);
            ui.label(egui::RichText::new(tr(&self.lang, "Just pick a folder, choose the main exe, save it. Done.", "选个文件夹，选主程序，保存。完事。", "フォルダを選んで、exeを選んで、保存するだけ。", "폴더 선택, exe 선택, 저장. 끝.", "Выберите папку, exe, сохраните. Готово.")).size(13.0));
            ui.add_space(24.0);

            let card = egui::Frame::none()
                .fill(if self.dark_mode { egui::Color32::from_rgb(35, 35, 42) } else { egui::Color32::from_rgb(248, 248, 250) })
                .stroke(egui::Stroke::new(1.5_f32, if self.dark_mode { egui::Color32::from_rgb(60, 60, 70) } else { egui::Color32::from_rgb(200, 200, 210) }))
                .rounding(egui::Rounding::same(12.0)).inner_margin(egui::Margin::symmetric(32.0, 28.0));

            card.show(ui, |ui| {
                ui.set_width(520.0);
                // Input folder
                ui.label(egui::RichText::new(format!("\u{1F4C1}  {}", tr(&self.lang, "Input Folder", "输入文件夹", "入力フォルダ", "입력 폴더", "Входная папка"))).size(14.0).strong());
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.add(egui::TextEdit::singleline(&mut self.input_path).hint_text(tr(&self.lang, "Select the folder containing your app...", "选择包含你应用的文件夹...", "アプリが含まれるフォルダを選択...", "앱이 있는 폴더 선택...", "Выберите папку с приложением...")).desired_width(400.0));
                    if ui.button(tr(&self.lang, "Browse", "浏览", "参照", "찾기", "Обзор")).clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_folder() { self.input_path = path.display().to_string(); if self.output_name.is_empty() { self.output_name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default(); } }
                    }
                });
                ui.add_space(18.0);
                // Main executable
                ui.label(egui::RichText::new(format!("\u{1F4BE}  {}", tr(&self.lang, "Main Executable", "主程序", "メイン実行ファイル", "메인 실행 파일", "Главный исполняемый файл"))).size(14.0).strong());
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.add(egui::TextEdit::singleline(&mut self.main_exe).hint_text(tr(&self.lang, "e.g. myapp.exe (relative to input folder)", "例如 myapp.exe（相对于输入文件夹）", "例: myapp.exe（入力フォルダからの相対パス）", "예: myapp.exe (입력 폴더 기준)", "напр. myapp.exe (относительно папки)")).desired_width(400.0));
                    if ui.button(tr(&self.lang, "Browse", "浏览", "参照", "찾기", "Обзор")).clicked() {
                        if let Some(path) = rfd::FileDialog::new().add_filter("Executables", &["exe", ""]).pick_file() { self.main_exe = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default(); }
                    }
                });
                ui.add_space(18.0);
                // Output folder
                ui.label(egui::RichText::new(format!("\u{1F4C2}  {}", tr(&self.lang, "Output Folder", "输出文件夹", "出力フォルダ", "출력 폴더", "Выходная папка"))).size(14.0).strong());
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.add(egui::TextEdit::singleline(&mut self.output_folder).hint_text(tr(&self.lang, "Where to save the packed file...", "打包文件保存到哪里...", "パックしたファイルの保存先...", "패킹된 파일 저장 위치...", "Куда сохранить упакованный файл...")).desired_width(400.0));
                    if ui.button(tr(&self.lang, "Browse", "浏览", "参照", "찾기", "Обзор")).clicked() { if let Some(path) = rfd::FileDialog::new().pick_folder() { self.output_folder = path.display().to_string(); } }
                });
                ui.add_space(18.0);
                // Output filename
                ui.label(egui::RichText::new(format!("\u{1F4DD}  {}", tr(&self.lang, "Output Filename", "输出文件名", "出力ファイル名", "출력 파일명", "Имя выходного файла"))).size(14.0).strong());
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.add(egui::TextEdit::singleline(&mut self.output_name).hint_text(tr(&self.lang, "e.g. MyApp", "例如 MyApp", "例: MyApp", "예: MyApp", "напр. MyApp")).desired_width(300.0));
                    ui.label(".exe");
                });
                if !self.output_folder.is_empty() && !self.output_name.is_empty() {
                    ui.add_space(10.0);
                    self.output_path = format!("{}/{}.exe", self.output_folder.trim_end_matches('/').trim_end_matches('\\'), self.output_name.trim());
                    ui.label(egui::RichText::new(format!("{} {}", tr(&self.lang, "Saved to:", "保存到：", "保存先：", "저장 위치:", "Сохранено в:"), self.output_path)).size(12.0));
                }
            });

            ui.add_space(16.0);
            // Compression
            ui.horizontal(|ui| {
                ui.label(format!("{} ", tr(&self.lang, "Compression:", "压缩：", "圧縮：", "압축:", "Сжатие:")));
                ui.selectable_value(&mut self.compression, 3, "Fast"); ui.selectable_value(&mut self.compression, 8, "Balanced");
                ui.selectable_value(&mut self.compression, 16, "Small"); ui.selectable_value(&mut self.compression, 22, "Tiny");
            });
            ui.add_space(12.0);
            // Options row
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.memory_mode, tr(&self.lang, "Memory Mode", "内存模式", "メモリモード", "메모리 모드", "Режим памяти"));
                ui.add_space(16.0);
                ui.checkbox(&mut self.admin_mode, tr(&self.lang, "Run as Admin", "管理员运行", "管理者として実行", "관리자 권한", "Запуск от админа"));
            });
            if self.memory_mode || self.admin_mode {
                ui.add_space(4.0);
                let mut hints = Vec::new();
                if self.memory_mode { hints.push(tr(&self.lang, "auto-cleanup", "自动清理", "自動クリーンアップ", "자동 정리", "автоочистка")); }
                if self.admin_mode { hints.push(tr(&self.lang, "admin privileges", "管理员权限", "管理者権限", "관리자 권한", "права админа")); }
                ui.label(egui::RichText::new(format!("{} {}", tr(&self.lang, "Enabled:", "已启用：", "有効：", "활성화:", "Включено:"), hints.join(", "))).size(11.0).weak());
            }

            ui.add_space(12.0);
            let ready = !self.input_path.is_empty() && !self.output_folder.is_empty() && !self.output_name.is_empty();
            ui.add_enabled_ui(!self.packing && ready, |ui| {
                if ui.add(egui::Button::new(egui::RichText::new("GO!").size(42.0).color(egui::Color32::WHITE)).fill(egui::Color32::from_rgb(46, 204, 113)).min_size(egui::vec2(320.0, 85.0)).rounding(egui::Rounding::same(16.0))).clicked() {
                    self.command_path = self.main_exe.clone(); self.runner_index = 0;
                    self.runner = self.available_runners.first().cloned().unwrap_or_default();
                    self.unpack_target = UnpackTarget::Temp; self.versioning = Versioning::SideBySide;
                    self.verification = Verification::Existence; self.console = ConsoleMode::Never;
                    self.current_dir = CurrentDir::Inherit;
                    self.cleanup = self.memory_mode; self.once = self.memory_mode;
                    self.build_dictionary = false; self.version_string.clear(); self.env_vars.clear();
                    self.icon_path.clear(); self.exclude_patterns.clear();
                    self.start_packing();
                }
            });
            if !ready && !self.packing {
                ui.add_space(8.0);
                ui.label(egui::RichText::new(tr(&self.lang, "Fill in all fields above to start", "填完上面所有字段即可开始", "上の項目をすべて入力してください", "위의 모든 필드를 입력하세요", "Заполните все поля выше")).size(13.0));
            }
        });
    }

    fn show_basic_tab(&mut self, ui: &mut egui::Ui) {
        egui::Grid::new("basic_grid").num_columns(3).spacing([10.0, 8.0]).striped(true).show(ui, |ui| {
            ui.label(tr(&self.lang, "Input:", "输入：", "入力：", "입력:", "Вход:"));
            ui.add(egui::TextEdit::singleline(&mut self.input_path).hint_text(tr(&self.lang, "Directory or executable to pack...", "要打包的目录或可执行文件...", "パックするディレクトリまたは実行ファイル...", "패킹할 디렉토리 또는 실행 파일...", "Папка или исполняемый файл...")).desired_width(400.0));
            if ui.button(tr(&self.lang, "Browse...", "浏览...", "参照...", "찾기...", "Обзор...")).clicked() { if let Some(path) = rfd::FileDialog::new().pick_folder() { self.input_path = path.display().to_string(); } }
            ui.end_row();
            ui.label(tr(&self.lang, "Command:", "命令：", "コマンド：", "명령:", "Команда:"));
            ui.add(egui::TextEdit::singleline(&mut self.command_path).hint_text(tr(&self.lang, "Executable relative to input...", "相对于输入目录的可执行文件...", "入力に対する実行ファイル...", "입력 기준 실행 파일...", "Исполняемый файл относительно входа...")).desired_width(400.0));
            if ui.button(tr(&self.lang, "Browse...", "浏览...", "参照...", "찾기...", "Обзор...")).clicked() { if let Some(path) = rfd::FileDialog::new().pick_file() { self.command_path = path.display().to_string(); } }
            ui.end_row();
            ui.label(tr(&self.lang, "Output:", "输出：", "出力：", "출력:", "Выход:"));
            ui.add(egui::TextEdit::singleline(&mut self.output_path).hint_text(tr(&self.lang, "Output executable path...", "输出可执行文件路径...", "出力実行ファイルのパス...", "출력 실행 파일 경로...", "Путь выходного файла...")).desired_width(400.0));
            if ui.button(tr(&self.lang, "Save as...", "另存为...", "名前を付けて保存...", "다른 이름으로 저장...", "Сохранить как...")).clicked() { if let Some(path) = rfd::FileDialog::new().save_file() { self.output_path = path.display().to_string(); } }
            ui.end_row();
            ui.label(tr(&self.lang, "Compression:", "压缩：", "圧縮：", "압축:", "Сжатие:"));
            ui.add(egui::Slider::new(&mut self.compression, 0..=22).text(tr(&self.lang, "level", "级别", "レベル", "레벨", "уровень")));
            ui.label(format!("{}: {}", tr(&self.lang, "Level", "级别", "レベル", "레벨", "Уровень"), self.compression));
            ui.end_row();
            ui.label(tr(&self.lang, "Runner:", "运行器：", "ランナー：", "런너:", "Раннер:"));
            egui::ComboBox::from_id_salt("runner_combo").selected_text(&self.runner).show_ui(ui, |ui| {
                for (i, r) in self.available_runners.iter().enumerate() { if ui.selectable_value(&mut self.runner_index, i, r).clicked() { self.runner = r.clone(); } }
            });
            ui.end_row();
            ui.label(tr(&self.lang, "Options:", "选项：", "オプション：", "옵션:", "Опции:"));
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.memory_mode, tr(&self.lang, "Memory Mode", "内存模式", "メモリモード", "메모리 모드", "Режим памяти"));
                ui.checkbox(&mut self.admin_mode, tr(&self.lang, "Run as Admin", "管理员运行", "管理者として実行", "관리자 권한", "Запуск от админа"));
            });
            ui.end_row();
        });
    }

    fn show_advanced_tab(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("advanced_grid").num_columns(2).spacing([10.0, 6.0]).striped(true).show(ui, |ui| {
                ui.label(tr(&self.lang, "Unpack Target:", "解压目标：", "展開先：", "압축 해제 대상:", "Цель распаковки:"));
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.unpack_target, UnpackTarget::Temp, tr(&self.lang, "Temp", "临时", "一時", "임시", "Врем"));
                    ui.selectable_value(&mut self.unpack_target, UnpackTarget::Local, tr(&self.lang, "Local", "本地", "ローカル", "로컬", "Локально"));
                    ui.selectable_value(&mut self.unpack_target, UnpackTarget::Cwd, tr(&self.lang, "CWD", "当前目录", "現在のディレクトリ", "현재 디렉토리", "Тек. папка"));
                }); ui.end_row();
                ui.label(tr(&self.lang, "Versioning:", "版本策略：", "バージョン管理：", "버전 관리:", "Версионирование:"));
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.versioning, Versioning::SideBySide, tr(&self.lang, "Side-by-side", "并存", "並存", "병존", "Рядом"));
                    ui.selectable_value(&mut self.versioning, Versioning::Replace, tr(&self.lang, "Replace", "替换", "置換", "교체", "Заменить"));
                    ui.selectable_value(&mut self.versioning, Versioning::None, tr(&self.lang, "None", "无", "なし", "없음", "Нет"));
                }); ui.end_row();
                ui.label(tr(&self.lang, "Verification:", "校验：", "検証：", "검증:", "Проверка:"));
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.verification, Verification::Existence, tr(&self.lang, "Existence", "存在", "存在", "존재", "Сущ."));
                    ui.selectable_value(&mut self.verification, Verification::Checksum, tr(&self.lang, "Checksum", "校验和", "チェックサム", "체크섬", "Контр. сумма"));
                    ui.selectable_value(&mut self.verification, Verification::None, tr(&self.lang, "None", "无", "なし", "없음", "Нет"));
                }); ui.end_row();
                ui.label(tr(&self.lang, "Version String:", "版本字符串：", "バージョン文字列：", "버전 문자열:", "Строка версии:"));
                ui.add(egui::TextEdit::singleline(&mut self.version_string).hint_text(tr(&self.lang, "Custom version (max 16 chars)...", "自定义版本（最多16字符）...", "カスタムバージョン（最大16文字）...", "사용자 정의 버전 (최대 16자)...", "Своя версия (макс 16 симв.)...")).desired_width(300.0)); ui.end_row();
                ui.label(tr(&self.lang, "Show Info:", "显示信息：", "情報表示：", "정보 표시:", "Показывать инфо:"));
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.show_information, ShowInfo::Title, tr(&self.lang, "Title", "标题", "タイトル", "제목", "Заголовок"));
                    ui.selectable_value(&mut self.show_information, ShowInfo::Verbose, tr(&self.lang, "Verbose", "详细", "詳細", "상세", "Подробно"));
                    ui.selectable_value(&mut self.show_information, ShowInfo::None, tr(&self.lang, "None", "无", "なし", "없음", "Нет"));
                }); ui.end_row();
                ui.label(tr(&self.lang, "Console:", "控制台：", "コンソール：", "콘솔:", "Консоль:"));
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.console, ConsoleMode::Auto, tr(&self.lang, "Auto", "自动", "自動", "자동", "Авто"));
                    ui.selectable_value(&mut self.console, ConsoleMode::Always, tr(&self.lang, "Always", "总是", "常に", "항상", "Всегда"));
                    ui.selectable_value(&mut self.console, ConsoleMode::Never, tr(&self.lang, "Never", "从不", "なし", "없음", "Никогда"));
                    ui.selectable_value(&mut self.console, ConsoleMode::Attach, tr(&self.lang, "Attach", "附加", "アタッチ", "연결", "Прикрепить"));
                }); ui.end_row();
                ui.label(tr(&self.lang, "Working Dir:", "工作目录：", "作業ディレクトリ：", "작업 디렉토리:", "Рабочая папка:"));
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.current_dir, CurrentDir::Inherit, tr(&self.lang, "Inherit", "继承", "継承", "상속", "Наследовать"));
                    ui.selectable_value(&mut self.current_dir, CurrentDir::Unpack, tr(&self.lang, "Unpack", "解压目录", "展開先", "압축 해제", "Распаковка"));
                    ui.selectable_value(&mut self.current_dir, CurrentDir::Runner, tr(&self.lang, "Runner", "运行器", "ランナー", "런너", "Раннер"));
                    ui.selectable_value(&mut self.current_dir, CurrentDir::Command, tr(&self.lang, "Command", "命令", "コマンド", "명령", "Команда"));
                }); ui.end_row();
                ui.label(tr(&self.lang, "Env Vars:", "环境变量：", "環境変数：", "환경 변수:", "Перем. окружения:"));
                ui.add(egui::TextEdit::singleline(&mut self.env_vars).hint_text("KEY1=value1 KEY2=value2 ...").desired_width(400.0)); ui.end_row();
                ui.label(tr(&self.lang, "Icon:", "图标：", "アイコン：", "아이콘:", "Иконка:"));
                ui.add(egui::TextEdit::singleline(&mut self.icon_path).hint_text(tr(&self.lang, "Path to .ico file (Windows only)...", ".ico 文件路径（仅 Windows）...", ".icoファイルのパス（Windowsのみ）...", ".ico 파일 경로 (Windows만)...", "Путь к .ico (только Windows)...")).desired_width(300.0));
                if ui.button(tr(&self.lang, "Browse...", "浏览...", "参照...", "찾기...", "Обзор...")).clicked() { if let Some(path) = rfd::FileDialog::new().add_filter("Icons", &["ico", "png", "jpg"]).pick_file() { self.icon_path = path.display().to_string(); } }
                ui.end_row();
                ui.label(tr(&self.lang, "Exclude:", "排除：", "除外：", "제외:", "Исключить:"));
                ui.add(egui::TextEdit::singleline(&mut self.exclude_patterns).hint_text("*.log node_modules/** .git/** ...").desired_width(400.0)); ui.end_row();
                ui.label(tr(&self.lang, "Options:", "选项：", "オプション：", "옵션:", "Опции:"));
                ui.vertical(|ui| {
                    ui.checkbox(&mut self.cleanup, tr(&self.lang, "Cleanup after exit", "退出后清理", "終了後にクリーンアップ", "종료 후 정리", "Очистка после выхода"));
                    ui.checkbox(&mut self.once, tr(&self.lang, "Single instance only", "仅允许单实例", "単一インスタンスのみ", "단일 인스턴스만", "Только один экземпляр"));
                    ui.checkbox(&mut self.build_dictionary, tr(&self.lang, "Build compression dictionary", "构建压缩字典", "圧縮辞書を構築", "압축 사전 빌드", "Построить словарь сжатия"));
                    ui.checkbox(&mut self.memory_mode, tr(&self.lang, "Memory Mode", "内存模式", "メモリモード", "메모리 모드", "Режим памяти"));
                    ui.checkbox(&mut self.admin_mode, tr(&self.lang, "Run as Admin", "管理员运行", "管理者として実行", "관리자 권한", "Запуск от админа"));
                }); ui.end_row();
            });
        });
    }

    fn show_about_tab(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(40.0);
            ui.label(egui::RichText::new("wrappe GUI").size(32.0).strong());
            ui.add_space(8.0);
            ui.label(egui::RichText::new(tr(&self.lang, "Pack executables into self-contained single binaries", "将可执行程序打包为自包含单文件", "実行ファイルを自己完結型の単一バイナリにパック", "실행 파일을 자체 포함 단일 바이너리로 패킹", "Упаковка исполняемых файлов в автономные бинарные файлы")).size(14.0));
            ui.add_space(20.0);
            ui.label(format!("{} {}", tr(&self.lang, "Version:", "版本：", "バージョン：", "버전:", "Версия:"), env!("CARGO_PKG_VERSION")));
            ui.add_space(8.0);
            ui.label(tr(&self.lang, "Built with Rust + egui + zstd", "使用 Rust + egui + zstd 构建", "Rust + egui + zstd で構築", "Rust + egui + zstd로 빌드", "Собрано на Rust + egui + zstd"));
            ui.add_space(8.0);
            ui.hyperlink_to("GitHub: ruin321/wrappe-NEWGUI", "https://github.com/ruin321/wrappe-NEWGUI");
            ui.add_space(4.0);
            ui.label(egui::RichText::new(tr(&self.lang, "Original project by Systemcluster", "原始项目作者 Systemcluster", "オリジナルプロジェクト: Systemcluster", "원본 프로젝트: Systemcluster", "Оригинальный проект: Systemcluster")).size(11.0).weak());
            ui.add_space(40.0);
            egui::Frame::none()
                .fill(if self.dark_mode { egui::Color32::from_rgb(35, 35, 42) } else { egui::Color32::from_rgb(245, 245, 248) })
                .rounding(egui::Rounding::same(8.0)).inner_margin(egui::Margin::symmetric(24.0, 16.0))
                .show(ui, |ui| {
                    ui.set_width(500.0);
                    ui.label(egui::RichText::new(tr(&self.lang, "Quick Guide", "快速指南", "クイックガイド", "빠른 가이드", "Краткое руководство")).size(16.0).strong());
                    ui.add_space(8.0);
                    ui.label(tr(&self.lang, "1. Select your app folder (Simple tab)", "1. 选择你的应用文件夹（简单标签页）", "1. アプリのフォルダを選択（簡単タブ）", "1. 앱 폴더 선택 (간단 탭)", "1. Выберите папку приложения"));
                    ui.label(tr(&self.lang, "2. Pick the main .exe file inside it", "2. 选择里面的主 .exe 文件", "2. 中のメイン.exeファイルを選択", "2. 메인 .exe 파일 선택", "2. Выберите главный .exe файл"));
                    ui.label(tr(&self.lang, "3. Choose where to save the packed file", "3. 选择打包文件的保存位置", "3. パックしたファイルの保存先を選択", "3. 패킹된 파일 저장 위치 선택", "3. Выберите куда сохранить"));
                    ui.label(tr(&self.lang, "4. Hit GO!", "4. 点击 GO！", "4. GO!をクリック", "4. GO! 클릭", "4. Нажмите GO!"));
                    ui.add_space(4.0);
                    ui.label(egui::RichText::new(tr(&self.lang, "For more options, use Basic or Advanced tabs.", "更多选项请使用基础或高级标签页。", "詳細オプションは基本または詳細タブをご利用ください。", "더 많은 옵션은 기본 또는 고급 탭을 사용하세요.", "Больше опций на вкладках Базовый и Продвинутый.")).size(12.0).weak());
                    ui.add_space(8.0);
                    ui.label(egui::RichText::new(tr(&self.lang, "Memory Mode: auto-cleanup + single instance. Admin Mode: requires elevation.", "内存模式：自动清理+单实例。管理员模式：需要提权运行。", "メモリモード：自動クリーンアップ+単一インスタンス。管理者モード：昇格が必要。", "메모리 모드: 자동 정리+단일 인스턴스. 관리자 모드: 권한 상승 필요.", "Режим памяти: автоочистка+один экземпляр. Режим админа: требует повышения.")).size(11.0).weak());
                });
        });
    }

    fn build_config(&self) -> PackConfig {
        PackConfig {
            runner: self.runner.clone(), compression: self.compression,
            unpack_target: self.unpack_target.as_str().to_string(), unpack_directory: None,
            versioning: self.versioning.as_str().to_string(), verification: self.verification.as_str().to_string(),
            version_string: if self.version_string.is_empty() { None } else { Some(self.version_string.clone()) },
            show_information: self.show_information.as_str().to_string(),
            console: self.console.as_str().to_string(), current_dir: self.current_dir.as_str().to_string(),
            env: self.env_vars.split_whitespace().filter(|s| s.contains('=')).map(|s| s.to_string()).collect(),
            icon: if self.icon_path.is_empty() { None } else { Some(PathBuf::from(&self.icon_path)) },
            cleanup: self.cleanup, once: self.once, build_dictionary: self.build_dictionary,
            input: PathBuf::from(&self.input_path),
            command: if self.command_path.is_empty() { None } else { Some(PathBuf::from(&self.command_path)) },
            output: if self.output_path.is_empty() { None } else { Some(PathBuf::from(&self.output_path)) },
            arguments: Vec::new(),
            exclude_patterns: self.exclude_patterns.split_whitespace().filter(|s| !s.is_empty()).map(|s| s.to_string()).collect(),
            admin: self.admin_mode,
        }
    }

    fn start_packing(&mut self) {
        self.packing = true; self.progress = 0.0; self.progress_message = "Starting...".to_string();
        self.result_message = None; self.result_error = false;
        let config = self.build_config();
        let (sender, receiver) = mpsc::channel();
        let cancelled = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let cancelled_clone = cancelled.clone();
        self.progress_receiver = Some(receiver); self.cancel_flag = Some(cancelled);
        let handle = thread::spawn(move || { let _ = wrappe::pack(config, move |progress| { let _ = sender.send(progress); }, cancelled_clone); });
        self.pack_thread = Some(handle);
    }
}
