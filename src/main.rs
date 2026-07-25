use std::path::PathBuf;

use clap::Parser;
use console::{style, Emoji};

#[derive(Parser)]
#[clap(about)]
pub struct Args {
    /// Platform to pack for (see --list-runners for available options)
    #[arg(short = 'r', long, default_value = "native")]
    runner:           String,
    /// Zstd compression level (0-22)
    #[arg(short = 'c', long, default_value = "8")]
    compression:      u32,
    /// Unpack directory target (temp, local, cwd)
    #[arg(short = 't', long, default_value = "temp")]
    unpack_target:    String,
    /// Unpack directory name [default: inferred from input directory]
    #[arg(short = 'd', long)]
    unpack_directory: Option<String>,
    /// Versioning strategy (sidebyside, replace, none)
    #[arg(short = 'v', long, default_value = "sidebyside")]
    versioning:       String,
    /// Verification of existing unpacked data (existence, checksum, none)
    #[arg(short = 'e', long, default_value = "existence")]
    verification:     String,
    /// Version string override [default: randomly generated]
    #[arg(short = 's', long)]
    version_string:   Option<String>,
    /// Information output details (title, verbose, none)
    #[arg(short = 'i', long, default_value = "title")]
    show_information: String,
    /// Show or attach to a console window (auto, always, never, attach)
    #[arg(short = 'n', long, default_value = "auto")]
    console:          String,
    /// Working directory of the command (inherit, unpack, runner, command)
    #[arg(short = 'w', long, default_value = "inherit")]
    current_dir:      String,
    /// Environment variables to store (can be specified multiple times)
    #[arg(short = 'a', long)]
    env:              Vec<String>,
    /// Path to an image to use as Windows executable icon
    #[arg(short = 'm', long)]
    icon:             Option<PathBuf>,
    /// Cleanup the unpack directory after exit
    #[arg(short = 'u', long, default_value = "false")]
    cleanup:          bool,
    /// Only allow one instance of the application to run
    #[arg(short = 'o', long, default_value = "false")]
    once:             bool,
    /// Build compression dictionary
    #[arg(short = 'z', long, default_value = "false")]
    build_dictionary: bool,
    /// Exclude files matching glob pattern (can be specified multiple times)
    #[arg(short = 'x', long)]
    exclude:          Vec<String>,
    /// Read configuration from a TOML file
    #[arg(short = 'f', long)]
    config_file:      Option<PathBuf>,
    /// Suppress non-error output
    #[arg(short = 'q', long, default_value = "false")]
    quiet:            bool,
    /// Read version string from a file
    #[arg(long, conflicts_with = "version_string")]
    version_from_file: Option<PathBuf>,
    /// Print available runners
    #[arg(short = 'l', long)]
    #[allow(dead_code)]
    list_runners:     bool,
    /// Path to the input file or directory
    #[arg(name = "input")]
    input:            PathBuf,
    /// Path to the executable in the input directory or the input file
    #[arg(name = "command")]
    command:          Option<PathBuf>,
    /// Path to or filename of the output executable
    #[arg(name = "output")]
    output:           Option<PathBuf>,
    /// Command line arguments to store
    #[arg(last = true)]
    arguments:        Vec<String>,
    /// Print version
    #[arg(short = 'V', long)]
    #[allow(dead_code)]
    version:          bool,
}

