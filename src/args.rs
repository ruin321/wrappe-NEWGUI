use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};

use rand::{
    distr::{Alphanumeric, Distribution},
    rng,
};
use staticfilemap::StaticFileMap;

use crate::types::{ARGS_SIZE, NAME_SIZE};

#[derive(StaticFileMap)]
#[parse("env")]
#[names("WRAPPE_TARGETS")]
#[files("WRAPPE_FILES")]
#[compression(16)]
#[algorithm("zstd")]
struct StarterMap;

pub fn list_runners() {
    use console::style;
    println!("{}:", style("available runners").blue().bright());
    println!(
        "  {} {}",
        StarterMap::keys()[0],
        style("(default)").bold().dim()
    );
    for runner in &StarterMap::keys()[1..] {
        println!("  {}", runner);
    }
}

pub fn get_available_runners() -> Vec<&'static str> {
    StarterMap::keys().to_vec()
}

pub fn get_runner_name(name: &str) -> Result<&'static str, String> {
    if name == "native" || name == "default" {
        return Ok(StarterMap::keys()[0]);
    }
    StarterMap::get_match_index(name)
        .map(|id| StarterMap::keys()[id])
        .ok_or_else(|| format!("not a valid runner: {}", name))
}

pub fn get_runner(name: &str) -> Result<&'static [u8], String> {
    let runner_name = if name == "native" || name == "default" {
        StarterMap::keys()[0]
    } else {
        name
    };
    StarterMap::get_match(runner_name).ok_or_else(|| format!("not a valid runner: {}", runner_name))
}

pub fn get_unpack_target(directory: &str) -> Result<u8, String> {
    match directory.to_lowercase().as_str() {
        "temp" => Ok(0),
        "default" => Ok(0),
        "local" => Ok(1),
        "cwd" => Ok(2),
        _ => Err(format!("not a valid target directory: {}", directory)),
    }
}

pub fn get_versioning(versioning: &str) -> Result<u8, String> {
    match versioning.to_lowercase().as_str() {
        "sidebyside" => Ok(0),
        "default" => Ok(0),
        "replace" => Ok(1),
        "none" => Ok(2),
        _ => Err(format!("not a valid versioning strategy: {}", versioning)),
    }
}

pub fn get_version(version: Option<&str>) -> Result<String, String> {
    let mut version = if let Some(version) = version {
        if version.len() > 16 {
            return Err("version specifier is longer than 16 characters".to_string());
        }
        if version.is_empty() {
            return Err("version specifier is empty".to_string());
        }
        version.chars().collect::<Vec<_>>()
    } else {
        Alphanumeric
            .sample_iter(rng())
            .map(char::from)
            .take(8)
            .collect::<Vec<_>>()
    };
    version.resize(16, 0 as char);
    Ok(version.iter().collect())
}

pub fn get_verification(verification: &str) -> Result<u8, String> {
    match verification.to_lowercase().as_str() {
        "none" => Ok(0),
        "default" => Ok(1),
        "existence" => Ok(1),
        "checksum" => Ok(2),
        _ => Err(format!("not a valid verification option: {}", verification)),
    }
}

pub fn get_show_information(show_information: &str) -> Result<u8, String> {
    match show_information.to_lowercase().as_str() {
        "none" => Ok(0),
        "default" => Ok(1),
        "title" => Ok(1),
        "verbose" => Ok(2),
        _ => Err(format!(
            "not a valid information details option: {}",
            show_information
        )),
    }
}

pub fn get_show_console(show_console: &str, runner_name: &str) -> Result<u8, String> {
    match show_console.to_lowercase().as_str() {
        "auto" => {
            if runner_name.contains("windows") {
                Ok(0)
            } else {
                Ok(1)
            }
        }
        "never" => Ok(0),
        "always" => Ok(1),
        "attach" => Ok(2),
        _ => Err(format!("not a valid console option: {}", show_console)),
    }
}

pub fn get_current_dir(current_dir: &str) -> Result<u8, String> {
    match current_dir.to_lowercase().as_str() {
        "inherit" => Ok(0),
        "unpack" => Ok(1),
        "runner" => Ok(2),
        "command" => Ok(3),
        _ => Err(format!(
            "not a valid current directory option: {}",
            current_dir
        )),
    }
}

pub fn get_icon_path(icon: Option<&Path>) -> Result<Option<PathBuf>, String> {
    let icon = match icon {
        Some(icon) => icon,
        None => return Ok(None),
    };
    let icon = Path::new(&std::env::current_dir().map_err(|e| e.to_string())?).join(icon);
    let icon = std::fs::canonicalize(&icon).map_err(|_| {
        format!("icon file does not exist: {}", icon.display())
    })?;
    if !icon.is_file() {
        return Err(format!("icon path is not a file: {}", icon.display()));
    }
    Ok(Some(icon))
}

pub fn get_source(source: &Path) -> Result<PathBuf, String> {
    let source = Path::new(&std::env::current_dir().map_err(|e| e.to_string())?).join(source);
    let source = std::fs::canonicalize(&source)
        .map_err(|_| format!("input path does not exist: {}", source.display()))?;
    if !source.is_dir() && !source.is_file() {
        return Err(format!(
            "input path is not a file or directory: {}",
            source.display()
        ));
    }
    Ok(source)
}

