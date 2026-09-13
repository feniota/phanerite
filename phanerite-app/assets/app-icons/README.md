These icons are raster exports of `../phanerite-logo.svg`, created by Enita
Nureya, copyright (c) 2026 The Feniota Team. They retain the logo's
[CC BY-SA 4.0 license](../phanerite-logo.LICENSE).

Regenerate all resolutions and the Windows/macOS containers from the SVG:

```sh
deno run --allow-read --allow-write --allow-env --allow-ffi phanerite-app/scripts/generate-app-icons.js
```

The script uses Deno and Sharp. Each size is rendered directly from the SVG;
the ICO and ICNS containers retain those rendered frames. Generated files are
committed so normal builds do not require Deno or an image toolchain.
