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

---

## Configuration Overview

All options are read from `[preprocessor.qr]` and its sub-tables.

| Key | Type | Description | Default |
|-----|------|--------------|----------|
| `enable` | bool | Enable or disable the preprocessor | `true` |
| `marker` | string | the marker where `<img>` is injectd| `{{QR_CODE}}`|
| `url` | string | The URL or text to encode | *(required)* |
| `qr-path` | string | Relative or absolute path to the output PNG | `"qr/mdbook-qr-code.png"` |
| `margin` | integer | Quiet zone around the QR code (in modules) | `2` |
| `background` | string | Hex color (`#RRGGBBAA`,`#RRGGBB`,`[RRR,GGG,BBB,AAA]`,`[RRR,GGG,BBB]` supported) | `"#FFFFFFFF"` |
| `module` | string | Hex color (`#RRGGBBAA`,`#RRGGBB`,`[RRR,GGG,BBB,AAA]`,`[RRR,GGG,BBB]` supported) | `"#000000FF"`  |
| `shape` | table | Boolean flags defining the QR module shape | `{ square = true }` |

>[!IMPORTANT]
> - `marker` is defaulted to `{{QR_CODE}}` and cannot explicitly be set to anything else. If you want to use your marker then create a `custom.*` sub-table, see [Custom Configurations](#configuration-overview) 

### Fit (Image Size)

```toml
[preprocessor.qr.fit]
width = 200
height = 200
```
If only one dimension is provided, the same value is used for the other.

---

## Shape

```toml
[preprocessor.qr.shape]
square = true
circle = true
rounded_square = true
vertical = true
horizontal = true
diamond = true
```

**Shape Precedence (first `true` wins):**

> `circle → rounded_square → vertical → horizontal → diamond → square`

If none are supplied, **square** is used.

> [!NOTE]  
> `fast_qr::convert::Shape::Command` (for custom procedural shapes) is not yet implemented.

---

## URL Resolution

If `url` is omitted, `mdbook-qr` resolves it automatically from:

- GitHub Actions environment variable `GITHUB_REPOSITORY`, producing:  
  
  - `https://{owner}.github.io/{repo}`

  >[!NOTE]
  > You can always set an env variable locally to test CI
  > - `export GIT_REPOSITORY="owner/repo"`
  >    
  > To `unset`
  > - `unset GIT_REPOSITORY`

---

## Custom Configurations

Custom QR definitions allow you to create **named styles** that inherit values from the main `[preprocessor.qr]` table.  

These are declared under `[preprocessor.qr.custom.*]` as a sub-table.

Each named sub-table inherits **all** parent values unless explicitly overridden, accept `marker`.

```toml
[preprocessor.qr]
url = "https://default.example.com"
margin = 2
background = "#FFFFFFFF"
module = "#000000FF"

[preprocessor.qr.custom.footer]
marker = "{{QR_FOOTER}}"
url = "https://github.com/CompEng0001"
qr-path = "src/footer-qr.png"
fit.width = 128
fit.height = 128
shape.diamond = true

[preprocessor.qr.custom.slide]
marker = "{{QR_SLIDE}}"
url = "https://slides.example.com"
module = "#22AAFFFF"
background = "#00000000"
shape.circle = true
```

>[!IMPORTANT]
> The `custom.*` sub-table only generates a QR code when:
> - When the `marker` is defined and placed in a document
> - If URL not defined then the sub-table will inherit from default
> - If `qr-path` is not defined then default will be `qr_codes/qr-code.png` 

### Custom Configuration Overview

| Key | Type | Description | Example |
|-----|------|--------------|----------|
| `preprocessor.qr.custom.marker` | string | Placeholder text used in Markdown | `"{{QR_CUSTOM}}"` |

Based on the configuration in [Custom Configurations](#custom-configurations) you would use these markers in your Markdown files:

```md
{{QR_FOOTER}}
{{QR_SLIDE}}
```

Each marker corresponds to its respective `[preprocessor.qr.custom.*]` block.  

---

## Example Outputs

![square QR](docs/src/figures/qr-square.png) ![diamond QR](docs/src/figures/qr-diamond.png) ![rounded circle transparent blueish QR](docs/src/figures/qr-rounded-circle-transparent-blueish.png)

```html
<img src="./qr.png" alt="QR code" style="width:200px;height:200px;" loading="eager">
```

---

## License

Licensed under the [MIT License](LICENSE.md)

---

## Author

[CompEng0001](https://github.com/CompEng0001)
