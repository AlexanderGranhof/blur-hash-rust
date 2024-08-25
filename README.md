# blur-hash-rust

A program that implements the [blur hash algorithm](https://github.com/woltapp/blurhash/blob/master/Algorithm.md) using multithreading.

### Running the program

To run the program you must compile it for now by running `cargo build --release` then run the binary by running `./target/release/blur-hash-rust <image.png>`.

### Performance

|      Type       | Time (seconds) |
| :-------------: | :------------: |
| Single Threaded |                |
|  Multithreaded  |     2.726s     |

### Help

```
Usage: blur-hash-rust [OPTIONS] <FILEPATH>

Arguments:
  <FILEPATH>  Path to the image file

Options:
  -x <X>         x component of the hash (1-9) [default: 4]
  -y <Y>         y component of the hash (1-9) [default: 4]
  -h, --help     Print help
  -V, --version  Print version
```
