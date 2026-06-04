# GV Pixara — metadata & privacy

## Policy

Every **converted** output file:

- **Removes** GPS, location, camera make/model, dates, comments, copyright, XMP, IPTC, and other EXIF identity tags
- **Bakes** EXIF orientation into pixels (outputs do not rely on an Orientation tag)
- **Optionally keeps** ICC color profile on **PNG** when **Keep color profile (ICC)** is enabled in the app

The app does **not** add fake camera metadata or disclosure templates.

## Skip same format (important)

When **Skip same format** is on and source format equals target format, the file is **not** re-encoded. The **original file on disk is unchanged**, including GPS and all EXIF.

To strip metadata from those files, turn **Skip same format** off so they are re-encoded.

## What we do not strip

- **Source files** you dropped (only new outputs are processed)
- **File system timestamps** (Created / Modified on Windows)
- **Invisible pixel watermarks** or model fingerprints (not metadata)
- **Skipped** or **overwrite: skip** outputs that were never written

## Verifying GPS removal

On a converted file, use any of:

- Windows **Properties → Details** (no GPS latitude/longitude)
- [ExifTool](https://exiftool.org/): `exiftool -gps:all image.jpg` should show nothing
- The in-app privacy pass runs after every encode (JPEG APP segments, PNG text/metadata chunks, WebP `EXIF` / `XMP`, GIF comment blocks, TIFF re-encode from pixels, HEIC/AVIF ISO-BMFF `meta` removal)

## Color (ICC)

ICC is separate from GPS/EXIF. **Keep color profile** controls whether PNG outputs embed `iCCP`. JPEG/WebP/HEIC targets may not carry ICC even when the option is on.
