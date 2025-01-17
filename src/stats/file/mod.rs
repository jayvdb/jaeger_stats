//!  Write the statistics to a JSON file and read them back in memory
//!
mod bincode;
mod bson;
mod json;
mod operation_stats_json;

use super::StatsRec;

pub use operation_stats_json::{OperationStatsJson, StatsRecJson};

/// write the (complete) statistics to either '.json'  or '.bincode'
pub fn write_stats(file_name: &str, stats: StatsRec, ext: &str) {
    let file_name = file_name.replace(".csv", &format!(".{ext}"));
    match ext {
        "bson" => bson::dump_file(&file_name, stats),
        "json" => json::dump_file(&file_name, stats),
        "bincode" => bincode::dump_file(&file_name, stats),
        unknown => panic!("Unknown output format: '{unknown}'"),
    }
}
