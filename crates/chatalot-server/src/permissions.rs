/// Unified role hierarchy:
/// instance_owner(5) > instance_admin(4) > owner(3) > admin(2) > moderator(1) > member(0)
fn role_level(role: &str) -> u8 {
    match role {
        "instance_owner" => 5,
        "instance_admin" => 4,
        "owner" => 3,
        "admin" => 2,
        "moderator" => 1,
        _ => 0,
    }
}

/// Compute the effective role for a user, considering instance-level privileges.
/// Instance owner/admin roles supersede any local (channel/community) role.
pub fn effective_role(local_role: Option<&str>, is_owner: bool, is_admin: bool) -> String {
    if is_owner {
        return "instance_owner".to_string();
    }
    if is_admin {
        return "instance_admin".to_string();
    }
    local_role.unwrap_or("member").to_string()
}

/// Check if actor_role can perform a moderation action on target_role.
/// An actor can only moderate users with strictly lower role level.
pub fn can_moderate(actor_role: &str, target_role: &str) -> bool {
    role_level(actor_role) > role_level(target_role)
}

/// Check if a role can delete messages from other users.
pub fn can_delete_others_messages(role: &str) -> bool {
    role_level(role) >= 1 // moderator and above
}

/// Check if a role can change other users' roles.
pub fn can_manage_roles(role: &str) -> bool {
    role_level(role) >= 3 // owner and above
}

/// Check if a community role meets a policy threshold.
/// Policies: "everyone" (any member), "moderator" (mod+), "admin" (admin/owner only).
/// Instance admins/owners always pass.
pub fn meets_policy(role: &str, policy: &str) -> bool {
    let level = match role {
        "instance_owner" | "instance_admin" | "owner" => 3,
        "admin" => 2,
        "moderator" => 1,
        _ => 0,
    };
    let required = match policy {
        "everyone" => 0,
        "moderator" => 1,
        _ => 2, // "admin" or any unknown value defaults to admin-level
    };
    level >= required
}
