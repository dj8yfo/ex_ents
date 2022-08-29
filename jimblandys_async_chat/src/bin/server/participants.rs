use std::{sync::{Arc, Mutex}, collections::HashSet, net::SocketAddr};

use crate::group_table::GroupTable;

pub struct GroupMembers {
    members: Mutex<HashSet<MemberId>>,
    groups: Arc<GroupTable>,
}

type MemberId = SocketAddr;


impl GroupMembers {
    pub fn new(group_table: Arc<GroupTable>) -> GroupMembers {
        GroupMembers {
            members: Mutex::new(HashSet::new()),
            groups: group_table,
        }
    }

    pub fn join(
        &self,
        group_name: &str,
        member: MemberId,
    ) -> Result<usize, String> {
        let mut guard = self.members.lock().unwrap();
        if !guard.insert(member) {
            return Err(format!(
                "double join attempt {} <- {}",
                group_name, member
            ));
        } else {
            println!("joined {} <- {}, ({})", group_name, member, guard.len());
        }
        Ok(guard.len())
    }

    pub fn leave(&self, group_name: &String, member: MemberId) -> usize {
        let mut guard = self.members.lock().unwrap();
        let removed = guard.remove(&member);
        println!(
            "removed from {} ->out  \"{}\", if removed {}, ({})",
            group_name, member, removed, guard.len(),
        );
        if guard.len() == 0 {
            self.groups.remove(group_name);
        }
        guard.len()
    }
}
