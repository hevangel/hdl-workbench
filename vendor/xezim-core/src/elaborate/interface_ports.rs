//! Hierarchical interface-port names refer to the connected instance (§25.3).

use super::*;
use std::borrow::Cow;

/// Replace the longest interface-port prefix, retaining selects and members.
/// The top's name is optional in flat keys. Bound the walk so malformed alias
/// cycles cannot hang a diagnostic or a runtime lookup.
pub fn resolve<'a>(name: &'a str, aliases: &HashMap<String, String>, top: &str) -> Cow<'a, str> {
    if aliases.is_empty() {
        return Cow::Borrowed(name);
    }
    let mut current = Cow::Borrowed(name);
    for _ in 0..aliases.len() {
        let flat = current
            .strip_prefix(top)
            .and_then(|s| s.strip_prefix('.'))
            .unwrap_or(&current);
        let found = std::iter::once(flat.len())
            .chain(
                flat.char_indices()
                    .rev()
                    .filter_map(|(i, c)| matches!(c, '.' | '[').then_some(i)),
            )
            .find_map(|end| aliases.get(&flat[..end]).map(|actual| (end, actual)));
        let Some((end, actual)) = found else { break };
        let next = format!("{actual}{}", &flat[end..]);
        if next == current {
            break;
        }
        current = Cow::Owned(next);
    }
    current
}
