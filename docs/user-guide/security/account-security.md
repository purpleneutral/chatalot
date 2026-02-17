# Account Security

> **Status: Complete**

This page covers all the tools available to secure your Chatalot account: password management, two-factor authentication, session management, recovery codes, and what to do if your account is compromised.

## Changing Your Password

You can change your password from **Settings > Account > Change Password**.

### Steps

1. Open **Settings** and go to the **Account** tab.
2. In the **Change Password** section, enter your **Current Password**.
3. Enter your **New Password**. A live checklist shows which requirements are met as you type.
4. Enter the new password again in **Confirm New Password**.
5. Click **Change Password**.

### Password Requirements

Chatalot enforces the following complexity rules on all passwords (8-128 characters):

| Requirement | Rule |
|-------------|------|
| Minimum length | At least 8 characters |
| Uppercase letter | At least one A-Z |
| Lowercase letter | At least one a-z |
| Digit | At least one 0-9 |
| Special character | At least one character that is not a letter or digit |

The **Change Password** button remains disabled until all five requirements are met and both password fields match.

> **Important:** Changing your password immediately signs you out of all devices. You will be redirected to the login page and must sign in again with your new password.

## Two-Factor Authentication (2FA)

Chatalot supports TOTP-based two-factor authentication, compatible with apps like Google Authenticator, Authy, 1Password, and Bitwarden.

### Enabling 2FA

1. Open **Settings > Security**.
2. In the **Two-Factor Authentication** section, click **Enable 2FA**.
3. A setup panel appears with:
   - An `otpauth://` URL (for QR code scanning)
   - A **manual entry secret** you can copy into your authenticator app
4. Add the account to your authenticator app.
5. Enter the 6-digit code from your app and click **Verify & Enable**.
6. If successful, a set of **backup codes** is displayed. Save these immediately.

### Disabling 2FA

1. In the **Two-Factor Authentication** section, click **Disable 2FA**.
2. Enter a valid 6-digit code from your authenticator app.
3. Click **Disable 2FA** to confirm.

### Backup Codes

When you enable 2FA, you receive a set of one-time backup codes. Each code can be used exactly once in place of a TOTP code if you lose access to your authenticator app.

**Save your backup codes in a secure location** (password manager, printed copy, etc.). They are only displayed once at setup time.

To regenerate backup codes (if you have used some or lost them):

1. In the **2FA Backup Codes** section, enter a valid 6-digit TOTP code.
2. Click **Regenerate**.
3. New codes are displayed. The old codes are immediately invalidated.

## Recovery Code

Your recovery code is a separate credential that lets you reset your password without needing an administrator. It is generated during registration and can be regenerated at any time.

### Regenerating Your Recovery Code

1. Open **Settings > Security**.
2. In the **Recovery Code** section, click **Generate New Recovery Code**.
3. A new code is displayed. **Copy and save it immediately** -- it replaces your previous code and will not be shown again.

> **Important:** Each time you generate a new recovery code, the previous one is permanently invalidated. Only the most recent code works.

## Active Sessions

Chatalot lets you see all devices and browsers where your account is currently signed in.

### Viewing Sessions

1. Open **Settings > Security**.
2. The **Active Sessions** section lists all sessions with:
   - **Device name** (e.g., browser user agent or desktop client identifier)
   - **IP address**
   - **Created date**
   - **Expiration** (days remaining, or "Expired")

Sessions are shown with device-appropriate icons (desktop monitor, mobile phone, or globe for web browsers).

### Revoking a Single Session

Click **Revoke** next to any session to immediately invalidate it. That device will be signed out and must log in again.

### Revoking All Sessions

Click **Revoke All** at the top of the sessions list to sign out of every device, including your current one. A confirmation dialog appears before proceeding. After confirmation, you are redirected to the login page.

## Account Deletion

If you need to permanently delete your account:

1. Open **Settings > Account**.
2. In the **Danger Zone** section, click **Delete Account**.
3. A confirmation panel appears warning that the action is permanent.
4. Enter your password and click **Delete Forever**.

All your data will be permanently deleted.

## What to Do If Your Account Is Compromised

If you suspect unauthorized access to your account:

1. **Change your password immediately** (Settings > Account > Change Password).
2. **Revoke all sessions** (Settings > Security > Active Sessions > Revoke All). This signs out every device.
3. **Enable 2FA** if you have not already (Settings > Security > Two-Factor Authentication).
4. **Regenerate your recovery code** (Settings > Security > Recovery Code) in case the attacker obtained it.
5. **Regenerate 2FA backup codes** if 2FA was already enabled, since the attacker may have recorded them.
6. **Check your profile** (Settings > Profile) for unauthorized changes to your display name, avatar, or bio.
7. **Notify your server administrator** if you believe the compromise involved server-level access.

## Related

- [Encryption Status](./encryption-status.md) -- what is and is not encrypted
- [Creating an Account](../../getting-started/creating-an-account.md) -- initial password and recovery code setup
