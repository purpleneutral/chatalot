/// Role hierarchy: owner(2) > admin(1) > member(0)
fn role_level(role: &str) -> u8 {
    match role {
        "owner" => 2,
        "admin" => 1,
        _ => 0,
    }
}

/// Check if actor_role can perform a moderation action on target_role.
/// An actor can only moderate users with strictly lower role level.
pub fn can_moderate(actor_role: &str, target_role: &str) -> bool {
    role_level(actor_role) > role_level(target_role)
}

/// Check if a role can delete messages from other users.
pub fn can_delete_others_messages(role: &str) -> bool {
    role_level(role) >= 1 // admin and owner
}

/// Check if a role can change other users' roles.
pub fn can_manage_roles(role: &str) -> bool {
    role_level(role) >= 2 // owner only
}
