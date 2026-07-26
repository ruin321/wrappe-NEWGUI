use std::{
    path::PathBuf,
    time::SystemTime,
};

mod types;
pub use types::*;

mod compress;
use compress::compress;

mod args;
pub use args::{
    get_available_runners, get_runner_name, get_version_from_file, get_versioning, list_runners,
};

/// Configuration for a packing operation.
#[derive(Debug, Clone)]
pub struct PackConfig {
    pub runner: String,
    pub compression: u32,
    pub unpack_target: String,
    pub unpack_directory: Option<String>,
    pub versioning: String,
    pub verification: String,
    pub version_string: Option<String>,
    pub show_information: String,
    pub console: String,
    pub current_dir: String,
    pub env: Vec<String>,
    pub icon: Option<PathBuf>,
    pub cleanup: bool,
    pub once: bool,
    pub build_dictionary: bool,
    pub input: PathBuf,
    pub command: Option<PathBuf>,
    pub output: Option<PathBuf>,
    pub arguments: Vec<String>,
    pub exclude_patterns: Vec<String>,
}

impl Default for PackConfig {
    fn default() -> Self {
        PackConfig {
            runner: "native".to_string(),
            compression: 8,
            unpack_target: "temp".to_string(),
            unpack_directory: None,
            versioning: "sidebyside".to_string(),
            verification: "existence".to_string(),
            version_string: None,
            show_information: "title".to_string(),
            console: "auto".to_string(),
            current_dir: "inherit".to_string(),
            env: Vec::new(),
            icon: None,
            cleanup: false,
            once: false,
            build_dictionary: false,
            input: PathBuf::new(),
            command: None,
            output: None,
            arguments: Vec::new(),
            exclude_patterns: Vec::new(),
        }
    }
}

/// Result of a successful pack operation.
#[derive(Debug, Clone)]
pub struct PackResult {
    pub compressed: u64,
    pub read: u64,
    pub written: u64,
    pub output_path: PathBuf,
}

/// Error during packing.
#[derive(Debug)]
pub enum PackError {
    Config(String),
    Io(std::io::Error),
}

impl std::fmt::Display for PackError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PackError::Config(msg) => write!(f, "config error: {}", msg),
            PackError::Io(e) => write!(f, "io error: {}", e),
        }
    }
}

impl From<std::io::Error> for PackError {
    fn from(e: std::io::Error) -> Self {
        PackError::Io(e)
    }
}

