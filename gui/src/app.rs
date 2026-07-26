use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;

use wrappe::{PackConfig, PackProgress, PackStage};

#[derive(Debug, Clone, PartialEq)]
enum Tab {
    Simple,
    Basic,
    Advanced,
    About,
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

#[derive(Debug, Clone, PartialEq)]
enum Lang {
    En,
    Zh,
    Ja,
    Ko,
    Ru,
}

impl Lang {
    fn name(&self) -> &str {
        match self {
            Lang::En => "English",
            Lang::Zh => "中文",
            Lang::Ja => "日本語",
            Lang::Ko => "한국어",
            Lang::Ru => "Русский",
        }
    }

    fn flag(&self) -> &str {
        match self {
            Lang::En => "EN",
            Lang::Zh => "中",
            Lang::Ja => "日",
            Lang::Ko => "한",
            Lang::Ru => "RU",
        }
    }
}

// Translation macro-like system
macro_rules! t {
    ($self:expr, $key:ident) => {
        match $self.lang {
            Lang::En => $key::EN,
            Lang::Zh => $key::ZH,
            Lang::Ja => $key::JA,
            Lang::Ko => $key::KO,
            Lang::Ru => $key::RU,
        }
    };
}

// Translation keys
mod tr {
    pub struct AppTitle;
    impl AppTitle { pub const EN: &str = "wrappe GUI"; pub const ZH: &str = "wrappe GUI"; pub const JA: &str = "wrappe GUI"; pub const KO: &str = "wrappe GUI"; pub const RU: &str = "wrappe GUI"; }

    pub struct Subtitle;
    impl Subtitle { pub const EN: &str = "pack your app into one file"; pub const ZH: &str = "打包你的应用到一个文件"; pub const JA: &str = "アプリを1つのファイルに"; pub const KO: &str = "앱을 하나의 파일로 패킹"; pub const RU: &str = "упакуйте приложение в один файл"; }

    pub struct TabSimple;
    impl TabSimple { pub const EN: &str = "Simple"; pub const ZH: &str = "简单"; pub const JA: &str = "簡単"; pub const KO: &str = "간단"; pub const RU: &str = "Просто"; }

    pub struct TabBasic;
    impl TabBasic { pub const EN: &str = "Basic"; pub const ZH: &str = "基础"; pub const JA: &str = "基本"; pub const KO: &str = "기본"; pub const RU: &str = "Базовый"; }

    pub struct TabAdvanced;
    impl TabAdvanced { pub const EN: &str = "Advanced"; pub const ZH: &str = "高级"; pub const JA: &str = "詳細"; pub const KO: &str = "고급"; pub const RU: &str = "Продвинутый"; }

    pub struct TabAbout;
    impl TabAbout { pub const EN: &str = "About"; pub const ZH: &str = "关于"; pub const JA: &str = "情報"; pub const KO: &str = "정보"; pub const RU: &str = "О пр��грамме"; }

    pub struct SimpleTitle;
    impl SimpleTitle { pub const EN: &str = "Super Simple Mode"; pub const ZH: &str = "超级简单模式"; pub const JA: &str = "超簡単モード"; pub const KO: &str = "초간단 모드"; pub const RU: &str = "Супер простой режим"; }

    pub struct SimpleDesc;
    impl SimpleDesc { pub const EN: &str = "Just pick a folder, choose the main exe, save it. Done."; pub const ZH: &str = "选个文件夹，选主程序，保存。完事。"; pub const JA: &str = "フォルダを選んで、exeを選んで、保存するだけ。"; pub const KO: &str = "폴더 선택, exe 선택, 저장. 끝."; pub const RU: &str = "Выберите папку, exe, сохраните. Готово."; }

    pub struct InputFolder;
    impl InputFolder { pub const EN: &str = "Input Folder"; pub const ZH: &str = "输入文件夹"; pub const JA: &str = "入力フォルダ"; pub const KO: &str = "입력 폴더"; pub const RU: &str = "Входная папка"; }

