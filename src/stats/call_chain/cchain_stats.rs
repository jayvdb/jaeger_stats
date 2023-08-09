use super::{
    call::{Call, CallDirection},
    call_chain::CallChain,
    file::{call_chain_key, LEAF_LABEL},
};
use crate::{
    aux::{calc_rate, format_float, format_float_opt, report, Chapter, Counted, TimeStats},
    string_hash,
};
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, collections::HashMap, error::Error};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CChainStatsValue {
    pub count: usize,
    pub depth: usize,
    pub duration_micros: Vec<i64>,
    pub start_dt_micros: Vec<i64>, // represented via start_dt.timestamp_micros()
    pub looped: Vec<String>,
    pub rooted: bool, //does this call-chain originate from the root of this trace.
    pub cc_not_http_ok: i32, // count of the number of call chanis that has one of more HTTP-error(s) somewhere along the chain
    pub cc_with_error_log: i32, // count of the number of call chanis that has one of more ERROR log-lines somewhere along the chain
    pub http_not_ok: Counted<i16>,
    pub error_logs: Counted<String>,
}

/// Key for the CChain containing part of the CChain-values
#[derive(Hash, Eq, PartialEq, Debug, PartialOrd, Ord, Serialize, Deserialize, Clone)]
pub struct CChainStatsKey {
    pub call_chain: CallChain,
    pub caching_process: String, // an empty string or a one or more caching-processes between square brackets
    pub is_leaf: bool,
}

impl CChainStatsKey {
    /// get the method of the current call (last call in the call-chain)
    pub fn get_method(&self) -> &str {
        &self.call_chain[self.call_chain.len() - 1].method
    }

    /// Extract a textual key that represents the full call-chain, including labels for caching_process and is_leaf
    pub fn call_chain_key(&self) -> String {
        call_chain_key(&self.call_chain, &self.caching_process, self.is_leaf)
    }

    /// Get the (external) end-point which is the start this call-chain
    pub fn get_endpoint(&self) -> String {
        self.call_chain
            .first()
            .expect("Call chain is epmty!")
            .get_process_method()
    }

    /// Get the leaf process/method which is located start this call-chain. This is also the identifier for the group at the level of 'stats'.
    pub fn get_leaf(&self) -> String {
        self.call_chain
            .last()
            .expect("Call chain is epmty!")
            .get_process_method()
    }

    /// Get the leaf process. This key is used to group processes in StatsRec.stats.
    pub fn get_leaf_process(&self) -> String {
        self.call_chain
            .last()
            .expect("Call chain is epmty!")
            .get_process()
    }

    /// parse a string generated by call_chain_key and reconstruct the full call chain.
    pub fn parse(cchain_str: &str) -> Result<Self, Box<dyn Error>> {
        let mut parts = cchain_str.split("&").map(|part| part.trim());
        let Some(cchain) = parts.next() else {
            Err("Provided line is empty")?
        };
        let caching_process = match parts.next() {
            Some(s) => s.to_owned(),
            None => "".to_owned(),
        };
        let is_leaf = match parts.next() {
            Some(s) => match s {
                LEAF_LABEL => true,
                "" => false,
                s => panic!("Expected {LEAF_LABEL} or empty string. Found {s}"),
            },
            None => false,
        };

        let call_chain = cchain
            .split("|")
            .map(|s| {
                let Some((proc, meth_dir)) = s.trim().split_once("/") else {
                        panic!("Failed to unpack '{s}' in a process/operation pair.");
                    };
                let (meth, call_direction) = match meth_dir.split_once("[") {
                    Some((meth, dir)) => {
                        let dir = &dir[0..(dir.len() - 1)];
                        (meth, dir.into())
                    }
                    None => (meth_dir, CallDirection::Unknown),
                };
                Call {
                    process: proc.trim().to_owned(),
                    method: meth.trim().to_owned(),
                    call_direction,
                }
            })
            .collect();
        Ok(Self {
            call_chain,
            caching_process,
            is_leaf,
        })
    }