fn main() {
    color_backtrace::install();

    if std::env::args().any(|arg| arg == "-l" || arg == "--list-runners") {
        wrappe::list_runners();
        std::process::exit(0);
    }

    if std::env::args().any(|arg| arg == "-V" || arg == "--version") {
        println!(
            "{} {}",
            style(env!("CARGO_PKG_NAME")).bold().bright(),
            style(env!("CARGO_PKG_VERSION")).bold().bright(),
        );
        std::process::exit(0);
    }

    let args = Args::parse();

    let quiet = args.quiet;

    if !quiet {
        println!(
            "{}",
            style(format!(
                "{} {}",
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_VERSION"),
            ))
            .bold()
            .bright(),
        );
    }

    // Build PackConfig from args
    let mut config = wrappe::PackConfig::default();

    // If config file specified, load it first
    if let Some(ref config_path) = args.config_file {
        match load_config_file(config_path) {
            Ok(cfg) => config = cfg,
            Err(e) => {
                eprintln!("{}: {}", style("error reading config file").red(), e);
                std::process::exit(-1);
            }
        }
    }

    // Override with CLI args
    config.runner = args.runner;
    config.compression = args.compression;
    config.unpack_target = args.unpack_target;
    config.unpack_directory = args.unpack_directory;
    config.versioning = args.versioning;
    config.verification = args.verification;
    config.show_information = args.show_information;
    config.console = args.console;
    config.current_dir = args.current_dir;
    config.env = args.env;
    config.icon = args.icon;
    config.cleanup = args.cleanup;
    config.once = args.once;
    config.build_dictionary = args.build_dictionary;
    config.input = args.input;
    config.command = args.command;
    config.output = args.output;
    config.arguments = args.arguments;
    config.exclude_patterns = args.exclude;

    // Version string: --version-from-file overrides --version-string
    if let Some(ref path) = args.version_from_file {
        match wrappe::get_version_from_file(path) {
            Ok(v) => config.version_string = Some(v),
            Err(e) => {
                eprintln!("{}: {}", style("error reading version file").red(), e);
                std::process::exit(-1);
            }
        }
    } else if args.version_string.is_some() {
        config.version_string = args.version_string;
    }

    // Notes/warnings
    let runner_name = match wrappe::get_runner_name(&config.runner) {
        Ok(n) => n,
        Err(e) => {
            eprintln!("{}: {}", style("error").red(), e);
            std::process::exit(-1);
        }
    };

    let versioning_val = match wrappe::get_versioning(&config.versioning) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{}: {}", style("error").red(), e);
            std::process::exit(-1);
        }
    };

    if !quiet {
        if (versioning_val == 1 || versioning_val == 2) && !config.once {
            println!(
                "{} {} {} {} {}",
                style("note: chosen versioning").yellow().dim(),
                style(&config.versioning).yellow().bold(),
                style("without option").yellow().dim(),
                style("once").yellow().bold(),
                style("can cause unpacking to fail while the application is already running").dim(),
            );
        }
        if versioning_val == 2 && config.verification != "none" {
            println!(
                "{} {} {}",
                style("note: verification will be ignored with")
                    .yellow()
                    .dim(),
                style(&config.versioning).yellow().bold(),
                style("versioning").yellow().dim(),
            );
        }
        if config.once && !(runner_name.contains("windows") || runner_name.contains("linux")) {
            println!(
                "{} {} {} {}",
                style("note: option").yellow().dim(),
                style("once").yellow().bold(),
                style("is only supported for Windows and Linux runners")
                    .yellow()
                    .dim(),
                style(format!("(target: {})", runner_name)).yellow().dim(),
            );
        }
        if config.icon.is_some() && !runner_name.contains("windows") {
            println!(
                "{}",
                style("note: setting an executable icon is only supported for Windows runners")
                    .yellow()
                    .dim(),
            );
        }
    }

    // Run the pack
    match wrappe::pack(config, move |progress| {
        if quiet {
            return;
        }
        match progress.stage {
            wrappe::PackStage::Counting => {
                println!(
                    "{} {}",
                    style("[1/4]").bold().dim(),
                    progress.message,
                );
            }
            wrappe::PackStage::WritingRunner => {
                println!(
                    "{} {}",
                    style("[2/4]").bold().dim(),
                    progress.message,
                );
            }
            wrappe::PackStage::Compressing => {
                if progress.is_error {
                    println!(
                        "      {}{}",
                        Emoji("❗ ", ""),
                        style(&progress.message).red(),
                    );
                } else if !progress.message.is_empty() {
                    println!(
                        "      {}{}",
                        Emoji("💡 ", ""),
                        style(&progress.message).dim(),
                    );
                }
            }
            wrappe::PackStage::Finalizing => {
                println!(
                    "{} {}",
                    style("[4/4]").bold().dim(),
                    progress.message,
                );
            }
            wrappe::PackStage::Done => {
                println!("      {}{}", Emoji("✨ ", ""), style("done!").green());
                if !progress.message.is_empty() {
                    println!("      {}{}", Emoji("💾 ", ""), style(&progress.message).dim());
                }
            }
        }
    }) {
        Ok(_result) => {}
        Err(e) => {
            eprintln!("{}: {}", style("error").red(), e);
            std::process::exit(-1);
        }
    }
}

/// Load PackConfig from a TOML config file
fn load_config_file(path: &PathBuf) -> Result<wrappe::PackConfig, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("couldn't read {}: {}", path.display(), e))?;
    let value: toml::Value = toml::from_str(&content)
        .map_err(|e| format!("invalid TOML in {}: {}", path.display(), e))?;

    let mut config = wrappe::PackConfig::default();

    if let Some(v) = value.get("runner").and_then(|v| v.as_str()) {
        config.runner = v.to_string();
    }
    if let Some(v) = value.get("compression").and_then(|v| v.as_integer()) {
        config.compression = v as u32;
    }
    if let Some(v) = value.get("unpack_target").and_then(|v| v.as_str()) {
        config.unpack_target = v.to_string();
    }
    if let Some(v) = value.get("unpack_directory").and_then(|v| v.as_str()) {
        config.unpack_directory = Some(v.to_string());
    }
    if let Some(v) = value.get("versioning").and_then(|v| v.as_str()) {
        config.versioning = v.to_string();
    }
    if let Some(v) = value.get("verification").and_then(|v| v.as_str()) {
        config.verification = v.to_string();
    }
    if let Some(v) = value.get("version_string").and_then(|v| v.as_str()) {
        config.version_string = Some(v.to_string());
    }
    if let Some(v) = value.get("show_information").and_then(|v| v.as_str()) {
        config.show_information = v.to_string();
    }
    if let Some(v) = value.get("console").and_then(|v| v.as_str()) {
        config.console = v.to_string();
    }
    if let Some(v) = value.get("current_dir").and_then(|v| v.as_str()) {
        config.current_dir = v.to_string();
    }
    if let Some(v) = value.get("cleanup").and_then(|v| v.as_bool()) {
        config.cleanup = v;
    }
    if let Some(v) = value.get("once").and_then(|v| v.as_bool()) {
        config.once = v;
    }
    if let Some(v) = value.get("build_dictionary").and_then(|v| v.as_bool()) {
        config.build_dictionary = v;
    }
    if let Some(v) = value.get("icon").and_then(|v| v.as_str()) {
        config.icon = Some(PathBuf::from(v));
    }
    if let Some(arr) = value.get("exclude").and_then(|v| v.as_array()) {
        config.exclude_patterns = arr
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect();
    }
    if let Some(table) = value.get("env").and_then(|v| v.as_table()) {
        config.env = table
            .iter()
            .map(|(k, v)| format!("{}={}", k, v.as_str().unwrap_or("")))
            .collect();
    }
    if let Some(arr) = value.get("arguments").and_then(|v| v.as_array()) {
        config.arguments = arr
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect();
    }

    Ok(config)
}
