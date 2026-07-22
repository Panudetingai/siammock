# SiamMock Editor Extension

Realtime validation, autocomplete, and file icons for SiamMock config files.

## Install (one time)

```bash
cargo build
cd editor/siammock
npm install
npm run compile
```

Then in Cursor / VS Code:

1. **Developer: Install Extension from Location...** → select `editor/siammock`
2. **Reload Window**

## File types

| Extension | Purpose |
|---|---|
| `.jsonsi` | SiamMock config (custom icon in Explorer) |
| `mock/*.json` | Legacy mock configs |

## Autocomplete

| Trigger | Suggestions |
|---|---|
| `"method": "` | GET, POST, PUT, PATCH, DELETE |
| `{{` | uuid, timestamp, param:id, body:field |
| `{` or `,` in route | path, method, response, request |
| `route` + Tab | Full route snippet |

Enable icons in suggest widget: `"editor.suggest.showIcons": true` (already in workspace settings).

## Manual validate

Command Palette → **SiamMock: Validate Active File**

## Settings

| Setting | Default |
|---|---|
| `siammock.binaryPath` | auto-detect |
| `siammock.validateOnChange` | `true` |
| `siammock.validateDebounceMs` | `400` |
| `siammock.mockJsonGlob` | `mock/*` |
