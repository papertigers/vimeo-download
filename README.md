# vimeo-download

This tool allows you to download a Vimeo video locally by combining the
audio/video streams from a provided master.json URL.

This was inspired by a [python](https://github.com/eMBee/vimeo-download) version
of vimeo-download which appeared to no longer be maintained.

### Usage

```
vimeo-download 0.1.0

USAGE:
    vimeo-download [FLAGS] [OPTIONS] <url>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information
    -v, --verbose

OPTIONS:
    -d, --directory <directory>
    -f, --filename <filename>

ARGS:
    <url>
```

### Example

```
$ ./vimeo-download https://183vod-adaptive.akamaized.net/path/segments/master.json?base64_init=1 -d /tmp -f foo.mp4
<.......snip.......>
$ file /tmp/foo.mp4
/tmp/foo.mp4: ISO Media, MP4 Base Media v1 [IS0 14496-12:2003]
```
