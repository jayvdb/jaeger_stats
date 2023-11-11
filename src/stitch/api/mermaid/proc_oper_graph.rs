use crate::stats::call_chain::{Call, CallDirection};

struct Loc {
    proc_idx: usize,
    oper_idx: usize,
}

/// defines a position in the ProcessConnection, where the first index is the process and the second is the operation.
#[derive(Debug)]
struct CallDescriptor {
    to_proc: usize,
    to_oper: usize,
    count: f64,
}

impl CallDescriptor {
    fn new(loc: Loc, count: f64) -> Self {
        Self {
            to_proc: loc.proc_idx,
            to_oper: loc.oper_idx,
            count,
        }
    }
}

/// Used to store outbound
#[derive(Debug)]
pub struct Operation {
    pub oper: String,
    pub call_direction: CallDirection,
    calls: Vec<CallDescriptor>,
}

impl Operation {
    /// Insert a link to the CallDescriptor 'to', or update it if is exists by incrementing the count
    fn upsert_link(&mut self, to: CallDescriptor) {
        match self
            .calls
            .iter()
            .position(|call| call.to_oper == to.to_oper && call.to_proc == to.to_proc)
        {
            Some(idx) => self.calls[idx].count += to.count,
            None => self.calls.push(to),
        }
    }
}

#[derive(Debug)]
pub struct Process {
    pub proc: String,
    pub operations: Vec<Operation>,
}

impl Process {
    /// push an Operation to the process and return the index
    fn push_oper(&mut self, oper: String, call_direction: CallDirection) -> usize {
        let oper_idx = self.operations.len();
        self.operations.push(Operation {
            oper,
            call_direction,
            calls: Vec::new(),
        });
        oper_idx
    }

    /// add this process as a subgraph with a series of nodes
    fn add_nodes(&self, diagram: &mut Vec<String>) {
        diagram.push(format!("\tsubgraph {}", self.proc));
        self.operations.iter().for_each(|oper| {
            diagram.push(format!(
                "\t\t{}/{}([{}/{}])",
                self.proc, oper.oper, self.proc, oper.oper
            ))
        });
        diagram.push("\tend".to_string());
    }

    /// add this process as a subgraph with a series of nodes
    fn add_links(&self, diagram: &mut Vec<String>, pog: &ProcOperGraph) {
        self.operations.iter().for_each(|oper| {
            oper.calls.iter().for_each(|call| {
                let target = pog.get_target(call.to_proc, call.to_oper);
                diagram.push(format!(
                    "\t{}/{} -->|{}| {}",
                    self.proc, oper.oper, call.count, target
                ))
            })
        });
    }
    /// Get the label of an operation (or outbound call) of this process
    fn get_operation_label(&self, oper_idx: usize) -> String {
        format!("{}/{}", self.proc, self.operations[oper_idx].oper)
    }
}

#[derive(Debug)]
pub struct ProcOperGraph(Vec<Process>);

impl ProcOperGraph {
    pub fn new() -> Self {
        ProcOperGraph(Vec::new())
    }

    /// insert a new Process and operation pair and returns its call-descriptor
    fn push_proc_oper(&mut self, call: Call) -> Loc {
        let mut process = Process {
            proc: call.process,
            operations: Vec::new(),
        };
        let proc_idx = self.0.len();
        let oper_idx = process.push_oper(call.method, call.call_direction);
        self.0.push(process);
        Loc { proc_idx, oper_idx }
    }

    /// find the proc_oper combination, or insert it, and return the index-pair as a CallDescriptor with count = 0.0
    fn get_proc_oper_idx(&mut self, call: Call) -> Loc {
        match self.0.iter().position(|p| p.proc == call.process) {
            Some(proc_idx) => match self.0[proc_idx]
                .operations
                .iter()
                .position(|o| o.oper == call.method)
            {
                Some(oper_idx) => Loc { proc_idx, oper_idx },
                None => {
                    let oper_idx = self.0[proc_idx].push_oper(call.method, call.call_direction);
                    Loc { proc_idx, oper_idx }
                }
            },
            None => self.push_proc_oper(call),
        }
    }

    pub fn add(&mut self, from: Call, to: Call, count: f64) {
        // determine the from and to and add them if they do not exist
        let from = self.get_proc_oper_idx(from);
        let mut to = self.get_proc_oper_idx(to);
        // Add or update the link
        let to = CallDescriptor::new(to, count);
        self.0[from.proc_idx].operations[from.oper_idx].upsert_link(to)
    }

    /// get the name of a target defined by proc_idx and oper_idx within this Graph.
    fn get_target(&self, proc_idx: usize, oper_idx: usize) -> String {
        self.0[proc_idx].get_operation_label(oper_idx)
    }

    /// generate a detailled Mermaid diagram, which includes the operations and the outbound calls of each of the services.
    fn mermaid_diagram_full(&self) -> String {
        let mut diagram = Vec::new();
        diagram.push("graph LR".to_string());

        self.0.iter().for_each(|p| p.add_nodes(&mut diagram));
        self.0.iter().for_each(|p| p.add_links(&mut diagram, self));

        diagram.join("\n")
    }

    /// Get a compact Mermaid diagram only showing the services, and discarding the detail regarding the actual operation being called.
    fn mermaid_diagram_compact(&self) -> String {
        unimplemented!()
    }

    /// Extract the mermaid diagram based on these imputs
    pub fn mermaid_diagram(&self, compact: bool) -> String {
        if compact {
            self.mermaid_diagram_compact()
        } else {
            self.mermaid_diagram_full()
        }
    }
}
