// use crate::report::Chapter;
use std::{
    fs::File,
    io::{Write, BufWriter},
    sync::Mutex};


#[derive(Copy, Clone)]
pub enum Chapter {
    Summary = 0,
    Ingest,
    Analysis,
    Details
}

static CHAPTER_NAMES: [&str;4] = ["Summary", "Ingest", "Analysis", "Details"];

impl Chapter {
    fn discriminant(&self) -> usize {
        unsafe { *(self as *const Self as *const u8) as usize }
    }
}

static STORE: Mutex<Vec<Vec<String>>> = Mutex::new(Vec::new());


pub fn report(chapter: Chapter, msg: String) {
    let idx = chapter.discriminant();
    if idx == Chapter::Summary as usize {
        println!("{msg}");
    }

    {
        let mut guard = STORE.lock().unwrap();
        while guard.len() <= idx {
            println!("Pushed one more as length {}  < {idx}", guard.len());
            guard.push(Vec::new());
        }
        guard[idx].push(msg);
    
    }

}


pub fn write_report(path: &str) {
    let mut guard = STORE.lock().unwrap();
    let contents = (0..guard.len()).
        map(|idx| {
            format!("{}\n{}\n\n", CHAPTER_NAMES[idx], guard[idx].join("\n"))
        })
        .collect::<Vec<_>>()
        .join("\n");
    let mut f = File::create(path).expect("Failed to create report-file");
    f.write_all(contents.as_bytes()).expect("Failed to write to report.");

    // wipe the contents
    *guard = Vec::new();
}

