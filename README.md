# Small-site

Very small static site generator. It has a very modest feature set, but it's
written in Rust and does everything in one pass over your templates and
content, so it should be fast.


## Usage

Say you have two folders: `src/` and `templates/`, with one file in each:

`src/index.html`:

```html
template=base.html
title=Hello Github
---
<p>Some content</p>
```

`templates/base.html`:
```
<html>
    <body>
        <h1>{{title}}</h1>
        {{content}}
    </body>
</html>
```

If you run `small-site` it should then create a third folder `/public` where it
puts what you'd expect. The program copies the html and markdown files from the
`/src` directory to `/public`, while parsing the header, finding the template
and replacing any variables set at the top.

Run `small-site -h` for command line options.


## Installation

Since this is just a program I made for my own personal needs, there is
currently no simple way to install it unless you have Rust installed. If you
do, clone the project and run `cargo build --release`. The binary can be found
at `/target/release/small-site`. So just put that somewhere what it's easy for
you to run it from.
