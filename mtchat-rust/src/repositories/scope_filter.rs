//! Scope filter shared by queries over a user's own (participant) dialogs.
//!
//! The "My chats" list and the unread badge must agree on which dialogs belong
//! to the caller's current scope, so both build their condition from here.

/// Bind values for [`participant_scope_condition`]: whether the filter is on,
/// then the three scope levels. Without a scope the filter is off and the
/// arrays are empty.
pub fn scope_binds<'a>(
    scope: Option<(&'a [String], &'a [String], &'a [String])>,
) -> (bool, &'a [String], &'a [String], &'a [String]) {
    match scope {
        Some((s0, s1, s2)) => (true, s0, s1, s2),
        None => (false, &[], &[], &[]),
    }
}

/// SQL condition that keeps a participant dialog when it matches the request scope.
///
/// `dialog_id` is the SQL expression holding the dialog id (e.g. `d.id`).
/// `first_param` is the number of the first of four consecutive placeholders,
/// bound in the order returned by [`scope_binds`]:
/// `$first_param` — filter on, then `scope_level0`, `scope_level1`, `scope_level2`.
///
/// A dialog passes when the filter is off, when it has no access-scope rows
/// (no tenant attribution), or when one of its scope rows matches on every
/// level — an empty level in the row is a wildcard.
pub fn participant_scope_condition(dialog_id: &str, first_param: usize) -> String {
    let enabled = first_param;
    let level0 = first_param + 1;
    let level1 = first_param + 2;
    let level2 = first_param + 3;

    format!(
        r#"(
             NOT ${enabled}::boolean
             OR NOT EXISTS (
               SELECT 1 FROM dialog_access_scopes s WHERE s.dialog_id = {dialog_id}
             )
             OR EXISTS (
               SELECT 1 FROM dialog_access_scopes s
               WHERE s.dialog_id = {dialog_id}
                 AND (s.scope_level0 = '{{}}' OR s.scope_level0 && ${level0})
                 AND (s.scope_level1 = '{{}}' OR s.scope_level1 && ${level1})
                 AND (s.scope_level2 = '{{}}' OR s.scope_level2 && ${level2})
             )
           )"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uses_consecutive_placeholders_from_first_param() {
        let sql = participant_scope_condition("d.id", 6);

        assert!(sql.contains("NOT $6::boolean"));
        assert!(sql.contains("s.scope_level0 && $7"));
        assert!(sql.contains("s.scope_level1 && $8"));
        assert!(sql.contains("s.scope_level2 && $9"));
        assert!(!sql.contains("$10"));
    }

    #[test]
    fn references_given_dialog_expression() {
        let sql = participant_scope_condition("dp.dialog_id", 2);

        assert_eq!(sql.matches("s.dialog_id = dp.dialog_id").count(), 2);
    }

    #[test]
    fn treats_empty_scope_level_as_wildcard() {
        let sql = participant_scope_condition("d.id", 2);

        assert!(sql.contains("s.scope_level0 = '{}'"));
        assert!(sql.contains("s.scope_level1 = '{}'"));
        assert!(sql.contains("s.scope_level2 = '{}'"));
    }

    #[test]
    fn disables_filter_without_scope() {
        let (enabled, s0, s1, s2) = scope_binds(None);

        assert!(!enabled);
        assert!(s0.is_empty() && s1.is_empty() && s2.is_empty());
    }

    #[test]
    fn passes_scope_levels_through() {
        let l0 = vec!["tenant-a".to_string()];
        let l1 = vec!["dept".to_string()];
        let l2: Vec<String> = vec![];

        let (enabled, s0, s1, s2) = scope_binds(Some((&l0, &l1, &l2)));

        assert!(enabled);
        assert_eq!(s0, l0.as_slice());
        assert_eq!(s1, l1.as_slice());
        assert!(s2.is_empty());
    }
}
