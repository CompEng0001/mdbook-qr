<div align="center">
  <h1 align="center"><b>mdbook-qr</b></h1>
</div>

<p align="center">
  <a href="https://crates.io/crates/mdbook-qr">
    <img src="https://img.shields.io/crates/v/mdbook-qr?style=for-the-badge" alt="Crates.io version" />
  </a>
  <a href="https://crates.io/crates/mdbook-qr">
    <img src="https://img.shields.io/crates/d/mdbook-qr?style=for-the-badge" alt="Downloads" />
  </a>
  <a href="https://docs.rs/mdbook-qr">
    <img src="https://img.shields.io/docsrs/mdbook-qr?style=for-the-badge" alt="Docs.rs" />
  </a>
  <a href="https://github.com/CompEng0001/mdbook-qr/actions">
    <img src="https://img.shields.io/github/actions/workflow/status/CompEng0001/mdbook-qr/release.yml?&style=for-the-badge&label=CI" alt="CI status" />
  </a>
  <img src="https://img.shields.io/badge/Built%20with-Rust-orange?logo=rust&style=for-the-badge" alt="Built with Rust" />
</p>

An [mdBook](https://github.com/rust-lang/mdBook) preprocessor that generates and embeds a QR code for your book, powered by [fast-qr](https://docs.rs/fast-qr).  

It produces a PNG image during the build and replaces `{{QR_CODE}}` markers in chapters with an `<img>` tag pointing to the generated QR code.

Read the documentation [here](https://compeng0001.github.io/mdbook-qr), to see the actual examples in action.

---

## Features

- Generates **PNG QR codes** using [`fast-qr`](https://docs.rs/fast-qr)
- Structured configuration under `[preprocessor.qr]` with sub-tables:
  - Configurable **RGB/A** or **hex** color options
  - Optional **fit width/height** for the `<img>` tag
  - Support for multiple **module shapes** (see [Shapes](#shape))
  - Adjustable **quiet zone margin**
- Supports **custom named QR configurations** under `[preprocessor.qr.custom.*]`  (see [Custom Configuration](#custom-configuration-overview))

---

## Installation

From crates.io:

```sh
cargo install mdbook-qr
```

From source (in this repository):

```sh
cargo install --path .
```

Ensure the `mdbook-qr` binary is available on your `PATH`.

---

## Quick Start

Add to your `book.toml`:

```toml
[preprocessor.qr]
enable = true
url = "https://example.com"
qr-path = "src/qr.png"
margin = 2
background = "#FFFFFFFF"
module = "#000000FF"

[preprocessor.qr.fit]
width = 256
height = 256

[preprocessor.qr.shape]
circle = true
```

Then, in any Markdown file:

```md
{{QR_CODE}}
```

During the build, this is replaced with:

```html
<img src="./qr.png" alt="QR code" style="width:256px;height:256px;" loading="eager">
```

...and rendered as:

![](docs/src/figures/qr-square.png)

---

## License

Licensed under the [MIT License](LICENSE.md)

---

## Author

[CompEng0001](https://github.com/CompEng0001)