    pub struct InputFolderHint;
    impl InputFolderHint { pub const EN: &str = "Select the folder containing your app..."; pub const ZH: &str = "选择包含你应用的文件夹..."; pub const JA: &str = "アプリが含まれるフォルダを選択..."; pub const KO: &str = "앱이 있는 폴더 선택..."; pub const RU: &str = "Выберите папку с приложением..."; }

    pub struct MainExe;
    impl MainExe { pub const EN: &str = "Main Executable"; pub const ZH: &str = "主程序"; pub const JA: &str = "メイン実行ファイル"; pub const KO: &str = "메인 실행 파일"; pub const RU: &str = "Главный исполняемый файл"; }

    pub struct MainExeHint;
    impl MainExeHint { pub const EN: &str = "e.g. myapp.exe (relative to input folder)"; pub const ZH: &str = "例如 myapp.exe（相对于输入文件夹）"; pub const JA: &str = "例: myapp.exe（入力フォルダからの相対パス）"; pub const KO: &str = "예: myapp.exe (입력 폴더 기준)"; pub const RU: &str = "напр. myapp.exe (относительно папки)"; }

    pub struct OutputFolder;
    impl OutputFolder { pub const EN: &str = "Output Folder"; pub const ZH: &str = "输出文件夹"; pub const JA: &str = "出力フォルダ"; pub const KO: &str = "출력 폴더"; pub const RU: &str = "Выходная папка"; }

    pub struct OutputFolderHint;
    impl OutputFolderHint { pub const EN: &str = "Where to save the packed file..."; pub const ZH: &str = "打包文件保存到哪里..."; pub const JA: &str = "パックしたファイルの保存先..."; pub const KO: &str = "패킹된 파일 저장 위치..."; pub const RU: &str = "Куда сохранить упакованный файл..."; }

    pub struct OutputFilename;
    impl OutputFilename { pub const EN: &str = "Output Filename"; pub const ZH: &str = "输出文件名"; pub const JA: &str = "出力ファイル名"; pub const KO: &str = "출력 파일명"; pub const RU: &str = "Имя выходного файла"; }

    pub struct OutputFilenameHint;
    impl OutputFilenameHint { pub const EN: &str = "e.g. MyApp"; pub const ZH: &str = "例如 MyApp"; pub const JA: &str = "例: MyApp"; pub const KO: &str = "예: MyApp"; pub const RU: &str = "напр. MyApp"; }

    pub struct Compression;
    impl Compression { pub const EN: &str = "Compression:"; pub const ZH: &str = "压缩："; pub const JA: &str = "圧縮："; pub const KO: &str = "압축:"; pub const RU: &str = "Сжатие:"; }

    pub struct SavedTo;
    impl SavedTo { pub const EN: &str = "Saved to:"; pub const ZH: &str = "保存到："; pub const JA: &str = "保存先："; pub const KO: &str = "저장 위치:"; pub const RU: &str = "Сохранено в:"; }

    pub struct Browse;
    impl Browse { pub const EN: &str = "Browse"; pub const ZH: &str = "浏览"; pub const JA: &str = "参照"; pub const KO: &str = "찾기"; pub const RU: &str = "Обзор"; }

    pub struct OpenFolder;
    impl OpenFolder { pub const EN: &str = "Open Folder"; pub const ZH: &str = "打开文件夹"; pub const JA: &str = "フォルダを開く"; pub const KO: &str = "폴더 열기"; pub const RU: &str = "Открыть папку"; }

    pub struct Clear;
    impl Clear { pub const EN: &str = "Clear"; pub const ZH: &str = "清除"; pub const JA: &str = "クリア"; pub const KO: &str = "지우기"; pub const RU: &str = "Очистить"; }

    pub struct FillFields;
    impl FillFields { pub const EN: &str = "Fill in all fields above to start"; pub const ZH: &str = "填完上面所有字段即可开始"; pub const JA: &str = "上の項目をすべて入力してください"; pub const KO: &str = "위의 모든 필드를 입력하세요"; pub const RU: &str = "Заполните все поля выше"; }

