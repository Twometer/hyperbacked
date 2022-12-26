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

https://user-images.githubusercontent.com/26793103/209542520-5d198371-0cc4-49ba-b98b-73fd10b25628.mp4

## Decryption

An example of decryption the secret created in the previous video, with only 2 of the 3 generated shards:

https://user-images.githubusercontent.com/26793103/209542512-00f8c6d2-5981-4ca5-9afd-e7135a3e1395.mp4

