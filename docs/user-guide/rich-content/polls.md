# Polls

> **Status: Complete**

Create polls to gather opinions from channel members. Polls support multiple options, multi-select voting, anonymous mode, and configurable expiry times.

## Where Polls Appear

Polls are available in **text channels** within communities. They are not available in direct messages (DMs).

There are two ways to access polls:

- **Polls panel** -- Click the bar-chart icon in the channel header (visible on desktop). This opens a panel above the message area showing all polls for the current channel.
- **Create poll button** -- Click the bar-chart icon next to the message input area, or click **New Poll** inside the polls panel.

## Creating a Poll

1. Click the **bar-chart icon** next to the message input (or the **New Poll** button in the polls panel).
2. A "Create Poll" modal appears with the following fields:

### Question

- Required
- 1 to 500 characters
- This is the main question displayed at the top of the poll

### Options

- Minimum **2** options, maximum **10** options
- Each option can be 1 to 200 characters
- Click **Add option** to add more choices (up to 10)
- Click the **X** button next to an option to remove it (minimum 2 must remain)

### Settings

| Setting | Description | Default |
|---------|-------------|---------|
| **Allow multiple votes** | When enabled, voters can select more than one option | Off |
| **Anonymous voting** | When enabled, voter identities are hidden from other members | Off |

### Expiry

Select how long the poll should remain open:

| Duration | Description |
|----------|-------------|
| Never | Poll stays open until manually closed |
| 15 minutes | |
| 1 hour | |
| 6 hours | |
| 1 day | |
| 3 days | |
| 1 week | Maximum: 7 days (10,080 minutes) |

3. Click **Create Poll** to publish it. The poll appears immediately in the channel's polls panel, and all channel members receive a real-time notification via WebSocket.

## Voting

- Click any option to cast your vote. Your selection is highlighted with an accent color and a checkmark.
- Each option shows the vote count and percentage.
- A progress bar fills behind each option to visualize the vote distribution.
- The total vote count is shown at the bottom of the poll.

### Single-select polls

In single-select mode (the default), voting for a new option automatically removes your previous vote.

### Multi-select polls

When "Allow multiple votes" is enabled, you can select as many options as you want. Clicking a selected option again removes your vote for that option.

### Anonymous polls

When "Anonymous voting" is enabled:

- Other members cannot see who voted for which option.
- Vote counts and percentages are still visible.
- The poll is tagged with an "Anonymous" badge.

## Poll Status

Polls display badges to indicate their state:

| Badge | Meaning |
|-------|---------|
| **Multi-select** | Voters can choose multiple options |
| **Anonymous** | Voter identities are hidden |
| **Expired** | The poll's expiry time has passed |
| **Closed** | The poll was manually closed |

Expired and closed polls still display results, but no new votes can be cast.

## Closing a Poll

The following users can close a poll:

- The **creator** of the poll
- Channel **admins** and **owners** (users with role management permissions)

To close a poll, click the **Close** button in the top-right corner of the poll card.

> **Tip:** Closing a poll is permanent. Once closed, it cannot be reopened.

## Real-Time Updates

Poll events are broadcast in real time via WebSocket:

- **Poll created** -- The polls panel refreshes automatically when a new poll is created in the current channel.
- **Vote cast** -- Vote counts update instantly for all channel members viewing the polls panel.
- **Poll closed** -- The poll immediately shows the "Closed" badge and disables voting.

## Who Can Create Polls

Any member of a text channel can create a poll. There are no special role or permission requirements for poll creation.
