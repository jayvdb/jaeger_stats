use bson;
use super::super::StatsRec;
use std::{fs, io};
use super::StatsRecJson;


pub fn dump_file(file_name: &str, stats: StatsRec) {
    let srj: StatsRecJson = stats.into();
    println!("input:\n{srj:#?}");
//    let buf = bson::to_vec(&srj).unwrap();
// match fs::write(file_name, buf) {
//     Ok(()) => (),
//     Err(err) => panic!("failed to Serialize !!\n\tError={err:?}"),
// }

    let options = bson::SerializerOptions::builder().human_readable(false).build();
    let doc = bson::to_document_with_options(&srj, options).unwrap();
    let f = fs::File::create(file_name).expect("Failed to open file");
    let writer = io::BufWriter::new(f);
    match doc.to_writer(writer) {
        Ok(()) => (),
        Err(err) => panic!("failed to Serialize !!\n\tError={err:?}")
    }
}

