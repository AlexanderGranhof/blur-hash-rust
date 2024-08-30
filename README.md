# blur-hash-rust

A program that implements the [blur hash algorithm](https://github.com/woltapp/blurhash/blob/master/Algorithm.md) using multithreading.

### Running the program

To run the program you must compile it for now by running `cargo build --release` then run the binary by running `./target/release/blur-hash-rust <image.png>`.

### Performance

Running the program on a 4k image

|              Type               | Time (seconds) |
| :-----------------------------: | :------------: |
|         Single Threaded         |     10.515     |
|          Multithreaded          |     2.726      |
| Multithreaded (sample rate: 10) |     0.089      |

### Sample rate

| Sample Rate | Time (seconds) | Output                                                                                                                       |
| :---------: | :------------: | ---------------------------------------------------------------------------------------------------------------------------- |
|  Original   |                | <img width="128px" src="https://raw.githubusercontent.com/AlexanderGranhof/blur-hash-rust/master/sample-data/shoes.jpg">     |
|      1      |     2.742      | <img width="128px" src="https://raw.githubusercontent.com/AlexanderGranhof/blur-hash-rust/master/sample-data/blur-1.png">    |
|      2      |     0.745      | <img width="128px" src="https://raw.githubusercontent.com/AlexanderGranhof/blur-hash-rust/master/sample-data/blur-2.png">    |
|     10      |     0.089      | <img width="128px" src="https://raw.githubusercontent.com/AlexanderGranhof/blur-hash-rust/master/sample-data/blur-10.png">   |
|     100     |     0.060      | <img width="128px" src="https://raw.githubusercontent.com/AlexanderGranhof/blur-hash-rust/master/sample-data/blur-100.png">  |
|     500     |     0.065      | <img width="128px" src="https://raw.githubusercontent.com/AlexanderGranhof/blur-hash-rust/master/sample-data/blur-500.png">  |
|    1000     |     0.061      | <img width="128px" src="https://raw.githubusercontent.com/AlexanderGranhof/blur-hash-rust/master/sample-data/blur-1000.png"> |

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
