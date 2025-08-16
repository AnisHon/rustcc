use std::collections::BTreeSet;
pub fn extend<'a, T, S>(set: &'a mut BTreeSet<T>, extends: S) -> bool
where
    T: Ord + PartialOrd + Clone,
    S: Iterator<Item = &'a T>
{
    let mut changed = false;
    for item in extends {
        if set.insert(item.clone()) {
            changed = true;
        }
    }
    changed
}