use super::super::{
    rate::calc_rate,
    stats::{format_float, format_float_opt},
};
use super::{
    call::{Call, CallChain, CallDirection},
    file::{call_chain_key, LEAF_LABEL},
};
use crate::{
    aux::{report, Chapter},
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
}

/// Key for the CChain containing part of the CChain-values
#[derive(Hash, Eq, PartialEq, Debug, PartialOrd, Ord, Serialize, Deserialize)]
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

    /// Get the (external) end-point which is the start this call-chain
    pub fn get_leaf(&self) -> String {
        self.call_chain
            .last()
            .expect("Call chain is epmty!")
            .get_process_method()
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
    pub fn new() -> Self {
        Default::default()
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
        let min_millis =
            *self.duration_micros.iter().min().expect("Not an integer") as f64 / 1000 as f64;
        let avg_millis = self.duration_micros.iter().sum::<i64>() as f64
            / (1000 as f64 * self.duration_micros.len() as f64);
        let max_millis =
            *self.duration_micros.iter().max().expect("Not an integer") as f64 / 1000 as f64;
        let caching_process = &ps_key.caching_process;
        let percentage = self.count as f64 / n;
        let rate = if let Some((avg_rate, _)) = calc_rate(&self.start_dt_micros, num_files) {
            Some(avg_rate)
        } else {
            None
        };
        let expect_duration = percentage * avg_millis;
        let expect_contribution = if ps_key.is_leaf { expect_duration } else { 0.0 };
        let call_chain = ps_key.call_chain_key();
        let cc_hash = string_hash(&call_chain);
        let end_point = ps_key.get_endpoint();
        let leaf = ps_key.get_leaf();

        // Call_chain; cc_hash; End_point; Process/operation; Is_leaf; Depth; Count; Looped; Revisit; Caching_proces; min_millis; avg_millis; max_millis; freq.; expect_duration; expect_contribution;

        let line = format!("{call_chain};{cc_hash}; {end_point}; {leaf}; {}; {}; {}; {}; {:?}; {caching_process}; {}; {}; {}; {}; {}; {}; {};", 
            ps_key.is_leaf, self.depth, self.count, self.looped.len()> 0, self.looped,
            format_float(min_millis), format_float(avg_millis), format_float(max_millis),
            format_float(percentage), format_float_opt(rate), format_float(expect_duration), format_float(expect_contribution));
        line
    }
}

/// the information is distributed over the key and the value (no duplication in value)
pub type CChainStats = HashMap<CChainStatsKey, CChainStatsValue>;
//#[derive(Default, Debug, Serialize, Deserialize)]
//pub struct CChainStats (pub HashMap<CChainStatsKey, CChainStatsValue>);
