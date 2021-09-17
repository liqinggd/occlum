use crate::error::{
    COPY_DIR_ERROR, COPY_FILE_ERROR, CREATE_DIR_ERROR, CREATE_SYMLINK_ERROR, FILE_NOT_EXISTS_ERROR,
    INCORRECT_HASH_ERROR,
};
use data_encoding::HEXUPPER;
use regex::Regex;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::process::{Command, Output};
use std::vec;

lazy_static! {
    /// This map stores the path of occlum-modified loaders.
    /// The `key` is the name of the loader. The `value` is the loader path.
    /// We read the loaders from the `LOADER_CONFIG_FILE`
    static ref OCCLUM_LOADERS: HashMap<String, String> = {
        const LOADER_CONFIG_FILE: &'static str = "/opt/occlum/etc/template/occlum_elf_loader.config";
        let mut m = HashMap::new();
        let config_path = PathBuf::from(LOADER_CONFIG_FILE);
        if !config_path.is_file() {
            // if no given config file is found, we will use the default loader in elf headers
            warn!("fail to find loader config file {}. No loader is set!", LOADER_CONFIG_FILE);
        } else {
            let file_content = std::fs::read_to_string(config_path).unwrap();
            for line in file_content.lines() {
                let full_path = line.trim();
                if full_path.len() <= 0 {
                    continue;
                }
                let loader_path = PathBuf::from(full_path);
                let loader_file_name = loader_path.file_name().unwrap().to_string_lossy().to_string();
                m.insert(loader_file_name, full_path.to_string());
            }
        }
        debug!("occlum elf loaders: {:?}", m);
        m
    };
}

// pattern used to extract dependencies from ldd result
lazy_static! {
    /// pattern: name => path
    /// example: libc.so.6 => /lib/x86_64-linux-gnu/libc.so.6
    static ref DEPENDENCY_REGEX: Regex = Regex::new(r"^(?P<name>\S+) => (?P<path>\S+) ").unwrap();
}

/// convert a dest path(usually absolute) to a dest path in root directory
pub fn dest_in_root(root_dir: &str, dest: &str) -> PathBuf {
    let root_path = PathBuf::from(root_dir);
    let dest_path = PathBuf::from(dest);
    let dest_relative = if dest_path.is_absolute() {
        PathBuf::from(dest_path.strip_prefix("/").unwrap())
    } else {
        dest_path
    };
    return root_path.join(dest_relative);
}

/// check if hash of the file is equal to the passed hash value.
pub fn check_file_hash(filename: &str, hash: &str) {
    let file_hash = calculate_file_hash(filename);
    if file_hash != hash.to_string() {
        error!(
            "The hash value of {} should be {:?}. Please correct it.",
            filename, file_hash
        );
        std::process::exit(INCORRECT_HASH_ERROR);
    }
}

/// Use sha256 to calculate hash for file content. The returned hash is a hex-encoded string.
pub fn calculate_file_hash(filename: &str) -> String {
    let mut file = std::fs::File::open(filename).unwrap_or_else(|e| {
        println!("can not open file {}. {}", filename, e);
        std::process::exit(FILE_NOT_EXISTS_ERROR);
    });
    let mut hasher = Sha256::new();
    std::io::copy(&mut file, &mut hasher).unwrap();
    let hash = hasher.finalize();
    let hash = HEXUPPER.encode(&hash);
    hash
}

/// This is the main function of finding dependent shared objects for an elf file.
/// Currently, we only support dependent shared objects with absolute path.
/// This function works in such a process.
/// It will first analyze the dynamic loader of the file if it has a dynamic loader,
/// which means the file is an elf file. Then, we will use the loader defined in *OCCLUM_LOADERS*
/// to replace the original loader. The modified loader will find dependencies for occlum.
/// We will use the dynamic loader to analyze the dependencies. We run the dynamic loader in command line
/// and analyze the stdout. We use regex to match the pattern of the loader output.
/// The loader will automatically find all dependencies recursively, i.e., it will also find dependencies
/// for each shared object, so we only need to analyze the top elf file.
pub fn find_dependent_shared_objects(file_path: &str) -> HashSet<(String, String)> {
    let mut shared_objects = HashSet::new();
    // find dependencies for the input file
    // first, we find the dynamic loader for the elf file, if we can't find the loader, return empty shared objects
    let dynamic_loader = auto_dynamic_loader(file_path);
    if dynamic_loader.is_none() {
        return shared_objects;
    }
    let (dynamic_loader_src, dynamic_loader_dest) = dynamic_loader.unwrap();
    shared_objects.insert((dynamic_loader_src.clone(), dynamic_loader_dest));
    let output = command_output_of_executing_dynamic_loader(&file_path, &dynamic_loader_src);
    if let Ok(output) = output {
        let mut objects = extract_dependencies_from_output(&file_path, output);
        for item in objects.drain() {
            shared_objects.insert(item);
        }
    }
    shared_objects
}

/// get the output of the given dynamic loader.
/// This function will use the dynamic loader to analyze the dependencies of an elf file
/// and return the command line output of the dynamic loader.
fn command_output_of_executing_dynamic_loader(
    file_path: &str,
    dynamic_loader: &str,
) -> Result<Output, std::io::Error> {
    // if the file path has only filename, we need to add a "." directory
    let file_path_buf = PathBuf::from(file_path);
    let file_path = if file_path_buf.parent() == None {
        PathBuf::from(".")
            .join(&file_path_buf)
            .to_string_lossy()
            .to_string()
    } else {
        file_path_buf.to_string_lossy().to_string()
    };
    // return the output of the command to analyze dependencies
    debug!("{} --list {}", dynamic_loader, file_path);
    Command::new(dynamic_loader)
        .arg("--list")
        .arg(file_path)
        .output()
}