    pub struct Packing;
    impl Packing { pub const EN: &str = "Packing in progress..."; pub const ZH: &str = "打包中..."; pub const JA: &str = "パック中..."; pub const KO: &str = "패킹 중..."; pub const RU: &str = "Упаковка..."; }

    pub struct AboutTitle;
    impl AboutTitle { pub const EN: &str = "Pack executables into self-contained single binaries"; pub const ZH: &str = "将可执行程序打包为自包含单文件"; pub const JA: &str = "実行ファイルを自己完結型の単一バイナリにパック"; pub const KO: &str = "실행 파일을 자체 포함 단일 바이너리로 패킹"; pub const RU: &str = "Упаковка исполняемых файлов в автономные бинарные файлы"; }

    pub struct AboutVersion;
    impl AboutVersion { pub const EN: &str = "Version:"; pub const ZH: &str = "版本："; pub const JA: &str = "バージョン："; pub const KO: &str = "버전:"; pub const RU: &str = "Версия:"; }

    pub struct AboutBuilt;
    impl AboutBuilt { pub const EN: &str = "Built with Rust + egui + zstd"; pub const ZH: &str = "使用 Rust + egui + zstd 构建"; pub const JA: &str = "Rust + egui + zstd で構築"; pub const KO: &str = "Rust + egui + zstd로 빌드"; pub const RU: &str = "Собрано на Rust + egui + zstd"; }

    pub struct AboutOriginal;
    impl AboutOriginal { pub const EN: &str = "Original project by Systemcluster"; pub const ZH: &str = "原始项目作者 Systemcluster"; pub const JA: &str = "オリジナルプロジェクト: Systemcluster"; pub const KO: &str = "원본 프로젝트: Systemcluster"; pub const RU: &str = "Оригинальный проект: Systemcluster"; }

    pub struct AboutGuide;
    impl AboutGuide { pub const EN: &str = "Quick Guide"; pub const ZH: &str = "快速指南"; pub const JA: &str = "クイックガイド"; pub const KO: &str = "빠른 가이드"; pub const RU: &str = "Краткое руководство"; }

    pub struct AboutStep1;
    impl AboutStep1 { pub const EN: &str = "1. Select your app folder (Simple tab)"; pub const ZH: &str = "1. 选择你的应用文件夹（简单标签页）"; pub const JA: &str = "1. アプリのフォルダを選択（簡単タブ）"; pub const KO: &str = "1. 앱 폴더 선택 (간단 탭)"; pub const RU: &str = "1. Выберите папку приложения (вкладка Просто)"; }

    pub struct AboutStep2;
    impl AboutStep2 { pub const EN: &str = "2. Pick the main .exe file inside it"; pub const ZH: &str = "2. 选择里面的主 .exe 文件"; pub const JA: &str = "2. 中のメイン.exeファイルを選択"; pub const KO: &str = "2. 메인 .exe 파일 선택"; pub const RU: &str = "2. Выберите главный .exe файл внутри"; }

    pub struct AboutStep3;
    impl AboutStep3 { pub const EN: &str = "3. Choose where to save the packed file"; pub const ZH: &str = "3. 选择打包文件的保存位置"; pub const JA: &str = "3. パックしたファイルの保存先を選択"; pub const KO: &str = "3. 패킹된 파일 저장 위치 선택"; pub const RU: &str = "3. Выберите куда сохранить файл"; }

    pub struct AboutStep4;
    impl AboutStep4 { pub const EN: &str = "4. Hit GO!"; pub const ZH: &str = "4. 点击 GO！"; pub const JA: &str = "4. GO!をクリック"; pub const KO: &str = "4. GO! 클릭"; pub const RU: &str = "4. Нажмите GO!"; }

    pub struct AboutMore;
    impl AboutMore { pub const EN: &str = "For more options, use Basic or Advanced tabs."; pub const ZH: &str = "更多选项请使用基础或高级标签页。"; pub const JA: &str = "詳細オプションは基本または詳細タブをご利用ください。"; pub const KO: &str = "더 많은 옵션은 기본 또는 고급 탭을 사용하세요."; pub const RU: &str = "Больше опций на вкладках Базовый и Продвинутый."; }

