<p align="center">
  <br>
  <br>
  <code>&nbsp;forge&nbsp;</code>
  <br>
  <br>
  <strong>URL in. Clean markdown out.</strong>
  <br>
  <em>One binary. One job. No config.</em>
  <br>
  <br>
  <a href="#install">Install</a> &nbsp;&middot;&nbsp;
  <a href="#usage">Usage</a> &nbsp;&middot;&nbsp;
  <a href="#what-it-strips">What it strips</a> &nbsp;&middot;&nbsp;
  <a href="#what-it-keeps">What it keeps</a>
  <br>
  <br>
</p>

---

```
$ forge https://en.wikipedia.org/wiki/Stigmergy

# Stigmergy

**Stigmergy** is a mechanism of indirect coordination, through
the environment, between agents or actions. The principle is
that the trace left in the environment by an individual action
stimulates the performance of a succeeding action by the same
or different agent.

## History

The term "stigmergy" was introduced by the French biologist
Pierre-Paul Grassé in 1959 to refer to termite behavior...
```

That's it. That's the whole interface.

---

## Why

Every LLM pipeline, every RAG system, every research script has the same first step: get a webpage, throw away the crap, keep the words. Forge does that one thing and does it well.

No browser. No JavaScript engine. No headless Chrome. No Python. No config files. No flags. No API keys. Just a 2.3 MB static binary that reads HTML and writes markdown.

## Install

```sh
# From source (requires Rust)
git clone https://github.com/denster32/forge.git
cd forge
cargo build --release
cp target/release/forge /usr/local/bin/
```

```sh
# Or just build and run directly
cargo install --path .
```

## Usage

```sh
forge <url>
```

Output goes to stdout. Pipe it wherever you want.

```sh
# Save to file
forge https://example.com > article.md

# Feed to an LLM
forge https://docs.rs/reqwest | llm "summarize this"

# Batch process
cat urls.txt | xargs -I {} sh -c 'forge {} > "$(echo {} | md5).md"'

# Quick reading in terminal
forge https://blog.rust-lang.org | less
```

## What it strips

Everything that isn't content.

| Category | Elements |
|---|---|
| **Scripts & styles** | `<script>`, `<style>` |
| **Navigation** | `<nav>`, `<footer>`, `<aside>` |
| **Embeds** | `<iframe>`, `<svg>`, `<canvas>` |
| **Interactive** | `<form>`, `<noscript>`, `<template>` |
| **Cookie banners** | Elements with `cookie`, `consent`, `banner` in class/id |
| **Ads & tracking** | Elements with `ad-`, `ads-`, `advert`, `tracking` in class/id |
| **Modals & popups** | Elements with `modal`, `popup`, `overlay` in class/id |
| **Hidden elements** | `aria-hidden="true"`, `hidden`, `display:none`, `visibility:hidden` |
| **ARIA roles** | `role="banner"`, `role="navigation"`, `role="dialog"` |

## What it keeps

Everything that is content.

```
article, main, h1-h6, p, a, img, ul, ol, li,
blockquote, pre, code, table, thead, tbody, tr, th, td,
strong, em, b, i, figure, figcaption, sup, dl, dt, dd
```

## Markdown mapping

| HTML | Markdown |
|---|---|
| `<h1>` ... `<h6>` | `#` ... `######` |
| `<p>` | Double newline |
| `<a href="url">text</a>` | `[text](url)` |
| `<img src="url" alt="text">` | `![text](url)` |
| `<strong>`, `<b>` | `**bold**` |
| `<em>`, `<i>` | `*italic*` |
| `<code>` | `` `inline` `` |
| `<pre><code>` | ` ```block``` ` |
| `<blockquote>` | `> quoted` |
| `<li>` | `- item` |
| `<table>` | Pipe table |
| `<sup>` | `^(superscript)` |
| `<figure>` + `<figcaption>` | Image + `*caption*` |
| `<br>` | Newline |
| `<hr>` | `---` |

## Content detection

Forge looks for content in this order:

1. **`<article>`** - preferred, usually the main content
2. **`<main>`** - fallback, typically wraps the primary area
3. **`<body>`** - last resort, minus nav/header/footer/aside

## Constraints

By design.

| | |
|---|---|
| **Binary size** | 2.3 MB |
| **Source files** | 1 (`src/main.rs`) |
| **Dependencies** | 3 (`reqwest`, `html5ever`, `markup5ever_rcdom`) |
| **Config files** | 0 |
| **CLI flags** | 0 |
| **Lines of code** | ~460 (excluding tests) |

## Tests

141 tests. All pass.

```
$ cargo test

running 121 tests                    # Unit tests
test result: ok. 121 passed

running 20 tests                     # Integration (live URLs)
test result: ok. 20 passed
```

**Unit tests** cover every markdown mapping, every stripped element, every hidden-element heuristic, content priority logic, edge cases (empty HTML, deeply nested DOMs, special characters, entities), and output quality (no excess blank lines, no trailing whitespace).

**Integration tests** hit three live URLs and verify real-world output:
- [Wikipedia: Stigmergy](https://en.wikipedia.org/wiki/Stigmergy) - article structure, headings, links, bold, references, no nav/scripts
- [Hacker News](https://news.ycombinator.com) - content presence, links, no scripts
- [Rust Blog](https://blog.rust-lang.org) - content, links, no HTML leaks, output formatting

## Dependencies

| Crate | Purpose |
|---|---|
| [`reqwest`](https://crates.io/crates/reqwest) | HTTP client with `rustls-tls` (no OpenSSL) |
| [`html5ever`](https://crates.io/crates/html5ever) | Browser-grade HTML parser (same as Servo) |
| [`markup5ever_rcdom`](https://crates.io/crates/markup5ever_rcdom) | DOM tree for html5ever |

No runtime dependencies. No C libraries. Statically linked.

## License

MIT

---

<p align="center">
  <br>
  <code>forge</code> turns the web into text.
  <br>
  Nothing more. Nothing less.
  <br>
  <br>
</p>
