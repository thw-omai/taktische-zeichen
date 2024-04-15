use std::{fs, fs::File, io::{
    self,
    Read
}, path::{Component, Path, PathBuf}, time::Duration};
use base64::{
    Engine,
    engine::general_purpose
};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use sha2::{Digest, Sha256};
use walkdir::WalkDir;

pub(crate) fn calc_hash(file_name: &str) -> String {
    let mut file = File::open(file_name).unwrap();
    let mut hasher = Sha256::new();
    io::copy(&mut file, &mut hasher).expect("TODO: panic message");
    let hash_bytes = hasher.finalize();
    let final_hash = format!("{:X}", hash_bytes);
    final_hash
}

pub(crate) fn save_to_file(file_name: &str, content: &str) {
    let parent = Path::new(file_name).parent().expect("ERROR during path traversal");
    if !parent.exists() {
        fs::create_dir_all(parent).expect("Unable to create directory");
    }
    fs::write(file_name, content).expect("Unable to write file");
}


pub(crate) fn create_progress_bar(
    spinner_style: &ProgressStyle,
    m: MultiProgress,
    prefix: &str,
    multi: bool,
) -> (ProgressBar, MultiProgress) {
    let pb;
    if multi {
        pb = m.add(ProgressBar::new_spinner());
    } else {
        pb = ProgressBar::new_spinner();
    }
    pb.set_style(spinner_style.clone());
    pb.set_prefix(format!("[{:>7}]", prefix));
    pb.enable_steady_tick(Duration::from_millis(100));
    (pb, m)
}


pub(crate) fn path_to_title(
    match_name: &str,
    path: PathBuf,
) -> String {
    let mut result = String::new();

    for component in path.components() {
        if let Component::Normal(name) = component {
            if name == match_name {
                result.clear();
            } else if name != "svg" {
                result.push_str(
                    name.to_str()
                        .unwrap_or("")
                        .replace("-", " ")
                        .as_str()
                );
                result.push(' ');
            }
        }
    }

    result
        .trim()
        .trim_end_matches(".svg")
        .to_string()
}

pub(crate) fn path_to_id(
    match_name: &str,
    path: &Path,
) -> String {
    let mut result = String::new();

    for component in path.components() {
        if let Component::Normal(name) = component {
            if name == match_name {
                result.clear();
            } else if name != "svg" {
                result.push_str(name.to_str().unwrap_or(""));
                result.push('-');
            }
        }
    }

    let mut result = result.trim_end_matches("-").to_string();
    if result.contains("original") {
        result = result.trim_start_matches("original-").to_string();
        result = format!("{}-original", result);
    } else if result.contains("inverted") {
        result = result.trim_start_matches("inverted-").to_string();
        result = format!("{}-inverted", result);
    }
    result
}

pub(crate) fn process_entries<F>(
    directory: &str,
    mut process_fn: F,
) where
    F: FnMut(PathBuf),
{
    map_entries(directory)
        .iter()
        .for_each(|item| process_fn((*item.clone()).to_path_buf()))
}

pub(crate) fn map_entries(
    directory: &str,
) -> Vec<PathBuf> {
    return WalkDir::new(directory)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter_map(|entry| {
            return if let Some(extension) = entry.path().extension() {
                if extension == "svg" {
                    Some(entry)
                } else {
                    None
                }
            } else { None };
        })
        .map(|entry| entry.path().to_path_buf())
        .collect();
}

pub(crate) fn file_to_base64(
    file_path: &str
) -> io::Result<String> {
    let mut file = File::open(file_path)?;

    let mut content = String::new();
    file.read_to_string(&mut content)?;

    Ok(general_purpose::STANDARD.encode(&content))
}


pub(crate) fn join_paths(
    paths: Vec<&str>,
) -> String {
    let mut string = paths
        .iter()
        .filter(|template| !template.is_empty())
        .map(|x| x.as_ref())
        .collect::<Vec<_>>()
        .join("/");
    string
        .push_str("/");
    string
}

pub(crate) fn join_filename(
    names: Vec<&str>,
) -> String {
    names
        .iter()
        .filter(|template| !template.is_empty())
        .map(|x| x.as_ref())
        .collect::<Vec<_>>()
        .join("-")
}

pub(crate) fn uppercase_first_letter(
    s: &str
) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}
