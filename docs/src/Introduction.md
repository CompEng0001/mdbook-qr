<div align="center">
  <h1 align="center"><b>Welcome to mdbook-qr</b></h1>
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
</p>

An [mdBook](https://github.com/rust-lang/mdBook) preprocessor that generates and embeds a QR code for your book, powered by [fast-qr](https://docs.rs/fast-qr).  

It produces a PNG image during the build and replaces `{{QR_CODE}}` markers in chapters with an `<img>` tag pointing to the generated QR code.

<br>

## Compatibility 

| mdbook-qr version|mdbook versions|Notes|
|:---------------:|:------------------:|----------------|
| v1.\*.*|v0.5.* | Requires API changes introduced in mdbook 0.5.* |  
| v0.\*.*|v0.4.* | Not compatible with 0.5.*+ |


<br>

For all options see [Documentation](./Documentation.md) chapter.

## Live Configuration Example 

As seen from this page the current preprocessor configuration is: 

```toml
[preprocessor.qr]
enable = true
qr-path = "figures/mdbook-qr-code.png"
margin = 1
shape.rectangle = true
background = "#FFFFFFFF"
module =  "#000000FF"
```


Then, in any Markdown file:

```md
{{QR_CODE}}
```

During the build, this is replaced with:

```html
 <img src="./figures/mdbook-qr-code.png?" alt="QR code" style="height:200px;width:200px" loading="eager">
```

...and rendered as: 

{{QR_CODE}}