    // Basic/Advanced tab labels
    pub struct UnpackTarget;
    impl UnpackTarget { pub const EN: &str = "Unpack Target:"; pub const ZH: &str = "解压目标："; pub const JA: &str = "展開��："; pub const KO: &str = "압축 해제 대상:"; pub const RU: &str = "Цель распаковки:"; }

    pub struct Versioning;
    impl Versioning { pub const EN: &str = "Versioning:"; pub const ZH: &str = "版本策略："; pub const JA: &str = "バージョン管理："; pub const KO: &str = "버전 관리:"; pub const RU: &str = "Версионирование:"; }

    pub struct Verification;
    impl Verification { pub const EN: &str = "Verification:"; pub const ZH: &str = "校验："; pub const JA: &str = "検証："; pub const KO: &str = "검증:"; pub const RU: &str = "Проверка:"; }

    pub struct VersionString;
    impl VersionString { pub const EN: &str = "Version String:"; pub const ZH: &str = "版本字符串："; pub const JA: &str = "バージョン文字列："; pub const KO: &str = "버전 문자열:"; pub const RU: &str = "Строка версии:"; }

    pub struct ShowInfo;
    impl ShowInfo { pub const EN: &str = "Show Info:"; pub const ZH: &str = "显示信息："; pub const JA: &str = "情報表示："; pub const KO: &str = "정보 표시:"; pub const RU: &str = "Показывать инфо:"; }

    pub struct Console;
    impl Console { pub const EN: &str = "Console:"; pub const ZH: &str = "控制台："; pub const JA: &str = "コンソール："; pub const KO: &str = "콘솔:"; pub const RU: &str = "Консоль:"; }

    pub struct WorkingDir;
    impl WorkingDir { pub const EN: &str = "Working Dir:"; pub const ZH: &str = "工作目录："; pub const JA: &str = "作業ディレクトリ："; pub const KO: &str = "작업 디렉토리:"; pub const RU: &str = "Рабочая папка:"; }

    pub struct EnvVars;
    impl EnvVars { pub const EN: &str = "Env Vars:"; pub const ZH: &str = "环境变量："; pub const JA: &str = "環境変数："; pub const KO: &str = "환경 변수:"; pub const RU: &str = "Перем. окружения:"; }

    pub struct Icon;
    impl Icon { pub const EN: &str = "Icon:"; pub const ZH: &str = "图标："; pub const JA: &str = "アイコン："; pub const KO: &str = "아이콘:"; pub const RU: &str = "Иконка:"; }

    pub struct Exclude;
    impl Exclude { pub const EN: &str = "Exclude:"; pub const ZH: &str = "排除："; pub const JA: &str = "除外："; pub const KO: &str = "제외:"; pub const RU: &str = "Исключить:"; }

    pub struct Options;
    impl Options { pub const EN: &str = "Options:"; pub const ZH: &str = "选项："; pub const JA: &str = "オプション："; pub const KO: &str = "옵션:"; pub const RU: &str = "Опции:"; }

    pub struct Cleanup;
    impl Cleanup { pub const EN: &str = "Cleanup after exit"; pub const ZH: &str = "退出后清理"; pub const JA: &str = "終了後にクリーンアップ"; pub const KO: &str = "종료 후 정리"; pub const RU: &str = "Очистка после выхода"; }

    pub struct SingleInstance;
    impl SingleInstance { pub const EN: &str = "Single instance only"; pub const ZH: &str = "仅允许单实例"; pub const JA: &str = "単一インスタンスのみ"; pub const KO: &str = "단일 인스턴스만"; pub const RU: &str = "Только один экземпляр"; }

    pub struct BuildDict;
    impl BuildDict { pub const EN: &str = "Build compression dictionary"; pub const ZH: &str = "构建压缩字典"; pub const JA: &str = "圧縮辞書を構築"; pub const KO: &str = "압축 사전 빌드"; pub const RU: &str = "Построить словарь сжатия"; }
}
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
    cancel_flag: Option<std::sync::Arc<std::sync::atomic::AtomicBool>>,

