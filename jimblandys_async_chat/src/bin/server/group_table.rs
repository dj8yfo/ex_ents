use crate::group::Group;
use crate::participants::GroupMembers;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct GroupTable(Mutex<HashMap<Arc<String>, Arc<Group>>>);

impl GroupTable {
    pub fn new() -> GroupTable {
        GroupTable(Mutex::new(HashMap::new()))
    }

    pub fn get(&self, name: &String) -> Option<Arc<Group>> {
        self.0.lock()
            .unwrap()
            .get(name)
            .cloned()
    }

    pub fn remove(&self, name: &String) -> Option<Arc<Group>> {
        self.0.lock()
            .unwrap()
            .remove(name)
    }

    pub fn get_or_create(self: &Arc<Self>, name: Arc<String>) -> Arc<Group> {
        let participants = Arc::new(GroupMembers::new(self.clone()));
        self.0.lock()
            .unwrap()
            .entry(name.clone())
            .or_insert_with(|| Arc::new(Group::new(name, participants)))
            .clone()
    }
}

