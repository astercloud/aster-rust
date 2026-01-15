# astery

Put `astery` in your $PATH if you want to launch via:

```
astery .
```

This will open aster GUI from any path you specify

# Unregister Deeplink Protocols (macos only)

`unregister-deeplink-protocols.js` is a script to unregister the deeplink protocol used by aster like `aster://`.
This is handy when you want to test deeplinks with the development version of aster.

# Usage

To unregister the deeplink protocols, run the following command in your terminal:
Then launch aster again and your deeplinks should work from the latest launched aster application as it is registered on startup.

```bash
node scripts/unregister-deeplink-protocols.js
```

