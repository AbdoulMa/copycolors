![](images/logo.png)

**copycolors** is command-Line Interface to extract the dominant colors from local or remote images faster.

It is built with Rust and is essentially based on the implementation the _Colors Thief_ algorithm written by  
[Yevhenii Reizner](https://github.com/RazrFalcon/color-thief-rs).

## Installation

copycolors CLI can be installed from different sources:

- You can get sources and pre-built binaries (for Linux and Windows) for the latest release of copycolors from the [releases page](https://www.github.com/abdoulma/copycolors/releases).

- If you have a Rust toolchain installed, you can also install the latest development version with `cargo install --git https://github.com/abdoulma/copycolors`.

## Usage

copycolors comes with a set of features that you can use through arguments.

### Basic example

```bash
# For a local image file example.png
copycolors example.png
# or
copycolors path/to/example.png

# For remote image
copycolors "https://www.example.com/example.png"
```

For example,  let's take a local image `example.png`.

![](images/example.png)

Now, let's try to extract the 5 dominant colors of this  image: 

![](images/cast/ex1.gif)

Instead of the path to your local file, you can provide the link to the remote image file. 

For example, let's extract the dominant colors  from an illustration from the NY Times, hosted at : <https://static01.nyt.com/images/2023/06/02/business/02adviser/02adviser-threeByTwoSmallAt2X.jpg?format=pjpg&quality=75&auto=webp&disable=upscale>

![](images/cast/ex2.gif)

You can choose the numbers of colors you want to extract with the `--nb-colors` option (or with its shorter version `-n`). The default is `5`.

```
copycolors example.png -n 8
```

![](images/cast/ex3.gif)

If you want to display the `rgb` value of the colors instead of hexadecimal, you can use the `--rgb` (or `-r`) flag.

```
copycolors example.png -r
```
![](images/cast/ex4.gif)

If you want a better and broader perception of the extracted colors, you can use the `--canvas` (or `-c`) flag to display the colors as rectangle.

![](images/cast/ex5.gif)

As, I know that it can be frustating to have some colors we don't want to extract, there is an `exc-colors` (or `-e`) option, which you can use to specify to hexadecimal value of colors you want to exclude of extraction. Under the hood, there is a method that filters out colors that are less than `5 %` distant of the colors you want to exclude.

```
# Put the colors hexadecimal values between quotations marks
copycolors example.png -c -e "#CCC8B2" "#E0796C"
```

![](images/cast/ex6.gif)
 

The last useful options is sorting the extracted colors by their contrast with white color with the flag `--bcw` or the opposite with the flag `--bcb`.

```
# Sort by colors best contrasting with white
copycolors example.png --bcw

# Sort by colors best contrasting with black
copycolors example.png --bcb
```

![](images/cast/ex7.gif)


The supported images formats are:

| Format |
| :----- |
| PNG    |
| JPEG   |
| GIF    |
| BMP    |
| ICO    |
| TIFF   |
| WEBP   |
| AVIF   |
| PNM    |
| DDS    |
| TGA    |

The supported colors formats are: `RGB` and `RGBA`.

You can access the CLI help with `--help` (`-h`) flag.

```
copycolors -h
```

## Contributing

If you experience bugs, feel free to open an issue or send a `Pull Request` with a fix. For new features, I would invite you to open an `issue` first so we can explore the design paradigm.
