//! A test env: a fixed map behind the same lookup shape the crate reads.

use std::collections::HashMap;

/// A lookup that answers from `pairs` and nothing else.
pub(crate) fn env(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> + use<> {
    let map: HashMap<String, String> = pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
    move |name| map.get(name).cloned()
}
