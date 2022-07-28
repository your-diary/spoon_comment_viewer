# 1. About

配信サイト[Spoon](https://www.spooncast.net/jp/)のコメントビューア

Cross-platform CUI comment viewer and assistant bot for [Spoon](https://www.spooncast.net/jp/).

Fully written in Rust.

# 2. Requirements

- [Rust](https://www.rust-lang.org/)

- [`geckodriver`](https://github.com/mozilla/geckodriver)

# 3. Usage

```bash
$ cargo run
```

# 4. Configurations

## 4.1 Configuration File

`./config.json` is read as a configuration file. A template is included in this repository:

```bash
$ cp ./config_template.json ./config.json
```

## 4.2 Example

```json
{
    "twitter_id": "example",
    "twitter_password": "ejeew#!jfe35AB",
    "comment_check_interval_ms": 1000,
    "listener_check_interval_ratio": 2,
    "should_comment_listener": false,
    "should_comment_heart": false,
    "should_comment_spoon": false,
    "webdriver_port": 4444,
    "implicit_timeout_ms": 5000
}
```

<!-- vim: set spell: -->

