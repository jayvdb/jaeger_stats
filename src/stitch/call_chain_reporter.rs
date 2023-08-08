use crate::{
    stats::call_chain::CChainStatsKey,
    stats::{call_chain::CChainStatsValue, StatsRec},
};
use std::collections::HashSet;

type Processor = fn(&CChainStatsValue, i32, usize) -> String;

pub struct CCReportItem {
    label: &'static str,
    processor: Processor,
}

impl CCReportItem {
    pub fn new(label: &'static str, processor: Processor) -> Self {
        Self { label, processor }
    }
}

pub struct CallChainReporter<'a> {
    buffer: &'a mut Vec<String>,
    data: &'a Vec<Option<StatsRec>>,
    report_items: Vec<CCReportItem>,
}

impl<'a> CallChainReporter<'a> {
    pub fn new(
        buffer: &'a mut Vec<String>,
        data: &'a Vec<Option<StatsRec>>,
        report_items: Vec<CCReportItem>,
    ) -> Self {
        Self {
            buffer,
            data,
            report_items,
        }
    }

    // find a deduplicated set of all keys and sort them
    pub fn get_keys(&self) -> Vec<CChainStatsKey> {
        let mut keys = HashSet::new(); // Computing all possible keys over the different datasets that need to be stitched.
        self.data.iter().for_each(|stats_rec| {
            if let Some(stats_rec) = stats_rec {
                stats_rec.stats.iter().for_each(|(proc_key, st)| {
                    st.call_chain.iter().for_each(|(cc_key, _)| {
                        // checks
                        let cc_key_clone = cc_key.clone();
                        if *cc_key != cc_key_clone {
                            println!("Failed to clone for '{cc_key:#?}'.")
                        };
                        let process = cc_key_clone.get_leaf_process();
                        if *proc_key != process {
                            println!(
                                "Mismatch between '{proc_key}' and extracted proces '{process}'"
                            )
                        }

                        _ = keys.insert(cc_key_clone)
                    })
                })
            }
        });
        let mut keys: Vec<_> = keys.into_iter().collect();
        keys.sort();
        keys
    }

    pub fn append_report(&mut self, cc_key: CChainStatsKey) {
        let process = cc_key.get_leaf_process();

        let cc_stats: Vec<_> = self
            .data
            .iter()
            .map(|stats_rec| match stats_rec {
                Some(stats_rec) => match stats_rec.stats.get(&process) {
                    Some(st) => match st.call_chain.get(&cc_key) {
                        Some(oper) => Some((oper, stats_rec.num_files, stats_rec.trace_id.len())),
                        None => {
                            println!("No value found for key '{}'.", cc_key.call_chain_key());
                            None
                        }
                    },
                    None => {
                        println!("no process found for '{process}'.");
                        None
                    }
                },
                None => None,
            })
            .collect();

        let cc_key_str = cc_key.call_chain_key();
        self.buffer.push(format!("# statistics for {cc_key_str}"));

        // set_show_rate_output(&process_operation[..] == "bspc-productinzicht/GET");

        self.report_items
            .iter()
            .for_each(|CCReportItem { label, processor }| {
                let values = cc_stats
                    .iter()
                    .map(|ms| {
                        ms.map_or("".to_owned(), |msv_nf| {
                            processor(msv_nf.0, msv_nf.1, msv_nf.2)
                        })
                    })
                    .collect::<Vec<_>>()
                    .join("; ");
                self.buffer.push(format!("{cc_key_str}; {label}; {values}"));
            });
    }
}