/// This function will try to find a dynamic loader for a elf file automatically
/// If we find the loader, we will return Some((loader_src, loader_dest)).
/// This is because the loader_src and loader_dest may not be the same directory.
/// If we can't find the loader, this function will return None
fn auto_dynamic_loader(filename: &str) -> Option<(String, String)> {
    let elf_file = match elf::File::open_path(filename) {
        Err(_) => return None,
        Ok(elf_file) => elf_file,
    };
    let interp_scan = match elf_file.get_section(".interp") {
        None => return None,
        Some(section) => section,
    };
    let interp_data = String::from_utf8_lossy(&interp_scan.data).to_string();
    let inlined_elf_loader = interp_data.trim_end_matches("\u{0}"); // this interp_data always with a \u{0} at end
    debug!("the loader of {} is {}.", filename, inlined_elf_loader);
    let inlined_elf_loader_path = PathBuf::from(inlined_elf_loader);
    let loader_file_name = inlined_elf_loader_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap();
    // If the loader file name is glibc loader or musl loader, we will use occlum-modified loader
    let occlum_elf_loader = OCCLUM_LOADERS
        .get(loader_file_name)
        .cloned()
        .unwrap_or(inlined_elf_loader.to_string());
    Some((
        occlum_elf_loader.to_string(),
        inlined_elf_loader.to_string(),
    ))
}

/// resolve the results of dynamic loader to extract dependencies
pub fn extract_dependencies_from_output(
    file_path: &str,
    output: Output,
) -> HashSet<(String, String)> {
    let mut shared_objects = HashSet::new();
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    debug!("The loader output of {}:\n {}", file_path, stdout);
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    // audodep may output error message. We should return this message to user for further checking.
    if stderr.trim().len() > 0 {
        error!("cannot autodep for {}. {}", file_path, stderr);
    }
    for line in stdout.lines() {
        let line = line.trim();
        let captures = DEPENDENCY_REGEX.captures(line);
        if let Some(captures) = captures {
            let raw_path = (&captures["path"]).to_string();
            if let Some(absolute_path) = convert_to_absolute(file_path, &raw_path) {
                shared_objects.insert((absolute_path.clone(), absolute_path.clone()));
                let raw_name = (&captures["name"]).to_string();
                let raw_name_path = PathBuf::from(&raw_name);
                if raw_name_path.is_absolute() {
                    shared_objects.insert((absolute_path, raw_name));
                }
            }
        }
    }
    debug!("find objects: {:?}", shared_objects);
    shared_objects
}

/// convert the raw path to an absolute path.
/// The raw_path may be an absolute path itself, or a relative path relative to some file
/// If the conversion succeeds, return Some(converted_absolute_path)
/// otherwise, return None
pub fn convert_to_absolute(file_path: &str, raw_path: &str) -> Option<String> {
    let raw_path = PathBuf::from(raw_path);
    // if raw path is absolute, return
    if raw_path.is_absolute() {
        return Some(raw_path.to_string_lossy().to_string());
    }
    // if the given relative path can be converted to an absolute path , return
    let converted_path = resolve_relative_path(file_path, &raw_path.to_string_lossy());
    let converted_path = PathBuf::from(converted_path);
    if converted_path.is_absolute() {
        return Some(converted_path.to_string_lossy().to_string());
    }
    // return None
    return None;
}

/// convert `a path relative to file` to the real path in file system
pub fn resolve_relative_path(filename: &str, relative_path: &str) -> String {
    let file_path = PathBuf::from(filename);
    let file_dir_path = file_path
        .parent()
        .map_or(PathBuf::from("."), |p| PathBuf::from(p));
    let resolved_path = file_dir_path.join(relative_path);
    resolved_path.to_string_lossy().to_string()
}

/// find an included file in the file system. If we can find the bom file, return the path
/// otherwise, the process exit with error
/// if included dir is relative path, if will be viewed as path relative to the `current` path (where we execute command)
pub fn find_included_bom_file(
    included_file: &str,
    bom_file: &str,
    included_dirs: &Vec<String>,
) -> String {
    let bom_file_path = PathBuf::from(bom_file);
    let bom_file_dir_path = bom_file_path
        .parent()
        .map_or(PathBuf::from("."), |p| p.to_path_buf());
    // first, we find the included bom file in the current dir of the bom file
    let included_file_path = bom_file_dir_path.join(included_file);
    if included_file_path.is_file() {
        return included_file_path.to_string_lossy().to_string();
    }
    // Then, we find the bom file in each included dir.
    for included_dir in included_dirs {
        let included_dir_path = std::env::current_dir().unwrap().join(included_dir);
        let included_file_path = included_dir_path.join(included_file);
        if included_file_path.is_file() {
            return included_file_path.to_string_lossy().to_string();
        }
    }
    // fail to find the bom file
    error!(
        "cannot find included bom file {} in {}.",
        included_file, bom_file
    );
    std::process::exit(FILE_NOT_EXISTS_ERROR);
}

/// Try to resolve a path may contain environmental variables to a path without environmental variables
/// This function relies on a third-party crate shellexpand.
/// Known limitations: If the environmental variable points to an empty value, the conversion may fail.
pub fn resolve_envs(path: &str) -> String {
    shellexpand::env(path).map_or_else(
        |_| {
            warn!("{} resolve fails.", path);
            path.to_string()
        },
        |res| res.to_string(),
    )
}
