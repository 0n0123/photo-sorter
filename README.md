# Photo sorter

## What's this?

This tool sorts photos by DateTimeOriginal from Exif metadata.

## How to use

```
$photo-sorter path/to/directory
```

The files includes `path/to/directory` will be renamed to `###__` prefixed.
`###` is a number of the sorted order.

By default, the files are sorted oldest to latest order.
Specify `--desc` option to reverse.

### Test mode

If `-t` or `--test` option is specified, the files will not be renamed.
The tool shows order in stdout.
