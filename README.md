ripdu
--

ripdu is a command line disk usage utility. it's build on rust and uses multi-thread
directory traversal same as used in ripgrep and fd

### Usage

```
ripdu 0.1.3
Nikolajus Krauklis <nikolajus@gmail.com>
ripdu is a command line disk usage utility - get back your space

USAGE:
    ripdu [OPTIONS] [FOLDER]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -n, --number <NUMBER>    Number of top results to return. Default: 10

ARGS:
    <FOLDER>    Folder where to scan
```

### Building

```
$ git clone https://github.com/dzhibas/ripdu
$ cd ripdu
$ cargo build --release
$ ./target/release/ripdu --version
$ cp ./target/release/ripdu /usr/local/bin
```