[![Rust](https://github.com/Ragnaroek/iron-wolf/actions/workflows/rust.yml/badge.svg)](https://github.com/Ragnaroek/iron-wolf/actions/workflows/rust.yml)

[![Buy Me a Coffee](https://img.shields.io/badge/Buy%20Me%20a%20Coffee-FFDD00?style=flat&logo=buy-me-a-coffee&logoColor=000)](https://www.buymeacoffee.com/ragnaroek)


# Iron Wolf
Wolfenstein 3D in Rust

The goal is to have a pixel, mod-friendly perfect recreation of Wolfenstein 3D in Rust.

E1M1 demo:
https://github.com/user-attachments/assets/54743451-ada0-4067-95cc-f9f454dc5d6a

## Playing Iron Wolf

`just run-sdl-shareware` should work out of the box on a cloned repo.
It will run the shareware version, that is also checked in along with the code
as testdata.

Alternatively you can play the web version here:
https://wolf.ironmule.dev/
If you have a copy of the full game files you can upload them there and play the full
version in your browser:

![Iron Wolf web version](https://wolf.ironmule.dev/gh/web_preview.png)

## Configuration File

A config file is optional. Copy the `default_iw_config.toml` as `iw_config.toml` and put it next to the Iron Wolf exectuable file.
The options are described as comments in the default config file.