pub fn get_output(output: Option<&Path>, command_path: &Path) -> Result<PathBuf, String> {
    let output = output
        .map(|path| path.as_os_str().to_owned())
        .unwrap_or_else(|| {
            let name = command_path.file_name().unwrap();
            let mut prefix = OsString::from("packed-");
            prefix.push(name);
            prefix
        });
    let output = Path::new(&std::env::current_dir().map_err(|e| e.to_string())?).join(output);
    if !output.parent().map(|path| path.is_dir()).unwrap_or(false) {
        return Err(format!(
            "output path has no parent directory: {}",
            output.parent().unwrap().display()
        ));
    }
    if output.is_dir() {
        return Err(format!("output path is a directory: {}", output.display()));
    }
    let parent = std::fs::canonicalize(output.parent().unwrap())
        .map_err(|_| format!("output path is invalid: {}", output.display()))?;
    Ok(parent.join(output.file_name().unwrap()))
}

pub fn get_unpack_directory(
    directory: Option<&str>,
    source: &Path,
) -> Result<[u8; NAME_SIZE], String> {
    let directory = if let Some(directory) = directory {
        directory.as_bytes().to_vec()
    } else {
        source
            .file_name()
            .ok_or_else(|| "couldn't infer unpack directory name from the input directory".to_string())?
            .to_string_lossy()
            .as_bytes()
            .to_vec()
    };
    if directory.len() >= NAME_SIZE {
        return Err("unpack directory name is longer than 127 characters".to_string());
    }
    let mut _directory = [0; NAME_SIZE];
    _directory[0..directory.len()].copy_from_slice(&directory);
    Ok(_directory)
}

pub fn get_command_path(command: Option<&Path>, source: &Path) -> Result<PathBuf, String> {
    if command.is_none() {
        if source.is_file() {
            return Ok(source.to_owned());
        } else {
            return Err("command must be specified when source is not a file".to_string());
        }
    }
    let command = command.unwrap();
    let source = if source.is_file() {
        source.parent().ok_or_else(|| "source path has no parent".to_string())?
    } else {
        source
    };
    let command = match std::fs::canonicalize(source.join(command)) {
        Err(_) => std::fs::canonicalize(
            Path::new(&std::env::current_dir().map_err(|e| e.to_string())?).join(command),
        ),
        command => command,
    }
    .map_err(|e| format!("command path is invalid: {}", e))?;
    if !command.is_file() {
        return Err("command path is not a file".to_string());
    }
    let command = if source.is_dir() {
        command.strip_prefix(source).map_err(|_| {
            "command path is not contained in the source directory".to_string()
        })?
    } else {
        command.strip_prefix(source).map_err(|_| {
            "command path is not contained in the source directory".to_string()
        })?
    };
    Ok(command.to_owned())
}

pub fn get_command(command_path: &Path) -> Result<[u8; NAME_SIZE], String> {
    let command = command_path
        .to_string_lossy()
        .as_bytes()
        .to_vec();
    if command.len() >= NAME_SIZE {
        return Err("command path is longer than 127 characters".to_string());
    }
    let mut _command = [0; NAME_SIZE];
    _command[0..command.len()].copy_from_slice(&command);
    Ok(_command)
}

pub fn get_env_vars(env: &[String]) -> Result<[u8; ARGS_SIZE], String> {
    let env = env
        .iter()
        .map(|v| {
            let (key, val) = v.split_once('=').ok_or_else(|| {
                "environment variable is not in the form of KEY=VALUE".to_string()
            })?;
            if key.is_empty() {
                return Err("environment variable key is empty".to_string());
            }
            Ok([key.trim(), val.trim()].join("\u{1e}"))
        })
        .collect::<Result<Vec<_>, String>>()?;
    let env = env.join("\u{1f}");
    let env = env.as_bytes();
    if env.len() >= ARGS_SIZE {
        return Err("environment variables list is longer than 511 characters".to_string());
    }
    let mut _env = [0; ARGS_SIZE];
    _env[0..env.len()].copy_from_slice(env);
    Ok(_env)
}

pub fn get_arguments(arguments: &[String]) -> Result<[u8; ARGS_SIZE], String> {
    let arguments = arguments.join("\u{1f}");
    let arguments = arguments.as_bytes();
    if arguments.len() >= ARGS_SIZE {
        return Err("arguments list is longer than 511 characters".to_string());
    }
    let mut _arguments = [0; ARGS_SIZE];
    _arguments[0..arguments.len()].copy_from_slice(arguments);
    Ok(_arguments)
}

/// Read a version string from a file (first line, trimmed, max 16 chars)
pub fn get_version_from_file(path: &Path) -> Result<String, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("couldn't read version file {}: {}", path.display(), e))?;
    let version = content.lines().next().unwrap_or("").trim();
    if version.is_empty() {
        return Err("version file is empty".to_string());
    }
    Ok(version.chars().take(16).collect())
}
