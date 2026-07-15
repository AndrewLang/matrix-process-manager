# SSH Keys

The SSH Keys screen lists the SSH keys found for your user and lets you generate
new key pairs. Use it to review your existing keys and to create a key for a new
server or service.

## What You Can Do

- List existing SSH keys with their type and details.
- View a key's fingerprint, comment, and public-key path.
- Copy a public key.
- Generate a new SSH key pair.

## Open the SSH Keys Screen

1. In the sidebar, select **SSH Keys**.
2. Your existing keys load.

## Reading the List

Each key can show:

- **Name** and **Key type**.
- **Public key path** and, where present, **Private key path**.
- **Fingerprint** and **Comment**.
- **Modified** date and whether a **private key** is present.

## Copy a Public Key

1. Select the key you want to share.
2. Copy its public key.
3. Paste it where it is needed (for example, into a server's authorized keys or a
   service's SSH settings).

Copying the public key is safe to share.

## Generate a New Key

1. Choose to generate a new key.
2. Provide a file name, key type, and comment.
3. Confirm to create the key pair.
4. The new key appears in the list.

## Safety Notes

> Keep your **private key** secret. Never share it, paste it into websites, or
> post it in logs or chats. Only the **public key** is meant to be shared. Prism
> surfaces public-key material and metadata; treat everything associated with the
> private key as sensitive.

## Recommended Workflow

1. Review existing keys before creating a new one to avoid duplicates.
2. Generate a dedicated key per service where practical.
3. Add a clear comment so you can identify the key later.
4. Copy the public key to the target service.

## Limitations

- Listing and generation depend on your system's SSH tooling and your account's
  access to the SSH key directory.

## Next Steps

- [Command Center](/prism/console/command-center)
- [Settings](/prism/settings/general)
