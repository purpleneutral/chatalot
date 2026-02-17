# Reports and Moderation

> **Status: Complete**

The Reports tab lets you view and act on content reports submitted by users across the instance.

> **Permission Required:** Instance Admin or Instance Owner

## Overview

Any user can report content (messages, users, or other items) by providing a reason. Reports are submitted to the admin panel for review. Each report has a lifecycle status that tracks its progress from submission to resolution.

## Report List

Reports are displayed as cards, each showing:

| Field | Description |
|-------|-------------|
| **Status badge** | Color-coded status: Pending (yellow), Reviewed (blue), Resolved (green), Dismissed (gray) |
| **Report type** | The category of the report (e.g., message, user) |
| **Date** | When the report was submitted |
| **Reason** | The reporter's description of the issue |
| **Target ID** | UUID of the reported content (click to copy) |
| **Reporter ID** | UUID of the user who submitted the report (click to copy) |
| **Admin Notes** | Notes added during review (if any) |

### Filtering by Status

Use the **Filter** dropdown in the top-right corner to filter reports by status:

- **All** -- Show all reports regardless of status
- **Pending** -- Reports awaiting review
- **Reviewed** -- Reports that have been seen but not yet resolved
- **Resolved** -- Reports where action has been taken
- **Dismissed** -- Reports that were determined to not require action

### Pagination

Reports are displayed 25 per page, with pagination controls at the bottom when there are more pages available.

## Reviewing a Report

1. Find a report with `pending` status.
2. Click the **Review** button at the bottom of the report card.
3. A review form appears inline with two fields:

| Field | Description |
|-------|-------------|
| **Status** | Select the new status: `Reviewed`, `Resolved`, or `Dismissed` |
| **Notes** | Optional admin notes (up to 5,000 characters) explaining the decision or action taken |

4. Click **Submit** to save the review, or **Cancel** to discard.

## Report Status Workflow

```
Pending --> Reviewed --> Resolved
   |                       ^
   +-----> Dismissed       |
   +-----> Resolved -------+
```

| Status | Meaning |
|--------|---------|
| **Pending** | Newly submitted, awaiting admin review |
| **Reviewed** | An admin has looked at it but has not taken final action yet |
| **Resolved** | The issue has been addressed (e.g., content removed, user warned/suspended) |
| **Dismissed** | The report was determined to be invalid, a duplicate, or not actionable |

## Taking Action on Reported Content

The report system is informational -- it presents the report data and lets you update its status. To take action on the reported content itself, use the appropriate admin tool:

| Action | Where |
|--------|-------|
| Suspend the reported user | [User Management](./user-management.md) |
| Delete the reported user | [User Management](./user-management.md) |
| Quarantine a reported message | [Security Settings](./security-settings.md) (Quick Quarantine) |
| Purge reported content | [Security Settings](./security-settings.md) (Purge Tools) |
| Quarantine or delete a reported file | [File Management](./file-management.md) |

After taking action, return to the report and mark it as **Resolved** with notes describing what was done.

## Audit Trail

Review actions are recorded in the [Audit Log](./audit-log.md) with the action `report_reviewed`, including the report ID and new status in the metadata.

## API Reference

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/admin/reports` | GET | List reports with optional `status`, `page`, `per_page` query params |
| `/admin/reports/{id}/review` | POST | Review a report (body: `{ status, admin_notes }`) |

## Related

- [Moderation Guide](../moderation-guide/README.md) -- Community-level moderation tools for warnings, timeouts, kicks, and bans

## Next Step

Continue to [Audit Log](./audit-log.md) to learn about the admin audit trail.
