use super::service_oper_graph::LinkType;
use std::collections::HashMap;

#[derive(Eq, PartialEq, Hash)]
pub struct CompKey<'a> {
    //    src: &'a str,
    pub target: &'a str,
}

impl<'a> CompKey<'a> {
    pub fn new(target: &'a str) -> Self {
        Self { target }
    }
}

#[derive(Clone, Copy)]
pub struct CompValue {
    pub count: f64,
    pub link_type: LinkType,
}

impl CompValue {
    pub fn new(count: f64, link_type: LinkType) -> Self {
        Self { count, link_type }
    }

    fn merge(&mut self, other: CompValue) {
        self.count += other.count;
        self.link_type = self.link_type.merge(other.link_type);
    }
}

pub struct CompactLink<'a>(pub HashMap<CompKey<'a>, CompValue>);

impl<'a> CompactLink<'a> {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    /// add a new prefix or increment the existing with 'count'
    pub fn add(&mut self, key: CompKey<'a>, c_value: CompValue) {
        self.0
            .entry(key)
            .and_modify(|value| value.merge(c_value))
            .or_insert(c_value.clone());
    }
}
