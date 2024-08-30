# blur-hash-rust

A program that implements the [blur hash algorithm](https://github.com/woltapp/blurhash/blob/master/Algorithm.md) using multithreading.

### Running the program

To run the program you must compile it for now by running `cargo build --release` then run the binary by running `./target/release/blur-hash-rust <image.png>`.

### Performance

Running the program on a 4k image

|      Type       | Time (seconds) |
| :-------------: | :------------: |
| Single Threaded |     10.515     |
|  Multithreaded  |     2.726      |

### Help

```
Usage: blur-hash-rust [OPTIONS] <FILEPATH>

Arguments:
  <FILEPATH>  Path to the image file

Options:
  -x <X>         x component of the hash (1-9) [default: 4]
  -y <Y>         y component of the hash (1-9) [default: 4]
  -s <STEP>      step size for the sampling [default: 1]
  -h, --help     Print help
  -V, --version  Print version
```
