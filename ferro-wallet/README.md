# ferro-wallet

Digital wallet pass issuance for the Ferro framework — Apple `.pkpass` files and Google Wallet save-links.

The crate exposes a `WalletSubject` trait (the content contract any downstream model implements for its domain object), an `ApplePassBuilder` (PKCS#7-signed `.pkpass` ZIP), and a `GoogleWalletBuilder` (RS256-signed save JWT pointing at `pay.google.com/gp/v/save/{jwt}`). Image normalisation and QR generation helpers are bundled. Reads `APP_NAME` / `APP_URL` from environment, matching the project-agnostic convention shared by every `ferro-*` crate.

Status: part of the [ferro](https://github.com/albertogferrario/ferro) framework workspace.

Documentation: https://docs.rs/ferro-wallet

License: MIT