    // UI state
    selected_tab: Tab,
    dark_mode: bool,
    lang: Lang,
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
            cancel_flag: None,
            selected_tab: Tab::Simple,
            dark_mode: true,
            lang: Lang::Zh,
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
                } else if stage == PackStage::Cancelled {
                    self.packing = false;
                    self.result_message = Some("Cancelled".to_string());
                    self.result_error = true;
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

        // Title bar
        egui::TopBottomPanel::top("title_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading(t!(self, tr::AppTitle));
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new(t!(self, tr::Subtitle))
                        .size(11.0)
                        .weak(),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Language selector
                    egui::ComboBox::from_id_salt("lang_selector")
                        .selected_text(self.lang.flag())
                        .width(50.0)
                        .show_ui(ui, |ui| {
                            for lang in &[Lang::En, Lang::Zh, Lang::Ja, Lang::Ko, Lang::Ru] {
                                if ui.selectable_label(self.lang == *lang, lang.name()).clicked() {
                                    self.lang = lang.clone();
                                }
                            }
                        });

                    if ui.button(if self.dark_mode { "\u{2600}" } else { "\u{1F319}" }).clicked() {
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
                // Open output folder button after success
                if !self.result_error && !self.output_folder.is_empty() {
                    if ui.button(t!(self, tr::OpenFolder)).clicked() {
                        let _ = open::that(&self.output_folder);
                    }
                }
                if ui.button(t!(self, tr::Clear)).clicked() {
                    self.result_message = None;
                    self.result_error = false;
                }
            }
        });

        // Central panel
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.selected_tab, Tab::Simple, "Simple");
                ui.selectable_value(&mut self.selected_tab, Tab::Basic, "Basic");
                ui.selectable_value(&mut self.selected_tab, Tab::Advanced, "Advanced");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.selectable_value(&mut self.selected_tab, Tab::About, "About");
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
                ui.horizontal(|ui| {
                    ui.label(t!(self, tr::Packing));
                    if ui.button("\u{2716} Cancel").clicked() {
                        if let Some(ref flag) = self.cancel_flag {
                            flag.store(true, std::sync::atomic::Ordering::Relaxed);
                        }
                    }
                });
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

            ui.label(egui::RichText::new("Super Simple Mode").size(28.0).strong());
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new("Just pick a folder, choose the main exe, save it. Done.")
                    .size(13.0),
            );
            ui.add_space(24.0);

            let card = egui::Frame::none()
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
                .rounding(egui::Rounding::same(12.0))
                .inner_margin(egui::Margin::symmetric(32.0, 28.0));

            card.show(ui, |ui| {
                ui.set_width(520.0);

                // Input folder
                ui.label(egui::RichText::new("\u{1F4C1}  Input Folder").size(14.0).strong());
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

                // Main executable
                ui.label(egui::RichText::new("\u{1F4BE}  Main Executable").size(14.0).strong());
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
                ui.label(egui::RichText::new("\u{1F4C2}  Output Folder").size(14.0).strong());
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
                ui.label(egui::RichText::new("\u{1F4DD}  Output Filename").size(14.0).strong());
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut self.output_name)
                            .hint_text("e.g. MyApp")
                            .desired_width(300.0),
                    );
                    ui.label(".exe");
                });

                if !self.output_folder.is_empty() && !self.output_name.is_empty() {
                    ui.add_space(10.0);
                    let preview = format!(
                        "{}/{}.exe",
                        self.output_folder.trim_end_matches('/').trim_end_matches('\\'),
                        self.output_name.trim(),
                    );
                    ui.label(
                        egui::RichText::new(format!("Saved to: {}", preview)).size(12.0),
                    );
                    self.output_path = preview;
                }
            });

            ui.add_space(16.0);

            // Compression presets
            ui.horizontal(|ui| {
                ui.label("Compression:");
                ui.selectable_value(&mut self.compression, 3, "Fast");
                ui.selectable_value(&mut self.compression, 8, "Balanced");
                ui.selectable_value(&mut self.compression, 16, "Small");
                ui.selectable_value(&mut self.compression, 22, "Tiny");
            });

            ui.add_space(16.0);

            let ready = !self.input_path.is_empty()
                && !self.output_folder.is_empty()
                && !self.output_name.is_empty();

            ui.add_enabled_ui(!self.packing && ready, |ui| {
                let btn = egui::Button::new(
                    egui::RichText::new("GO!").size(42.0).color(egui::Color32::WHITE),
                )
                .fill(egui::Color32::from_rgb(46, 204, 113))
                .min_size(egui::vec2(320.0, 85.0))
                .rounding(egui::Rounding::same(16.0));

                if ui.add(btn).clicked() {
                    self.command_path = self.main_exe.clone();
                    self.runner_index = 0;
                    self.runner = self.available_runners.first().cloned().unwrap_or_default();
                    // Keep user's compression preset, don't override
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
                    egui::RichText::new("Fill in all fields above to start").size(13.0),
                );
            }
        });
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
                        .hint_text("Executable relative to input...")
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
                        ui.selectable_value(&mut self.versioning, Versioning::SideBySide, "Side-by-side");
                        ui.selectable_value(&mut self.versioning, Versioning::Replace, "Replace");
                        ui.selectable_value(&mut self.versioning, Versioning::None, "None");
                    });
                    ui.end_row();

                    ui.label("Verification:");
                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut self.verification, Verification::Existence, "Existence");
                        ui.selectable_value(&mut self.verification, Verification::Checksum, "Checksum");
                        ui.selectable_value(&mut self.verification, Verification::None, "None");
                    });
                    ui.end_row();

                    ui.label("Version String:");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.version_string)
                            .hint_text("Custom version (max 16 chars)...")
                            .desired_width(300.0),
                    );
                    ui.end_row();

                    ui.label("Show Info:");
                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut self.show_information, ShowInfo::Title, "Title");
                        ui.selectable_value(&mut self.show_information, ShowInfo::Verbose, "Verbose");
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
                        ui.selectable_value(&mut self.current_dir, CurrentDir::Command, "Command");
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

    fn show_about_tab(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(40.0);
            ui.label(egui::RichText::new("wrappe GUI").size(32.0).strong());
            ui.add_space(8.0);
            ui.label(
                egui::RichText::new("Pack executables into self-contained single binaries")
                    .size(14.0),
            );
            ui.add_space(20.0);

            ui.label(format!("Version: {}", env!("CARGO_PKG_VERSION")));
            ui.add_space(8.0);
            ui.label("Built with Rust + egui + zstd");
            ui.add_space(8.0);
            ui.hyperlink_to(
                "GitHub: ruin321/wrappe-NEWGUI",
                "https://github.com/ruin321/wrappe-NEWGUI",
            );
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new("Original project by Systemcluster").size(11.0).weak(),
            );

            ui.add_space(40.0);

            egui::Frame::none()
                .fill(if self.dark_mode {
                    egui::Color32::from_rgb(35, 35, 42)
                } else {
                    egui::Color32::from_rgb(245, 245, 248)
                })
                .rounding(egui::Rounding::same(8.0))
                .inner_margin(egui::Margin::symmetric(24.0, 16.0))
                .show(ui, |ui| {
                    ui.set_width(500.0);
                    ui.label(egui::RichText::new("Quick Guide").size(16.0).strong());
                    ui.add_space(8.0);
                    ui.label("1. Select your app folder (Simple tab)");
                    ui.label("2. Pick the main .exe file inside it");
                    ui.label("3. Choose where to save the packed file");
                    ui.label("4. Hit GO!");
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new("For more options, use Basic or Advanced tabs.")
                            .size(12.0)
                            .weak(),
                    );
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
        let cancelled = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let cancelled_clone = cancelled.clone();

        self.progress_receiver = Some(receiver);
        self.cancel_flag = Some(cancelled);

        let handle = thread::spawn(move || {
            let _ = wrappe::pack(config, move |progress| {
                let _ = sender.send(progress);
            }, cancelled_clone);
        });

        self.pack_thread = Some(handle);
    }
}
