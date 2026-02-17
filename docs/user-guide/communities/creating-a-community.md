# Creating a Community

> **Status: Complete**

Any user can create a community, unless the instance administrator has restricted community creation to admins only.

## How to Create a Community

1. Open the **community picker** on the left side of the screen.
2. Click the **+** (plus) button at the bottom of the community list.
3. Fill in the community details:
   - **Name** (required) -- 1 to 64 characters
   - **Description** (optional) -- up to 2,048 characters
4. Click **Create**.

Your new community is created immediately, and you are set as its **Owner**.

## What Gets Created by Default

When you create a community, Chatalot automatically sets up:

- A **"General" group** -- the first organizational folder in your community
- A **#general text channel** inside that group -- ready for conversation

You can rename, reconfigure, or delete these defaults at any time.

## Who Can Create Communities

Community creation is controlled by the instance administrator through the `COMMUNITY_CREATION_MODE` environment variable:

| Mode | Who can create |
|------|---------------|
| `everyone` (default) | Any registered user |
| `admin_only` | Only instance administrators |

If you cannot see the create button, your instance may have restricted community creation. Contact your instance administrator for access.

## After Creating

Once your community exists, you will want to:

1. **Invite members** -- generate an invite link so others can join (see [Managing Members](./managing-members.md))
2. **Create groups** -- organize your community with additional groups (see [Creating Groups](../groups/creating-groups.md))
3. **Configure settings** -- set up the community icon, banner, and policies (see [Community Settings](./community-settings.md))
4. **Upload custom emoji** -- add fun emoji for your community (see [Custom Emoji](./custom-emoji.md))

## Limits

- Community name: 1--64 characters
- Community description: up to 2,048 characters
- Groups per community: up to 200
- There is no hard limit on the number of communities a user can create or join

> **Tip:** Choose a descriptive community name and add a description. These are shown in invite previews when the community is set to discoverable.