    /// try to remap a non-rooted call-chain based on expected call chains and return whether the remapping succeeded.
    pub fn remap_callchain(&mut self, expected_cc: &Vec<CChainStatsKey>) -> bool {
        let cc_len = self.call_chain.len();
        let matches: Vec<_> = expected_cc
            .iter()
            .filter(|ecc| {
                let ecc_len = ecc.call_chain.len();
                if cc_len > ecc_len {
                    false // the chain is longer than the expected chain currently under investigation
                } else {
                    let ecc_iter = ecc.call_chain.iter().skip(ecc_len - cc_len);
                    // compare the call-chains and only return true when these are equal (other fields, such as 'is_leaf' can still differ)
                    self.call_chain.iter().cmp(ecc_iter) == Ordering::Equal
                }
            })
            .collect();
        let the_match = match matches.len() {
            0 => None,
            1 => Some(matches[0]),
            2 => {
                if matches[0].is_leaf == self.is_leaf {
                    Some(matches[0])
                } else {
                    Some(matches[1])
                }
            }
            n => {
                report(
                    Chapter::Details,
                    format!(
                        "NO FIX: {n} matches found for non-rooted '{:?}'",
                        self.call_chain
                    ),
                );
                None
            }
        };
        if let Some(the_match) = the_match {
            self.is_leaf = the_match.is_leaf;
            self.call_chain = the_match.call_chain.clone();
            true
        } else {
            false
        }
    }
}

impl CChainStatsValue {
    pub fn new(depth: usize, looped: Vec<String>, rooted: bool) -> Self {
        Self {
            depth,
            looped,
            rooted,
            ..Default::default()
        }
    }

    pub fn get_min_millis_str(&self) -> String {
        TimeStats(&self.duration_micros).get_min_millis_str()
    }

    pub fn get_avg_millis(&self) -> f64 {
        TimeStats(&self.duration_micros).get_avg_millis()
    }

    pub fn get_avg_millis_str(&self) -> String {
        TimeStats(&self.duration_micros).get_avg_millis_str()
    }

    pub fn get_max_millis_str(&self) -> String {
        TimeStats(&self.duration_micros).get_max_millis_str()
    }

    pub fn get_avg_rate_str(&self, num_files: i32) -> String {
        TimeStats(&self.start_dt_micros).get_avg_rate_str(num_files)
    }

    pub fn get_median_rate_str(&self, num_files: i32) -> String {
        TimeStats(&self.start_dt_micros).get_median_rate_str(num_files)
    }

    pub fn get_frac_not_http_ok_str(&self) -> String {
        format_float(self.cc_not_http_ok as f64 / self.count as f64)
    }

    pub fn get_frac_error_log_str(&self) -> String {
        format_float(self.cc_with_error_log as f64 / self.count as f64)
    }

    /// reports the statistics for a single line
    pub fn report_stats_line(
        &self,
        process_key: &str,
        ps_key: &CChainStatsKey,
        n: f64,
        num_files: i32,
    ) -> String {
        assert_eq!(
            process_key,
            ps_key
                .call_chain
                .last()
                .expect("Call chain is empty!")
                .process
        );
        let caching_process = &ps_key.caching_process;
        let percentage = self.count as f64 / n;
        let expect_duration = percentage * self.get_avg_millis();
        let expect_contribution = if ps_key.is_leaf { expect_duration } else { 0.0 };
        let call_chain = ps_key.call_chain_key();
        let cc_hash = string_hash(&call_chain);
        let end_point = ps_key.get_endpoint();
        let leaf = ps_key.get_leaf();

        // Call_chain; cc_hash; End_point; Process/operation; Is_leaf; Depth; Count; Looped; Revisit; Caching_proces; min_millis; avg_millis; max_millis; freq.; expect_duration; expect_contribution;

        let line = format!("{call_chain};{cc_hash}; {end_point}; {leaf}; {}; {}; {}; {}; {:?}; {caching_process}; {}; {}; {}; {}; {}; {}; {}; {}; {}", 
            ps_key.is_leaf, self.depth, self.count, self.looped.len()> 0, self.looped,
            self.get_min_millis_str(), self.get_avg_millis_str(), self.get_max_millis_str(),
            format_float(percentage), self.get_avg_rate_str(num_files), format_float(expect_duration), format_float(expect_contribution),
            self.get_frac_not_http_ok_str(), self.get_frac_error_log_str()
        );
        line
    }
}

/// the information is distributed over the key and the value (no duplication in value)
pub type CChainStats = HashMap<CChainStatsKey, CChainStatsValue>;
//#[derive(Default, Debug, Serialize, Deserialize)]
//pub struct CChainStats (pub HashMap<CChainStatsKey, CChainStatsValue>);
