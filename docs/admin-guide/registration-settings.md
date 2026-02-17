# Registration Settings

> **Status: Complete**

Chatalot supports three registration modes that control how new users can create accounts on the instance.

> **Permission Required:** Server administrator (requires access to environment variables or server configuration)

## Registration Modes

The registration mode is controlled by the `REGISTRATION_MODE` environment variable. The default is `invite_only`.

| Mode | Value | Behavior |
|------|-------|----------|
| **Open** | `open` | Anyone can register freely without an invite code |
| **Invite Only** | `invite_only` | An invite code is required to register (default) |
| **Closed** | `closed` | Registration is completely disabled |

### Open Registration

```env
REGISTRATION_MODE=open
```

Any visitor can create an account by filling in the registration form. No invite code is required. This mode is suitable for public-facing instances or during initial setup when you want to quickly onboard users.

### Invite-Only Registration

```env
REGISTRATION_MODE=invite_only
```

Users must provide a valid invite code during registration. The invite code field appears automatically on the registration form when the client detects this mode. Codes are generated from the [Invite System](./invite-system.md) in the admin panel.

This is the recommended mode for most private or organizational instances.

### Closed Registration

```env
REGISTRATION_MODE=closed
```

Registration is fully disabled. The registration endpoint returns an error: "registration is currently disabled." Existing accounts continue to function normally. This is useful when you want to freeze the user base.

## Changing the Registration Mode

The registration mode is set at the server level, not through the admin UI. To change it:

### Docker Compose

Edit your `.env` file or `docker-compose.yml`:

```yaml
services:
  chatalot:
    environment:
      REGISTRATION_MODE: invite_only
```

Then restart the container:

```bash
docker compose up -d
```

### Manual Deployment

Set the environment variable before starting the server:

```bash
export REGISTRATION_MODE=invite_only
```

The change takes effect immediately on the next server start. There is no need to run database migrations.

## Recommended Setup Flow

1. Set `REGISTRATION_MODE=open` for the first launch.
2. Register the first account, which automatically becomes the instance owner and admin.
3. Change `REGISTRATION_MODE=invite_only` and restart the server.
4. Generate invite codes from the admin panel for additional users.

## Registration Requirements

Regardless of the registration mode, all new accounts must provide:

| Field | Requirements |
|-------|-------------|
| **Username** | 3-32 characters; letters, numbers, underscores, hyphens, dots; must start with a letter or number; no consecutive dots; must not end with a dot |
| **Email** | Valid email format; must be unique on the instance |
| **Password** | 8-128 characters with at least one uppercase, one lowercase, one digit, and one special character |
| **Display Name** | Optional |
| **Invite Code** | Required in `invite_only` mode |

Additionally, the client generates E2E encryption keys (identity key, signed prekey, one-time prekeys) during registration. This is automatic and invisible to the user.

## First User Auto-Promotion

The first user to register on a new instance is automatically promoted to both **Instance Admin** and **Instance Owner**. This happens during the registration flow -- after the user record is created, the server checks if the total user count is 1, and if so, sets both the `is_admin` and `is_owner` flags.

## Admin Username Seeding

The `ADMIN_USERNAME` environment variable can be used to ensure a specific user is always an admin:

```env
ADMIN_USERNAME=alice
```

On server startup, if a user with this username exists and is not already an admin, they will be promoted to admin automatically.

## Email Verification

Chatalot does not currently implement email verification. Email addresses are collected during registration but are not verified. They serve as a contact field visible to instance admins in the [User Management](./user-management.md) panel.

## Related

- [Invite System](./invite-system.md) -- Managing invite codes for invite-only registration
- [User Management](./user-management.md) -- Viewing and managing registered users
- [Self-Hosting Configuration](../self-hosting/configuration.md) -- Full list of environment variables

## Next Step

Continue to [Role Hierarchy](./role-hierarchy.md) for a complete breakdown of roles and permissions at every level.
