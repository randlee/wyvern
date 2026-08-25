# Path picker (wizard native file/folder)

Two-page vanilla-chrome wizard that calls `WyvernApi.postPickerFile` and
`WyvernApi.postPickerFolder` on the sources page. Finish JSON carries path
strings only — page JS never reads or writes the filesystem.

`config.seed_paths` pre-fills `file_paths` / `folder_paths` so an agent can
demo a provisioned list before the user browses additional paths.

## GUI

```bash
wyvern {wyvern_share}/examples/path-picker/wizard.json \
  --ui-root {wyvern_share}/examples/path-picker
```

Repo checkout:

```bash
wyvern share/wyvern/examples/path-picker/wizard.json \
  --ui-root share/wyvern/examples/path-picker
```

## Headless + mock picker

```bash
WYVERN_MOCK_PICKER_PATH=/tmp/wyvern-path-picker-fixture.txt \
  WYVERN_VIEWER=none \
  wyvern {wyvern_share}/examples/path-picker/wizard.json \
  --ui-root {wyvern_share}/examples/path-picker
```

Repo checkout:

```bash
WYVERN_MOCK_PICKER_PATH=/tmp/wyvern-path-picker-fixture.txt \
  WYVERN_VIEWER=none \
  wyvern share/wyvern/examples/path-picker/wizard.json \
  --ui-root share/wyvern/examples/path-picker
```

`--viewer none` starts the host without a window. Drive
`POST /api/picker/file`, `POST /api/picker/folder`, navigate, then
`POST /api/wizard/finish`. Stdout finish shape:

```json
{
  "button": "finish",
  "data": {
    "file_paths": ["/abs/path/to/file.csproj"],
    "folder_paths": ["/abs/path/to/root"]
  },
  "stack": []
}
```

Lint the package:

```bash
wyvern wizard lint {wyvern_share}/examples/path-picker
```
