# Reports

> **Status: Complete**

Reports allow any community member to flag content for review. The report system bridges the gap between regular users (who can see a problem) and moderators/admins (who can take action).

## How Users File Reports

Any authenticated user can report a message, user, or file. The most common flow is reporting a message directly from the chat interface.

### Reporting a Message

1. Right-click (or long-press on mobile) the message you want to report.
2. Select **Report** from the context menu. This option appears for all messages that were not sent by you.
3. A **Report Message** modal opens, asking "Why are you reporting this message?"
4. Describe the issue in the text area (required, up to 1,000 characters in the UI; up to 2,000 via the API).
5. Click **Submit Report**.
6. You will see a confirmation: "Report submitted -- admins will review it."

### Report Types

Reports can target three types of content:

| Report Type | Description | When to Use |
|-------------|-------------|-------------|
| `message` | A specific chat message | Harassment, spam, rule violations, illegal content |
| `user` | A user account | Impersonation, repeated harassment, bot/spam accounts |
| `file` | An uploaded file | Malicious files, inappropriate media, copyright violations |

The message report flow uses `report_type: "message"` automatically. User and file reports can be submitted via the API.

### API Details

```
POST /api/reports
```

Request body:
```json
{
  "report_type": "message",
  "target_id": "<message-uuid>",
  "reason": "This message contains harassment directed at another member."
}
```

- `report_type` must be `message`, `user`, or `file`.
- `target_id` is the UUID of the message, user, or file being reported.
- `reason` must be 1--2,000 characters.

The report is automatically associated with the reporting user and timestamped. A `report_created` entry is added to the audit log.

## Report Workflow for Admins

> **Permission Required:** Instance Admin or Instance Owner

Reports are reviewed in the **admin panel**, not at the community level. This ensures a consistent review process across the entire instance.

### Viewing Reports

Navigate to the **Reports** tab in the admin panel. Reports are listed with:

- **Status badge** -- color-coded: pending (yellow), reviewed (blue), resolved (green), dismissed (gray)
- **Report type** -- `message`, `user`, or `file`
- **Reason** -- the reporter's description of the issue
- **Reporter ID** -- who submitted the report
- **Target ID** -- what was reported
- **Created at** -- when the report was filed

Reports can be filtered by status and are paginated for large volumes.

### Report Statuses

| Status | Meaning |
|--------|---------|
| **pending** | New report, not yet reviewed by an admin |
| **reviewed** | An admin has seen the report and is investigating |
| **resolved** | Action was taken -- the issue has been addressed |
| **dismissed** | The report was reviewed but no action was warranted |

### Reviewing a Report

1. Open the admin panel and navigate to the **Reports** tab.
2. Click on a pending report to expand its details.
3. Investigate the reported content:
   - For message reports, find the message and check its context and edit history.
   - For user reports, review the user's profile and recent activity.
   - For file reports, inspect the file in the admin file browser.
4. Take appropriate action if needed (warn, timeout, kick, ban, delete, quarantine).
5. Update the report status and add **admin notes** (optional, up to 5,000 characters).

**From the API:**

```
POST /api/admin/reports/{reportId}/review
```

Request body:
```json
{
  "status": "resolved",
  "admin_notes": "User was warned and the message was deleted. Harassment policy violation."
}
```

The review records the reviewing admin's ID and a `reviewed_at` timestamp. The action is also logged in the audit log.

## Dismissing vs. Acting on Reports

Not every report warrants action. Here is a framework for deciding:

### When to Resolve (Take Action)

- The reported content clearly violates community rules or instance policies.
- The reported user has a history of similar behavior.
- The content is illegal, threatening, or constitutes harassment.

After taking moderation action (warning, timeout, kick, ban, delete, quarantine), set the report status to **resolved** and document what was done in the admin notes.

### When to Dismiss

- The report is a misunderstanding or disagreement, not a rule violation.
- The reported content does not violate any rules upon review.
- The report appears to be retaliatory or abusive.

Set the report status to **dismissed**. Adding a brief note explaining the reasoning is good practice in case the same content is reported again.

### When to Mark as Reviewed

- You have seen the report and want to investigate further before making a decision.
- You are gathering more context or consulting with other admins.
- The situation is borderline and needs monitoring.

## Report Tips for Users

- **Be specific.** Describe exactly what the problem is and why it violates the rules.
- **Report once.** Submitting the same report multiple times does not speed up review.
- **Do not abuse the system.** Frivolous or retaliatory reports waste moderator time and may result in action against the reporter.
- **Block if needed.** If you feel unsafe, you can also block the user immediately. Blocking is separate from reporting and takes effect instantly.

## Next Steps

- [Content Moderation](./content-moderation.md) -- Tools admins use to act on reported content
- [Permissions Reference](./permissions-reference.md) -- Who can file and review reports
- [Admin Guide -- Reports and Moderation](../admin-guide/reports-and-moderation.md) -- Full admin-side report management
