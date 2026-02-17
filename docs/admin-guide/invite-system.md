# Invite System

> **Status: Complete**

The Invites tab lets you generate, view, and manage registration invite codes. Invite codes are used when the instance is configured in **invite-only** mode.

> **Permission Required:** Instance Admin or Instance Owner

## Overview

Registration invite codes are alphanumeric codes that prospective users enter during account registration. Each code is a randomly generated 12-character string (e.g., `aBcDeFgHiJkL`). Codes can be configured with optional usage limits and expiration times.

> **Tip:** Set `REGISTRATION_MODE=invite_only` in your server environment to require invite codes for new registrations. See [Registration Settings](./registration-settings.md) for details.

## Viewing Invites

The invites table displays all invite codes ordered by creation date (newest first), with the following columns:

| Column | Description |
|--------|-------------|
| **Code** | The invite code string (monospace font, click to copy) |
| **Uses** | Current usage count, with max uses if set (e.g., `3/10` or just `3`) |
| **Expires** | Expiration date and time, or "Never" if no expiry is set |
| **Created** | Date the code was created |
| **Actions** | Copy and Delete buttons |

## Generating an Invite Code

1. Click the **Generate Invite** button in the top-right corner of the Invites section.
2. A creation form appears with two optional fields:

| Field | Description | Default |
|-------|-------------|---------|
| **Max Uses** | Maximum number of times the code can be used | Unlimited |
| **Expires In (hours)** | Number of hours until the code expires (1 to 8,760, i.e. up to 1 year) | Never |

3. Click **Create** to generate the code.
4. The new code will appear in the table. A success toast will display the code, and you can copy it using the **Copy** button.

## Sharing Invite Codes

After generating a code, click **Copy** next to it to copy it to your clipboard. Share the code with the intended user through a secure channel (direct message, email, etc.).

The user enters this code in the **Invite Code** field on the registration page.

## Deleting an Invite Code

Click **Delete** next to any invite code to permanently remove it. A confirmation dialog is shown before deletion. Once deleted, the code cannot be used for new registrations, even if it had remaining uses or had not yet expired.

> **Note:** Deleting an invite code does not affect accounts that were already created using that code.

## How Invite Validation Works

When a user registers with an invite code, the server atomically validates and consumes the code:

1. The code must exist in the database.
2. If a `max_uses` limit is set, the `used_count` must be less than `max_uses`.
3. If an `expires_at` timestamp is set, the current time must be before the expiration.
4. If all checks pass, the `used_count` is incremented by 1 in the same database transaction, preventing race conditions.

If any check fails, the registration is rejected with the message: "invalid, expired, or fully used invite code."

## API Reference

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/admin/invites` | GET | List all registration invites |
| `/admin/invites` | POST | Create a new invite (body: `{ max_uses, expires_in_hours }`) |
| `/admin/invites/{id}` | DELETE | Delete an invite code |

## Next Step

Continue to [File Management](./file-management.md) to learn about managing uploaded files.
