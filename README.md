# OLED ssd1351 Raspberry PIvideo player

- enable larger spidev buffers by appending `spidev.bufsiz=32768` to `/boot/cmdline.txt`
- videos in BGR565 Big Endian format are mapped from the disk directly to the memory, so preprocessing step is needed:
```
$ ffmpeg -i input.webm -vf scale=128x128 -f rawvideo -pix_fmt bgr565be output.raw
```

# Production build & run
```sh
$ find path/to/playlists
path/to/playlists/1
path/to/playlists/1/first.bgr565be
path/to/playlists/1/second.bgr565be
path/to/playlists/2
path/to/playlists/2/1hour.bgr565be
$ cargo build --release
$ ./target/release/ssd1351_pideoplayer path/to/playlists
```
