# 1. About

配信サイト[Spoon](https://www.spooncast.net/jp/)のチャットボット(自動マネージャー)機能付きコメントビューア

以下のような機能を持つ。

- コメントビューア機能

- チャットボット (自動マネージャー)

    - 入室コメ (「いらっしゃい」)

    - 退室コメ (「またきてね」)

    - 再入室コメ (「おかえりなさい」)

    - ハーコメ (「ハートありがとう」)

    - スプーンやバスターのお礼 (「スプーンありがとう」)

- 人工知能による自動配信 (ChatGPTやCoeFontとの連携によるコメントへの自動応答)

- CUI

- cross-platform (Windows, macOS, Linux対応)

- Fully written in Rust.

問い合わせ先: [@ynn_diary](https://twitter.com/ynn_diary)

| [![](./readme_assets/demo.png)](https://www.youtube.com/watch?v=mWLUacHuatY) |
| :-: |
| Demo (YouTube) |

# 2. 使い方

## 2.1 前提

以下がインストールされていること。

- [Rust](https://www.rust-lang.org/)

- [`geckodriver`](https://github.com/mozilla/geckodriver)

## 2.2 設定

プロジェクト配下の`./config.json`が設定ファイルです。テンプレートが同梱されているので、以下のコマンドでコピーして使ってください。

```bash
$ cp ./config_template.json ./config.json
```

**例:**

`twitter`オブジェクトには、SpoonにログインするためのTwitterのIDとパスワードを設定します。それ以外の設定はデフォルト値のままで大丈夫です。

```json
{
    "twitter": {
        "id": "example",
        "password": "ejeew#!jfe35AB"
    },
    "spoon": {
        "url": "https://www.spooncast.net/jp/",
        "comment_check_interval_ms": 1000,
        "listener_check_interval_ratio": 2,
        "should_comment_listener": true,
        "should_comment_heart": true,
        "should_comment_spoon": true,
        "should_comment_guide": true
    },
    "selenium": {
        "webdriver_port": 4444,
        "implicit_timeout_ms": 5000
    },
    "chatgpt": {
        "enabled": false,
        "project_dir": "~/Build/chatgpt",
        "excluded_user": "DJりな"
    }
}
```

<sub>(`chatgpt`オブジェクトについてはundocumentedですが、Rustで書かれていてかつ対話的なChatGPTクライアントのRustプロジェクトを`chatgpt.project_dir`フィールドに指定すれば、リスナーのコメントに対してChatGPTで自動返信することができます。クライアントはどこかで公開されているわけではなく、各自で実装する必要があります。)</sub>

## 2.3 実行

```bash
$ geckodriver
$ cargo run --release
```
