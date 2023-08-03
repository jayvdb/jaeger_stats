use crate::{aux::read_lines, cchain_cache::EndPointCChain, cchain_stats::CChainStatsKey};
use serde::{Deserialize, Serialize};
use std::{error::Error, path::Path};

#[derive(Serialize, Deserialize, Hash, PartialEq, Eq, PartialOrd, Ord, Debug, Default, Clone)]
pub enum CallDirection {
    Inbound,
    Outbound,
    #[default]
    Unknown,
}

impl From<&str> for CallDirection {
    fn from(s: &str) -> Self {
        match s {
            "Inbound" => CallDirection::Inbound, // would be nice if "Inbound" could be taken from 'CallDirection::Inbound.as_str()'
            "Outbound" => CallDirection::Outbound,
            "Unknown" => CallDirection::Unknown,
            _ => panic!("Invalid value for CallDirection"),
        }
    }
}

impl From<Option<&String>> for CallDirection {
    fn from(s: Option<&String>) -> Self {
        match s {
            Some(s) if &s[..] == "server" => CallDirection::Inbound,
            Some(s) if &s[..] == "client" => CallDirection::Outbound,
            None => CallDirection::Unknown,
            _ => panic!("Invalid value for CallDirection: Observed '{s:?}'"),
        }
    }
}

impl CallDirection {
    fn as_str(&self) -> &'static str {
        match self {
            CallDirection::Inbound => "Inbound",
            CallDirection::Outbound => "Outbound",
            CallDirection::Unknown => "Unknown",
        }
    }
}

#[derive(Hash, Eq, PartialEq, PartialOrd, Ord, Debug, Default, Clone, Serialize, Deserialize)]
pub struct Call {
    pub process: String,
    pub method: String,
    #[serde(default)]
    pub call_direction: CallDirection,
}

impl Call {
    pub fn get_process_method(&self) -> String {
        let process = &self.process;
        let method = &self.method;
        format!("{process}/{method}")
    }
}

impl ToString for Call {
    fn to_string(&self) -> String {
        match self.call_direction {
            CallDirection::Unknown => self.process.to_owned() + "/" + &self.method,
            _ => self.process.to_owned() + "/" + &self.method + " [" + self.call_direction.as_str(),
        }
    }
}

pub type CallChain = Vec<Call>;

/// get the file-name for a specific key (excluding path)
pub fn cchain_filename(key: &str) -> String {
    format!("{key}.cchain")
}

const LEAF_LABEL_WITH_SPACE: &str = " *LEAF*";

pub const LEAF_LABEL: &str = "*LEAF*"; // LEAF_LABEL_WITH_SPACE.trim();

/// build a call-chain-key based on parameters.
/// This is a separate function as this allows us to put in another caching_process than contained in the CallChainStatsKey.
pub fn call_chain_key(call_chain: &CallChain, caching_process: &str, is_leaf: bool) -> String {
    let call_chain_str = call_chain.iter().fold(String::new(), |a, b| {
        let sep = if a.len() > 0 { " | " } else { "" };
        a + sep + &b.to_string()
    });
    let leaf_str = if is_leaf { LEAF_LABEL_WITH_SPACE } else { "" };
    call_chain_str + &"& " + &caching_process + &"& " + &leaf_str // using '&' as separator as a ';' separator would break the CSV-files
}

/// read a cchain-file and parse it
pub fn read_cchain_file(path: &Path) -> Result<EndPointCChain, Box<dyn Error>> {
    Ok(read_lines(path)?
        .filter_map(|l| {
            let l = l.unwrap();
            let l = l.trim();
            if l.len() == 0 || l.starts_with("#") {
                None
            } else {
                Some(CChainStatsKey::parse(&l).unwrap())
            }
        })
        .collect())
}

/// the label shows whether cached processes are in the call-chain and if so returns a suffix to represent it.
pub fn caching_process_label(caching_process: &Vec<String>, call_chain: &CallChain) -> String {
    if caching_process.len() == 0 {
        return "".to_owned();
    }
    let mut cached = Vec::new();

    call_chain.iter().for_each(
        |Call {
             process, method, ..
         }| {
            match &method[..] {
                "GET" | "POST" | "HEAD" | "QUERY" => (), // ignore these methods as the inbound call has been matched already. (prevent duplicates of cached names)
                _ => match caching_process.iter().find(|&s| *s == *process) {
                    Some(_) => cached.push(process.to_owned()),
                    None => (),
                },
            }
        },
    );
    if cached.len() > 0 {
        format!(" [{}]", cached.join(", "))
    } else {
        "".to_owned()
    }
}
