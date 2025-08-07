use std::collections::{HashMap, BTreeMap, BTreeSet, HashSet};
use std::cmp::Ord;

pub fn to_btree_map<K, V>(map: HashMap<K, V>) -> BTreeMap<K, V>
where
    K: Ord,
{
    map.into_iter().collect()
}
pub fn to_btree_set<K>(map: HashSet<K>) -> BTreeSet<K>
where
    K: Ord,
{
    map.into_iter().collect()
}
