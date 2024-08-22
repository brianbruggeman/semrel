# Book

## Dependencies

We need to install mdbook and mdbook-pdf to build a pdf of the book.

```bash
$ cargo install mdbook mdbook-pdf
```

## Build

To build the book, run the following command:

```bash
$ mdbook build
```

The book will be built in the `book/pdf` directory.
```bash
$ open book/pdf/output.pdf
```


## Viewing

Alternatively, you can view the book in your browser by running:

```bash
$ mdbook serve
```

You can then point your browser to: http://[::1]:3000