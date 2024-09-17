

## VSCode Tips

Due to the size of some of the XML files. This can cause VS Code to crash or freeze.

To prevent this from happening. Add the following to your project's `settings.json` file:

In `.vscode/settings.json`
```json
  "files.exclude": {
      "**/*.hidden.xml": true
  }
```