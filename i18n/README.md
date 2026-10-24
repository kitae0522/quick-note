# English and Korean messages

Edit `en.json` or `ko.json` as UTF-8 JSON. These files contain application labels,
instructions, accessibility names, shortcut help, dialogs, errors and generated
Markdown labels. English is the default. **Options → Language** switches immediately
and saves the choice for the next launch.

The JSON object uses stable field names. For example, these entries in `ko.json`
control the New note button, save status and deletion explanation:

```json
{
  "new_note": "새 노트",
  "saved": "저장됨",
  "delete_body": "“{title}” 노트가 완전히 삭제돼요."
}
```

This is an excerpt of the bundled catalog. Required fields,
nonempty text, unknown keys and named placeholders are validated before applying
a file. Preserve placeholders such as `{title}`, `{error}`, `{line}` and `{column}`.
Use `{{` and `}}` for literal braces. Values inserted into placeholders are never
parsed again, so a note titled `{error}` stays literal.

## Edit installed messages without rebuilding

Copy the supplied JSON files into an `i18n` directory inside the note data folder:

- macOS: `~/Library/Application Support/Quick Note/i18n/`
- Windows: `%LOCALAPPDATA%\Quick Note\i18n\`

Edit the copied file, then select that language again in Options or restart the
app. Only files you create there override the defaults; the app never overwrites
your edits. Installed overrides may contain only the fields you want to change;
missing fields inherit that language's bundled text, including labels added by updates.
Unknown fields, invalid values and changed placeholders are still rejected.

The app checks data-folder overrides first, then packaged catalogs in macOS
`Contents/Resources/i18n/` or the Windows executable's adjacent `i18n/` folder,
then built-in copies. Use the data folder for macOS customization so signed app
resources remain intact. Source edits take effect in the next build.

Invalid changes keep the current language when switching. An invalid catalog for
the saved language prevents startup and identifies the file to fix; it never
resets the note database. Remove the override to return to the packaged catalog.

User-written note titles and contents are not translated or rewritten. Only
generated blank-note names change language. Theme names, format identifiers,
keyboard key names, and technical error details from the OS or SQLite are retained.
Native file-picker system buttons follow the platform's language settings.

Run `./scripts/check.sh` for catalog and behavior tests, and
`python3 scripts/check-localization.py` for isolated native-window verification.
