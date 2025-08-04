# Static Site Generator

Initial template based off of this article: https://kerkour.com/rust-static-site-generator

## Features

- Converts Markdown files to HTML
- Watches for changes in the `content/` directory and rebuilds automatically

## Usage

1. Place your Markdown files in the `content/` directory.
2. Create your homepage `index.html` and place in the `content/` directory.
3. The links to html generated from your `.md` files will replace `{{LINKS}}` in your index.html
4. An `index.css` file placed in the `content/` directory will be applied

## Example

```
content/
    index.html
    index.css
    about.md
    blog/
        programming/
        movies/
        myfirstblogpost.md
```

## License

MIT