# Releasing

Installers are built by `.github/workflows/release.yml` (macOS universal
`.dmg`/`.app`, Windows NSIS and MSI) when a version tag is pushed. The
workflow creates a draft pre-release, uploads both platforms' installers
and publishes it once both have succeeded. The release text comes from
`.github/release-notes.md` (`<version>` is replaced with the version).

Without any secrets the installers are **unsigned**: the macOS app is only
ad-hoc signed (`signingIdentity: "-"`), so Gatekeeper asks the user to
allow it under Privacy & Security; Windows SmartScreen warns once; and the
program cannot update itself. Signing needs accounts that only the
project's maintainers can hold, so it is set up separately.

## 1. Version and tag

Bump the version in `Cargo.toml` (`[workspace.package] version`),
`app/src-tauri/tauri.conf.json` and `app/package.json`, commit, then tag
and push:

```bash
git tag v0.2.0 && git push origin main v0.2.0
```

The tag must be `v` followed by the version in `tauri.conf.json`, or the
workflow stops. A run that failed for a passing reason (a network error)
can be re-run from the Actions page. To ship a fix, move the tag to the
fixed commit (`git tag -f v0.2.0 && git push -f origin v0.2.0`); either
way the run reuses the existing draft release.

## 2. macOS signing and notarisation

Needs an Apple Developer Program membership.

1. Create a *Developer ID Application* certificate and export it as `.p12`.
2. Repository secrets:
   - `APPLE_CERTIFICATE`: the `.p12`, base64-encoded
   - `APPLE_CERTIFICATE_PASSWORD`
   - `APPLE_SIGNING_IDENTITY`, e.g. `Developer ID Application: Name (TEAMID)`
   - `APPLE_ID`, `APPLE_PASSWORD` (an app-specific password), `APPLE_TEAM_ID`

`tauri-action` signs and notarises when these are present;
`APPLE_SIGNING_IDENTITY` replaces the ad-hoc identity from
`tauri.conf.json`.

## 3. Windows code signing

Open-source projects can sign for free through
[SignPath Foundation](https://signpath.org). Alternatively use an OV/EV
certificate and set `bundle.windows.certificateThumbprint` (or a
`signCommand`) in `tauri.conf.json`.

## 4. In-app updates

The updater checks
`https://github.com/chess-organiser/chess-organiser/releases/latest/download/latest.json`
and only installs updates signed with the project's update key.

1. `cargo tauri signer generate -w ~/.tauri/chess-organiser.key`
2. Put the **public** key in `app/src-tauri/tauri.updater.conf.json`
   (`plugins.updater.pubkey`). Keep the private key out of the repository.
3. Repository secrets `TAURI_SIGNING_PRIVATE_KEY` (the key file's content)
   and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`.
4. Repository variable `UPDATER_ENABLED` = `true`. The workflow then builds
   with `--features updater --config src-tauri/tauri.updater.conf.json` and
   uploads `latest.json`.

Builds without the `updater` feature show "This build does not update
itself" under Settings › Updates.

## 5. Publishing server

`co-publish` is a single binary:

```bash
cargo build --release -p co-publish
CO_PUBLISH_TOKEN=… CO_PUBLISH_DATA=/var/lib/co-publish CO_PUBLISH_ADDR=127.0.0.1:8787 ./target/release/co-publish
```

Put it behind a reverse proxy that terminates HTTPS (Caddy, nginx). The
app refuses to upload over plain HTTP except to `localhost`.

## Local builds

```bash
cd app && pnpm tauri build            # .app and .dmg in target/release/bundle/
```
