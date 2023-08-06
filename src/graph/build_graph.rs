use super::{process_node::ProcessNodes, fix_callchain::fix_call_chain};
use crate::stats::json::StatsRecJson;

pub fn build_graph(srj: &StatsRecJson) -> ProcessNodes {
    let mut process_nodes = ProcessNodes::new();

    srj.stats.iter().for_each(|(_root_process_key, stat)| {
        stat.call_chain.iter().for_each(|(cc_key, cc_val)| {
            let count = cc_val.count.try_into().unwrap();
            let fixed_cc = fix_call_chain(&cc_key.call_chain);
            // ProcessNodes::tmp_check_cc(
            //     &fixed_cc,
            //     cc_key.is_leaf,
            //     cc_val.rooted,
            //     &cc_val.looped,
            // );
            process_nodes.add_call_chain(&fixed_cc, count);
//            process_nodes.add_call_chain(&cc_key.call_chain, count);
        })
    });

    process_nodes
}