/// Progress information during packing.
#[derive(Debug, Clone)]
pub struct PackProgress {
    pub stage: PackStage,
    pub current: u64,
    pub total: u64,
    pub message: String,
    pub is_error: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PackStage {
    Counting,
    WritingRunner,
    Compressing,
    Finalizing,
    Done,
    Cancelled,
}

/// Run the full packing process.
///
/// Progress updates are sent through the `progress` callback.
/// Pass a `cancelled` AtomicBool to allow cancellation.
pub fn pack(
    config: PackConfig,
    progress: impl Fn(PackProgress) + Send + Sync + 'static,
    cancelled: std::sync::Arc<std::sync::atomic::AtomicBool>,
) -> Result<PackResult, PackError> {
    use std::fs::File;
    use std::io::{BufWriter, Cursor, Write};

    use editpe::Image;
    use jwalk::WalkDir;
    use zstd::stream::copy_decode;

    // Validate and resolve config
    let runner = args::get_runner(&config.runner)
        .map_err(|e| PackError::Config(e))?;
    let runner_name = args::get_runner_name(&config.runner)
        .map_err(|e| PackError::Config(e))?;
    let unpack_target = args::get_unpack_target(&config.unpack_target)
        .map_err(|e| PackError::Config(e))?;
    let versioning = args::get_versioning(&config.versioning)
        .map_err(|e| PackError::Config(e))?;
    let version = args::get_version(config.version_string.as_deref())
        .map_err(|e| PackError::Config(e))?;
    let source = args::get_source(&config.input)
        .map_err(|e| PackError::Config(e))?;
    let command_path = args::get_command_path(config.command.as_deref(), &source)
        .map_err(|e| PackError::Config(e))?;
    let command = args::get_command(&command_path)
        .map_err(|e| PackError::Config(e))?;
    let output = args::get_output(config.output.as_deref(), &command_path)
        .map_err(|e| PackError::Config(e))?;
    let unpack_directory = args::get_unpack_directory(config.unpack_directory.as_deref(), &source)
        .map_err(|e| PackError::Config(e))?;
    let verification = args::get_verification(&config.verification)
        .map_err(|e| PackError::Config(e))?;
    let show_information = args::get_show_information(&config.show_information)
        .map_err(|e| PackError::Config(e))?;
    let arguments = args::get_arguments(&config.arguments)
        .map_err(|e| PackError::Config(e))?;
    let current_dir = args::get_current_dir(&config.current_dir)
        .map_err(|e| PackError::Config(e))?;
    let icon_path = args::get_icon_path(config.icon.as_deref())
        .map_err(|e| PackError::Config(e))?;
    let env_vars = args::get_env_vars(&config.env)
        .map_err(|e| PackError::Config(e))?;

    // Compile exclude patterns
    let exclude_patterns: Vec<glob::Pattern> = config
        .exclude_patterns
        .iter()
        .filter_map(|p| match glob::Pattern::new(p) {
            Ok(pattern) => Some(pattern),
            Err(e) => {
                progress(PackProgress {
                    stage: PackStage::Counting,
                    current: 0,
                    total: 0,
                    message: format!("invalid exclude pattern '{}': {}", p, e),
                    is_error: true,
                });
                None
            }
        })
        .collect();

    let mut show_console = args::get_show_console(&config.console, runner_name)
        .map_err(|e| PackError::Config(e))?;
    let once = if config.once { 1 } else { 0 };
    let cleanup = if config.cleanup { 1 } else { 0 };

    if output == source {
        return Err(PackError::Config(format!(
            "output file can't be the input file: {}",
            output.display()
        )));
    }

    let file = File::create(&output)?;

    let canonical_current_dir = std::fs::canonicalize(std::env::current_dir().unwrap()).unwrap();
    let relative_source = source
        .strip_prefix(&canonical_current_dir)
        .unwrap_or(&source);
    let relative_source = if relative_source.components().count() == 0 {
        &canonical_current_dir
    } else {
        relative_source
    };

    let count = if source.is_dir() {
        progress(PackProgress {
            stage: PackStage::Counting,
            current: 0,
            total: 0,
            message: format!("counting contents of {}…", relative_source.display()),
            is_error: false,
        });
        WalkDir::new(&source).skip_hidden(false).into_iter().count() as u64 - 1
    } else {
        progress(PackProgress {
            stage: PackStage::Counting,
            current: 0,
            total: 0,
            message: format!("checking {}…", relative_source.display()),
            is_error: false,
        });
        1
    };

    progress(PackProgress {
        stage: PackStage::WritingRunner,
        current: 0,
        total: count,
        message: format!("writing runner for target {}…", runner_name),
        is_error: false,
    });

    let mut writer = BufWriter::new(file);
    if runner_name.contains("windows") {
        let mut decompressed = Vec::new();
        copy_decode(Cursor::new(runner), &mut decompressed).unwrap();

        let decompressed = (|| -> Result<Vec<u8>, Box<dyn std::error::Error>> {
            let mut runner_image = Image::parse(&decompressed)?;
            runner_image.set_subsystem(if show_console == 1 { 3 } else { 2 });
            Ok(runner_image.data().to_owned())
        })()
        .unwrap_or_else(|error| {
            progress(PackProgress {
                stage: PackStage::WritingRunner,
                current: 0,
                total: count,
                message: format!("failed to set subsystem for runner: {}", error),
                is_error: true,
            });
            decompressed
        });

        let decompressed = (|| -> Result<Vec<u8>, Box<dyn std::error::Error>> {
            let mut runner_image = Image::parse(&decompressed)?;
            let command_path_data = if source.is_file() {
                source.clone()
            } else {
                source.join(args::get_command_path(config.command.as_deref(), &source)
                    .map_err(|e| format!("{}", e))?)
            };
            let command_data = std::fs::read(command_path_data)?;
            let command_image = Image::parse(command_data)?;
            let mut command_resources = command_image
                .resource_directory()
                .cloned()
                .unwrap_or_default();
            if let Some(ref icon) = icon_path {
                command_resources.set_main_icon(image::ImageReader::open(icon)?.decode()?)?;
            }
            if config.console == "auto" {
                show_console = if command_image.subsystem() == 3 { 1 } else { 0 };
                runner_image.set_subsystem(command_image.subsystem());
            }
            runner_image.set_resource_directory(command_resources)?;
            Ok(runner_image.data().to_owned())
        })()
        .unwrap_or_else(|error| {
            progress(PackProgress {
                stage: PackStage::WritingRunner,
                current: 0,
                total: count,
                message: format!("failed to copy resources to runner: {}", error),
                is_error: true,
            });
            decompressed
        });

        writer.write_all(&decompressed).unwrap();
    } else {
        copy_decode(Cursor::new(&runner), &mut writer).unwrap();
    }

    progress(PackProgress {
        stage: PackStage::Compressing,
        current: 0,
        total: count,
        message: format!("compressing {} files and directories…", count),
        is_error: false,
    });

    let now = SystemTime::now();
    let (compressed, read, written) = compress(
        &source,
        &mut writer,
        &output,
        &exclude_patterns,
        config.compression,
        config.build_dictionary,
        || {},
        |message| {
            progress(PackProgress {
                stage: PackStage::Compressing,
                current: 0,
                total: count,
                message: message.to_string(),
                is_error: true,
            });
        },
        |message| {
            progress(PackProgress {
                stage: PackStage::Compressing,
                current: 0,
                total: count,
                message: message.to_string(),
                is_error: false,
            });
        },
        |message| {
            progress(PackProgress {
                stage: PackStage::Compressing,
                current: 0,
                total: count,
                message: message.to_string(),
                is_error: false,
            });
        },
        &cancelled,
    );

    writer.flush()?;

    progress(PackProgress {
        stage: PackStage::Finalizing,
        current: count,
        total: count,
        message: "writing startup configuration…".to_string(),
        is_error: false,
    });

    let info = StarterInfo {
        signature: WRAPPE_SIGNATURE,
        show_console,
        current_dir,
        verification,
        show_information,
        cleanup,
        uid: version.as_bytes().try_into().unwrap(),
        unpack_target,
        versioning,
        unpack_directory,
        once,
        command,
        arguments,
        env: env_vars,
        wrappe_format: WRAPPE_FORMAT,
    };
    writer.write_all(info.as_bytes())?;
    writer.flush()?;
    drop(writer);

    #[cfg(any(unix, target_os = "redox"))]
    {
        use std::fs::{metadata, set_permissions};
        use std::os::unix::prelude::*;
        let mode = metadata(&output)
            .map(|meta| meta.permissions().mode())
            .unwrap_or(0o755);
        set_permissions(&output, PermissionsExt::from_mode(mode | 0o111)).unwrap_or_else(|e| {
            progress(PackProgress {
                stage: PackStage::Finalizing,
                current: count,
                total: count,
                message: format!("failed to set permissions for {}: {}", output.display(), e),
                is_error: true,
            });
        });
    }

    progress(PackProgress {
        stage: PackStage::Done,
        current: count,
        total: count,
        message: format!(
            "{:.2}MB read, {:.2}MB written, {:.2}% of original size, took {:.2}s",
            read as f64 / 1024.0 / 1024.0,
            written as f64 / 1024.0 / 1024.0,
            (written as f64 / read as f64) * 100.0,
            now.elapsed().unwrap_or_default().as_secs_f64()
        ),
        is_error: false,
    });

    Ok(PackResult {
        compressed,
        read,
        written,
        output_path: output,
    })
}
