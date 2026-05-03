# Nook

Nook is a tool for managing Envoy bookings and check-ins.

## Setup

### 1. Create a profile

```sh
$ nook --profile "my-profile" profile create
```

Prompts for:
- **Location ID** — the Envoy location ID (e.g. `<your-location-id>`)
- **Timezone** — IANA timezone for the office (e.g. `Australia/Sydney`)
- **Access token** — your Envoy access token (input hidden)
- **Refresh token** — your Envoy refresh token (input hidden)

If `NOOK_AUTH_KEY` is not set, you will be prompted to generate a new
encryption key or paste an existing one. **Store this key safely** — it is
required to decrypt your tokens and is never stored on disk.

### 2. Show profile info

```sh
$ NOOK_AUTH_KEY=<key> nook --profile "my-profile" profile info
```

Shows user info (name, email), token expiry times, and which profiles file is being used.

### 3. Refresh tokens

```sh
$ NOOK_AUTH_KEY=<key> nook --profile "my-profile" profile refresh
```

Explicitly refreshes the access and refresh tokens. Tokens are also refreshed
automatically when expired or on a 401/403 response.

## Bookings

### Show bookings

```sh
$ NOOK_AUTH_KEY=<key> nook --profile "my-profile" booking show
```

Shows bookings for the next 14 days. Optionally specify a date range:

```sh
$ NOOK_AUTH_KEY=<key> nook --profile "my-profile" booking show \
    --start-date 2026-05-01 \
    --end-date 2026-05-14
```

### Create a booking

```sh
$ NOOK_AUTH_KEY=<key> nook --profile "my-profile" booking create --date "2026-05-05"
```

The location is read from the profile. Use `--date latest` to book for today.
Use `--backfill` to book all available dates in the next 14 days:

```sh
$ NOOK_AUTH_KEY=<key> nook --profile "my-profile" booking create \
    --backfill \
    --date latest
```

Optionally specify a preferred desk by name or raw ID:

```sh
# By desk name (recommended) — looks up the ID automatically
$ NOOK_AUTH_KEY=<key> nook --profile "my-profile" booking create \
    --date latest \
    --desk "26.036"

# By raw ID — bypasses lookup entirely
$ NOOK_AUTH_KEY=<key> nook --profile "my-profile" booking create \
    --date latest \
    --desk raw:12345
```

Desk names are resolved via a local cache (`~/.cache/nook/cache/desks-<location-id>.json`).
The cache is populated on first use and automatically refreshed if a desk name is not found.
To force a refresh manually, delete the cache file.

If the booking lands on a different desk than requested, it is shown in yellow
in the output table with the requested desk ID noted.

### Check-in to a booking

```sh
$ NOOK_AUTH_KEY=<key> nook --profile "my-profile" booking check-in --date latest
```

Use `--date latest` to check in to all past bookings in temporal order, stopping
at the first future booking. Use an explicit date to check in to a specific day.

## Configuration

All configuration is via environment variables:

| Variable             | Description                                                        | Default                                |
|----------------------|--------------------------------------------------------------------|----------------------------------------|
| `NOOK_AUTH_KEY`      | Base64-encoded 32-byte AES-256 encryption key (required)           | —                                      |
| `NOOK_PROFILES_FILE` | Path to the profiles YAML file (overrides path resolution)         | See below                              |
| `NOOK_LOG_FILE`      | Path to the log file                                               | `$XDG_CACHE_HOME/nook/<timestamp>.log` |
| `NOOK_LOG_LEVEL`     | Log file level (`error`, `warn`, `info`, `debug`, `trace`)        | `debug`                                |
| `RUST_LOG`           | Stderr log level (`error`, `warn`, `info`, `debug`, `trace`)      | `warn`                                 |
| `XDG_CONFIG_HOME`    | XDG config directory (for profiles file)                           | `~/.config`                            |
| `XDG_CACHE_HOME`     | XDG cache directory (for log files)                                | `~/.cache`                             |

## File locations

### Profiles file

Nook looks for the profiles file in this order:

1. `NOOK_PROFILES_FILE` env var (if set)
2. `./nook.yml` in the current working directory (if it exists)
3. `$XDG_CONFIG_HOME/nook/nook.yml` (i.e. `~/.config/nook/nook.yml`)

New profiles are always created at the XDG path unless a local `./nook.yml` already exists.

| File       | Default path                        | Description                           |
|------------|-------------------------------------|---------------------------------------|
| Profiles   | `~/.config/nook/nook.yml`           | Encrypted profile config              |
| Log file   | `~/.cache/nook/logs/<YYYYMMDDTHHMMSS>.log` | Structured JSON log, one file per run |
| Latest log | `~/.cache/nook/logs/latest.log`            | Symlink to the most recent log file   |
| Desk cache | `~/.cache/nook/cache/desks-<location-id>.json` | Cached desk name→id map, delete to refresh |

### Profiles file format

```yaml
profiles:
  - name: my-profile
    location_id: "<your-location-id>"
    timezone: "Australia/Sydney"
    auth:
      last_refreshed_at: "2026-05-03T12:00:00Z"
      token:
        token_type: Bearer
        access_token:
          aes256: "<encrypted>"
          expiry: "2026-05-04T12:00:00Z"
        refresh_token:
          aes256: "<encrypted>"
          expiry: "2026-06-03T12:00:00Z"
```

## Logging

Nook uses two log outputs:

- **Stderr** — human-readable, controlled by `RUST_LOG` (default: `warn`)
- **Log file** — structured JSON at `DEBUG` level, always written

```sh
# See retry/refresh events on stderr
RUST_LOG=info nook --profile "my-profile" booking show

# See all debug events on stderr
RUST_LOG=debug nook --profile "my-profile" booking show

# Include raw API response bodies in the log file (contains sensitive data)
NOOK_LOG_LEVEL=trace nook --profile "my-profile" booking show

# Tail the latest log file
tail -f ~/.cache/nook/logs/latest.log | jq .
```

On any fatal error, the path to the log file is printed to stderr:

```
error: token refresh failed: ...
(log file: /Users/you/.cache/nook/logs/20260503T120400.log)
```

Log entries are structured JSON for easy parsing:

```json
{"timestamp":"2026-05-03T12:04:01","level":"WARN","fields":{"message":"Server error, retrying after backoff...","status":"502","attempt":1,"delay_ms":634},"target":"nook::envoy::client"}
```
