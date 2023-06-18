
![](images/logo.png)

**copycolors** is command-Line Interface to  extract  the dominant colors from local or remote images faster.
It is built with Rust and is essentially based on the implementation the *Colors Thief* algorithm written by  
[Yevhenii Reizner](https://github.com/RazrFalcon/color-thief-rs).

## Installation

 copycolors CLI can be installed from different sources: 

 - You can get sources and pre-built binaries for the latest release of copycolors from the [releases page](https://www.github.com/abdoulma/copycolors/releases).

- If you have a Rust toolchain installed, you can also install the latest development version with `cargo install --git https://github.com/abdoulma/copycolors` or more simply you can install the stable version with `cargo install copycolors`

## Usage
 
copycolors comes with a set of features that you can use through arguments.

### Basic example

```bash
# For a local image file example.png 
copycolors example.png 
# or 
copycolors path/to/example.png
 
# For remote image 
copycolors "https://www.example.com/png"
```

![](images/ex1.gif)

You can choose the numbers of colors you want to extract with the `--nb-colors` option (or with its shorter version `-n`). The default  is `5`.

```
copycolors example.png -n 8 
```

If you want to display the `rgb` value of the colors instead of hexadecimal, you can use the `--rgb` (or `-r`) flag. 

```
copycolors example.png -r 
```

If you want a better and broader perception of the extracted colors, you can use the `--canvas`  (or `-c`)  flag to display the colors as rectangle. 

As, I know that it can be frustating to have some colors we don't want to extract, there is an  `exc-colors` (or `-e`) option, which you can use to specify to hexadecimal value of colors you want to exclude of extraction. Under the hood, there is a method that filters  out colors that are less than `5 %` distant of the colors you want to exclude.

```
# Put the colors hexadecimal values between quotations marks
copycolors example.png -c -e "#A4E14b" "#84C512" 
```

The last useful options is sorting the extracted colors  by their contrast with white color with the flag `--bcw` or the opposite with the flag  `--bcb`. 

```
# Sort by colors best contrasting with white
copycolors example.png --bcw

# Sort by colors best contrasting with black
copycolors example.png --bcb
```

The supported images formats are: 
 
|Format| Extension|
|:-------|:------|
|PNG ||
|JPEG||
|GIF||
|BMP ||
|ICO ||
|TIFF ||
|WEBP ||
|AVIF ||
|PNM ||
|DDS ||
|TGA ||

The supported colors formats are: `RGB` and `RGBA`.

You can access the CLI help with `--help` (`-h`) flag. 

```
copycolors -h
```

## Contributing

If you experience bugs, feel free to open an issue or send a `Pull Request` with a fix. For new features, I would invite you to open an `issue` first so we can explore the design paradigm.