## 🦀 pasties

`pasties` is a small markdown-equipped pastebin site written in rust. it uses tokio and axum to handle async routing and the api. the frontend is generated with askama templates, and interacts with the api using `htmx`. 

pasties also depends on `pulldown-cmark` for rendering markdown, although a custom renderer is in the works to add extra functionality.

pasties is currently under heavy developement, the following functionality is still missing:

- a given paste's custom url and password cannot be edited through the frontend gui yet
- pastes cannot be deleted through the frontend gui yet

### running

pasties' developer dependencies are only `cargo` and a compiler for `sass`, although `cargo-watch` is recommended for the following command:

```
cargo watch -x run -w ./src/ -w ./templates/ -c -q
```

### contributions

please feel free to fork the repository and open a pull request if you feel like you have useful additions! for giving recommendations, you may also open an issue on github, or message the maintainer on discord: `@twoespresso`