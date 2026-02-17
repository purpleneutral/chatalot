# Creating an Account

> **Status: Complete**

This page walks you through the Chatalot registration process.

![Chatalot registration page](../../screenshots/09-register-page.png)

## Registration Modes

Chatalot instances can be configured in one of three registration modes:

| Mode | Behavior |
|------|----------|
| **Open** | Anyone can register freely |
| **Invite only** | An invite code is required to register |
| **Closed** | Registration is disabled; contact the server administrator for access |

When you visit the registration page, the client automatically detects which mode the server is using and adjusts the form accordingly. If the server requires an invite code, an **Invite Code** field will appear at the top of the form.

## Step-by-Step Registration

1. Navigate to the Chatalot instance URL in your browser.
2. On the login page, click **Create account** in the bottom-right corner.
3. Fill in the registration form:

### Invite Code (if required)

If the server is configured as invite-only, you will see an **Invite Code** field at the top of the form. Enter the code provided to you by the server administrator or another member.

### Username

- **Required**
- 3 to 32 characters
- Allowed characters: letters, numbers, underscores, hyphens, and dots
- Must be unique on the server

### Display Name (optional)

- Up to 64 characters
- This is the name shown to other users in chat
- If left blank, your username is used as your display name

### Email

- **Required**
- Must be a valid email address

### Password

- **Required**
- Minimum 8 characters
- Must contain **all** of the following:
  - At least one uppercase letter (A-Z)
  - At least one lowercase letter (a-z)
  - At least one digit (0-9)
  - At least one special character (anything that is not a letter or digit)

As you type your password, a live checklist shows which requirements have been met.

### Confirm Password

- **Required**
- Must match the password entered above

4. Click **Create Account**.

## Recovery Code

After successful registration, a **recovery code** is displayed in a modal dialog. This is the only way to recover your account if you forget your password.

**Save this code immediately.** Copy it to a secure location (password manager, printed note, etc.). It will not be shown again.

Once you have saved the code, click **I've Saved My Code** to continue to the main chat interface.

## E2E Encryption Keys

During registration, Chatalot automatically generates your end-to-end encryption keys in the background. This includes:

- An **identity key** (permanent key that identifies you)
- A **signed prekey** (rotated periodically)
- A set of **one-time prekeys** (used to establish new sessions)

This process is invisible to you and requires no manual steps.

> **Tip:** If you already have an account, click **Sign in** at the bottom of the registration page to return to the login screen.

## Next Step

Now that your account is created, continue to [Setting Up Your Profile](./setting-up-your-profile.md).
