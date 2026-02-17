# Announcements

> **Status: Complete**

The Announcements tab lets you publish server-wide announcements that are displayed as banners to all users in the chat interface.

> **Permission Required:** Instance Admin or Instance Owner

## Overview

Announcements are a way to communicate important information to every user on the instance -- maintenance windows, policy changes, new features, or any other server-wide notices. Announcements appear as dismissible banners in the chat view. Each user can dismiss announcements individually, but the announcement remains visible to users who have not yet dismissed it.

## Creating an Announcement

1. Navigate to the **Announcements** tab in the admin panel.
2. Fill in the creation form at the top of the page:

| Field | Requirements |
|-------|-------------|
| **Title** | Required. 1 to 200 characters. |
| **Body** | Required. 1 to 5,000 characters. |

3. Click **Publish Announcement**.

The announcement is immediately:

- Saved to the database.
- Broadcast via WebSocket to all currently connected users, appearing as a real-time banner.
- Added to the top of the announcements list in the admin panel.

## Viewing Announcements

The Announcements tab displays all announcements in reverse chronological order (newest first), limited to the most recent 100. Each announcement card shows:

- **Title** -- The announcement heading
- **Body** -- The full announcement text
- **Date** -- When the announcement was published

## How Users See Announcements

When a user loads the chat interface, the client fetches any announcements they have not yet dismissed. These appear as banners at the top of the chat area. Users can dismiss individual announcements, which records a dismissal in the database so the announcement does not reappear for that user.

New announcements published while users are connected are delivered instantly via WebSocket.

## Announcement Lifecycle

```
Admin publishes --> Banner shown to all users
                --> User dismisses --> Banner hidden for that user
                --> New user logs in --> Banner shown (unless previously dismissed)
```

There is currently no built-in mechanism to edit or delete announcements after publication. Announcements persist in the database indefinitely. If you need to retract an announcement, you can do so directly in the database:

```sql
-- Delete an announcement by ID
DELETE FROM announcements WHERE id = 'announcement-uuid-here';

-- Delete associated dismissals
DELETE FROM announcement_dismissals WHERE announcement_id = 'announcement-uuid-here';
```

## Best Practices

- **Keep titles concise.** The title is the most prominent element in the banner.
- **Use the body for details.** Include links, timelines, or instructions in the body.
- **Avoid frequent announcements.** Users may begin dismissing banners without reading them if announcements are too frequent.
- **Coordinate with scheduled maintenance.** Publish announcements in advance of planned downtime.

## API Reference

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/admin/announcements` | POST | Create an announcement (body: `{ title, body }`) |
| `/admin/announcements` | GET | List all announcements (admin view, up to 100) |

The following user-facing endpoints handle announcement display and dismissal:

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/announcements` | GET | List undismissed announcements for the authenticated user |
| `/announcements/{id}/dismiss` | POST | Dismiss an announcement for the authenticated user |

## Next Step

Continue to [Community Oversight](./community-oversight.md) to learn about admin visibility into communities.
