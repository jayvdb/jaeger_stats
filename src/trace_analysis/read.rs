//! Reading raw json-formatted Jaeger-traces from file
use crate::{
    aux::{report, Chapter},
    processed::{extract_traces, Trace},
    raw::read_jaeger_trace_file,
};
use std::{error::Error, ffi::OsStr, fs, path::Path};

/// read a single file and process it to get clean Tcaecs. Returns a set of traces, or an error
fn read_trace_file(input_file: &Path) -> Result<Vec<Trace>, Box<dyn Error>> {
    println!("Reading a Jaeger-trace from '{}'", input_file.display());
    let jt = read_jaeger_trace_file(input_file).unwrap();

    Ok(extract_traces(&jt))
}

fn read_trace_folder(folder: &Path) -> Result<(Vec<Trace>, i32), Box<dyn Error>> {
    let mut num_files = 0;
    let traces = fs::read_dir(folder)
        .expect("Failed to read directory")
        .into_iter()
        .filter_map(|entry| {
            let entry = entry.expect("Failed to extract file-entry");
            let path = entry.path();

            let metadata = fs::metadata(&path).unwrap();
            if metadata.is_file() {
                let file_name = path.to_str().expect("path-string").to_owned();
                if file_name.ends_with(".json") {
                    num_files += 1;
                    read_trace_file(&path).ok()
                } else {
                    println!("Ignore '{file_name} as it does not have suffix '.json'.");
                    None // Not .json file
                }
            } else {
                None // No file
            }
        })
        .flatten()
        .collect();
    Ok((traces, num_files))
}

///Check whether path is a file or folder and read all traces.
pub fn read_process_file_or_folder<'a>(path: &'a Path) -> (Vec<Trace>, i32, &'a Path) {
    report(
        Chapter::Summary,
        format!("Reading all traces from folder: {}", path.display()),
    );
    let (traces, num_files, folder) =
        if path.is_file() && path.extension() == Some(OsStr::new("json")) {
            let traces = read_trace_file(&path).unwrap();
            //let path = Path::new(input_file);
            (
                traces,
                1,
                path.parent()
                    .expect("Could not extract parent of input_file"),
            )
        } else if path.is_dir() {
            let (traces, num_files) = read_trace_folder(&path).unwrap();
            (traces, num_files, path)
        } else {
            panic!(
                " Expected file with extention '.json' or folder. Received: '{}' ",
                path.display()
            );
        };
    report(
        Chapter::Summary,
        format!(
            "Read {} traces in total from {} files.",
            traces.len(),
            num_files
        ),
    );

    (traces, num_files, folder)
}