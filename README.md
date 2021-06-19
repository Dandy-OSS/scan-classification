# Scan classification

Binary for quickly classifying STL files


### Installation

This program requires only a recent version of Rust.

If you do not have Rust installed, you can find it [here](https://www.rust-lang.org/tools/install)

If you have an old version of Rust installed, you can update to the most recent release by running `rustup update stable && rustup default stable`

Having Rust installed, you can then download and run this binary,

```bash
git clone https://github.com/Dandy-OSS/scan-classification
cd scan-classification
cargo r --release
```

### Usage
Provided are sample STL files to test with. It's possible to add more STLs by modifying `path_queue` in `src/main.rs`.

Using the keys `W`, `A`, `S`, and `D`, you can write the path of the STL to files of the same name. 

Using the keys `P` and `Q` you can pause the rotation of the model and quit the program respectively.

You can zoom in using the mouse wheel (or scrolling equivalent), and you can get more precise rotation using the arrow keys.