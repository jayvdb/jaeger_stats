use std::{
    collections::HashMap,
    path::PathBuf,
};
use crate::{
    call_chain::{cchain_filename, read_cchain_file},
    cchain_stats::CChainStatsKey};


/// An end-point has a set of call-chains that originate from this endpoint (each represented by a CChainStatsKey)
pub type EndPointCChain = Vec<CChainStatsKey>;

pub struct CChainEndPointCache {
    path: PathBuf,
    cache: HashMap<String, Option<EndPointCChain>>,
}


impl CChainEndPointCache {

    pub fn new(path: PathBuf) -> Self {
        Self{path, cache: HashMap::new()}
    }

    // pub fn get_cchain_key(&mut self, key: &str) -> Option<&EndPointCChain> {
    //     // {
    //     //     if let Some(cchain_key) = self.cache.get(key) {
    //     //         return Some(cchain_key)
    //     //     };
    //     // }
    //     // self.load_cchain_key(key)
    //     let cchain_key = self.cache.get(key);
    //     match cchain_key {
    //         Some(cck) => {
    //             println!("Hello");
    //             Some(cck)   // when returning the value the code fails. if I return None everything is fine
    //         },
    //         None =>  self.load_cchain_key(key)

    //     }
        
    // }

    pub fn get_cchain_key(&mut self, key: &str) -> Option<&EndPointCChain> {
        self.cache
            .entry(key.to_string())
            .or_insert_with(|| {
//                self.load_cchain_key(key)
                let mut path = self.path.clone();
                path.push( cchain_filename(&key));
                if path.is_file() {
                    match read_cchain_file(&path) {
                        Ok(cchain_key) => {
                                Some(cchain_key)
                            },
                        Err(err) => {
                            println!("Loading of entry '{key}' failed");
                            None
                        }
                    }
                } else {
                    println!("Could not find '{}' so no call-chain available", path.display());
                    None
                }
    })
            .as_ref()
        }
        

    /// the entry should be loaded from file, added to the cache and returned
    fn load_cchain_key(&mut self, key: &str) -> Option<EndPointCChain> {
        let mut path = self.path.clone();
        path.push( cchain_filename(&key));
        if path.is_file() {
            match read_cchain_file(&path) {
                Ok(cchain_key) => {
                        Some(cchain_key)
                    },
                Err(err) => {
                    println!("Loading of entry '{key}' failed");
                    None
                }
            }
        } else {
            println!("Could not find '{}' so no call-chain available", path.display());
            None
        }
    }

}