use super::{
    service_oper_graph::{Position, ServiceOperGraph, ServiceOperationType},
    trace_data::TraceData,
    trace_forrest::EmbeddingKind,
    tt_utils::{
        get_call_chain_prefix, mark_selected_call_chain, split_service, split_service_operation,
    },
    MermaidScope,
};
use crate::{
    mermaid::trace_forrest::TraceForrest,
    stats::{CChainStatsKey, LeafService},
    EdgeValue,
};
use regex;
use std::collections::HashMap;

/// The trace-tree is used to build the Mermaid-charts.
/// This datastructure is derived from either:
///    * a Stiched dataset (as generated by stitch)
///    * A StatsRec (as generated by trace_analysis snapshot)
/// This trace-tree will only contain the call-chains and does not extract the process-oper statistics, as that data is not needed (would be a duplicate).
///
/// NOTE: this structure contains a collection of paths, but is not yet a TraceTree as it does not merge joined prefixes of paths to a single prefix (real tree structure)   
pub struct TracePaths(pub HashMap<LeafService, Vec<TraceData>>);

impl TracePaths {
    fn build_trace_forrest(&self, service_oper: &str) -> TraceForrest {
        let (service, oper_opt) = split_service_operation(service_oper);
        // find all paths that end in this 'service' and build a trace-tree out of it (filtered on the 'operation')
        match self.0.get(service) {
            Some(paths) => {
                let paths = if let Some(oper) = oper_opt {
                    // next deref is needed otherwise we get the wrong type
                    paths
                        .iter()
                        .filter(|td| td.trace_path.get_operation() == oper)
                        .collect::<Vec<_>>()
                } else {
                    // no operation defined so return all paths.
                    paths.iter().collect()
                };
                TraceForrest::build_trace_forrest(paths)
            }
            None => panic!("Failure to find the paths that terminate in service '{service}'."),
        }
    }

    /// Build the ServiceOperationGraph based on the TraceTree (Stiched or StatsRec data) and for the selected 'service_oper'.
    /// The input 'data' is a dataset of stitched data-point containings all traces though the graph and the statistics for the endpoint of this trace.
    /// In this function we reconstruct the original graph by taking the last step of each of the traces that pass through or end in 'service_oper'.
    /// The statistic collected is the average number of traces that pass through a node.
    /// Some nodes are reachable via multiple paths, in that case the sum is used to aggegate the counts.
    ///
    /// This is a two stage-process.
    /// 1. find all paths in 'data' that end in 'service_oper' and construct a traceForrest (filter for all paths that reach and pass 'service_oper'.)
    /// 2. Build the ServiceOperGraph for all paths that are 'embedded' or 'extend' the TraceForrest generated in step 1.
    fn build_serv_oper_graph(&self, service_oper: &str) -> ServiceOperGraph {
        let trace_forrest = self.build_trace_forrest(service_oper);

        //println!("{trace_forrest:#?}");
        let service = split_service(service_oper);
        // Stage-1: build the downstream graphs and collect the set of incoming paths via the counted_prefix
        let sog = self.0
            .iter()
            .flat_map(|(_k, ccd_vec)| {
                // _k is final service/operation of the call-chain, which is not needed here
                ccd_vec
                    .iter()
                    .map(|ccd| (trace_forrest.embedding(&ccd.trace_path.call_chain), ccd))
                    .filter(|(embedding, cck)| *embedding != EmbeddingKind::None)
                    .filter_map(|(embedding, ccd)| {
                        let call_chain = &ccd.trace_path.call_chain;
                        if call_chain.len() >= 2 {
                            let skip = call_chain.len() - 2;
                            let mut cc = call_chain.into_iter().skip(skip);
                            let from = cc.next().unwrap();
                            let to = cc.next().unwrap();

                            // and result as tuple for to be folded
                            Some((embedding, from, to, &ccd.data))
                        } else {
                            println!(
                                "Skipping call-chain as it is consists of a single step '{}' (no link)",
                                ccd.full_key
                            );
                            None
                        }
                    })
            })
            .fold(
                ServiceOperGraph::new(),
                |mut sog, (embedding, from, to, edge_data)| {
                    // add the connection to the graph
                    let default_pos = match embedding {
                        EmbeddingKind::Embedded => Position::Inbound,
                        EmbeddingKind::Extended => Position::Outbound,
                        EmbeddingKind::None => panic!("EmbeddingKind::None is not valid here.")
                    };
                    sog
                        // TODO: remove the clone operations (and fix downstream)
                        .add_connection(&from, &to, edge_data, service, default_pos);
                    sog
                },
            );

        sog
    }

    /// Mark downstream nodes as reachable and do a count of the number of paths reachable over current path up to 'service_oper' that is under investigation
    fn mark_and_count_downstream(
        &self,
        mut sog: ServiceOperGraph,
        service_oper: &str,
        call_chain_key: &str,
    ) -> ServiceOperGraph {
        let prefix = get_call_chain_prefix(service_oper, call_chain_key);

        self.0.iter().for_each(|(_k, ccd_vec)| {
            // _k is final service/operation of the call-chain and this is not relevant
            ccd_vec
                .iter()
                .filter(|ccd| ccd.full_key.starts_with(&prefix))
                .for_each(|ccd| {
                    let cc = CChainStatsKey::parse(&ccd.full_key).unwrap();
                    if cc.call_chain.len() >= 2 {
                        let skip = cc.call_chain.len() - 2;
                        let mut cc = cc.call_chain.into_iter().skip(skip);
                        let from = cc.next().unwrap();
                        let to = cc.next().unwrap();

                        // and update the reach_count
                        sog.update_inbound_path_count(&from, &to, &ccd.data);
                    } else {
                        println!(
                            "Skipping call-chain as it is consists of a single step '{}' (no link)",
                            ccd.full_key
                        );
                    }
                });
        });
        //TODO: to implement
        sog
    }

    /// Build a diagram for the 'service_oper'  and 'call_chain_key' based on the stitched 'data'.
    pub fn get_diagram(
        &self,
        service_oper: &str,
        call_chain_key: Option<&str>,
        edge_value: EdgeValue,
        scope: MermaidScope,
        compact: bool,
    ) -> String {
        let sog = self.build_serv_oper_graph(service_oper);

        // If a callchain-key is specified we mark the this call-chain and add the additional statistics.
        let mut sog = if let Some(call_chain_key) = call_chain_key {
            let sog = self.mark_and_count_downstream(sog, service_oper, call_chain_key);

            // Emphasize the selected path if the call_chain-key is provided
            mark_selected_call_chain(sog, call_chain_key)
        } else {
            sog
        };

        sog.update_service_operation_type(service_oper, ServiceOperationType::Emphasized);
        sog.mermaid_diagram(scope, compact, edge_value)
    }
}
