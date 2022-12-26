# Hyperbacked

A clone of [Superbacked](https://superbacked.com/), written in Rust.

Basically, it stores secrets securely using printable PDFs that contain encrypted QR-Codes. The encrypted backup can optionally be sharded, so that
it can be distributed across many trusted people, with only a configurable subset of shards being required to decrypt the secret.

## Features

-   Written in pure, 100% safe Rust
-   Free, open-source, and auditable
-   Supports _Plausible Deniability_, _Secret Sharing_, and _256-bit AES-GCM encryption_.
-   Runs on Windows, macOS, and Linux with a native GUI using [iced](https://iced.rs)

## Encryption

An example of encrypting a secret using 2/3 secret sharing:

![Encryption](./media/encrypt.mp4)

## Decryption

An example of decryption the secret created in the previous video, with only 2 of the 3 generated shards:

![Encryption](./media/decrypt.mp4)
