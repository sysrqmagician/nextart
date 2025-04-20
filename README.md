# NextArt
<img src="https://github.com/sysrqmagician/nextart/blob/main/assets/nextart-showcase.png?raw=true" width=50% height=50%>

NextArt is a desktop application for managing the box art of [NextUI](https://github.com/loveretro/nextui) ROM collections.

[![Crates.io](https://img.shields.io/crates/v/nextart)](https://crates.io/crates/nextart)
[![License](https://img.shields.io/github/license/sysrqmagician/nextart)](https://github.com/sysrqmagician/nextart/blob/main/LICENSE)
[![Downloads](https://img.shields.io/github/downloads/sysrqmagician/nextart/total)](https://github.com/sysrqmagician/nextart/releases)


## Features
https://github.com/user-attachments/assets/51ac4183-d5a0-404a-b7c5-24a85749aad4

- **Box Art Management**: View, add, replace, copy, and delete box art for each ROM
- **Clipboard Integration**: Easily copy/paste box art images directly from the clipboard

## Installation

### Build using cargo
```bash
cargo install nextart
```


### Download pre-built binaries
Download the latest binary for your system [here](https://github.com/sysrqmagician/nextart/releases/latest).


## Directory Structure

NextArt expects your ROMs to be organized in the NextUI format:

```
Root Directory/
├── Collection1/
│   ├── game1.rom
│   ├── game2.rom
│   └── .media/
│       ├── game1.png
│       └── game2.png
├── Collection2/
│   ├── game3.rom
│   └── .media/
│       └── game3.png
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the GPL-v3 License - see the [LICENSE file](LICENSE) for details.
