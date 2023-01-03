# 1. About

配信サイト[Spoon](https://www.spooncast.net/jp/)のチャットボット(自動マネージャー)機能付きコメントビューア

以下のような機能を持つ。

- コメントビューア

- チャットボット (自動マネージャー)

    - 入室コメ (「いらっしゃい」)

    - 退室コメ (「またきてね」)

    - 再入室コメ (「おかえりなさい」)

    - ハーコメ (「ハートありがとう」)

    - スプーンやバスターのお礼 (「スプーンありがとう」)

    - 点呼 (配信終了直前)

- 読み上げ機能 (VOICEVOXと連携してハーコメを読み上げるなど)

- BGM再生 (ループ再生やボリュームの設定も可能)

- 連続配信 (設定した内容の枠を次枠として作り続ける)

- 人工知能による自動配信 (ChatGPTやVOICEVOXとの連携によるコメントへの自動応答)

- CUI

- cross-platform (Windows, macOS, Linux対応)

- Fully written in Rust.

問い合わせ先: [@ynn_diary](https://twitter.com/ynn_diary)

| [![](./readme_assets/demo.png)](https://www.youtube.com/watch?v=mWLUacHuatY) |
| :-: |
| デモ動画 (YouTube) |

# 2. 使い方

## 2.1 前提

以下がインストールされていること。

- [Rust](https://www.rust-lang.org/)

- [`geckodriver`](https://github.com/mozilla/geckodriver)

- [`sox`](https://github.com/chirlu/sox) (BGM再生機能や読み上げ機能を使う場合のみ)

## 2.2 設定

プロジェクト配下の`./config.json`が設定ファイルです。テンプレートが同梱されているので、以下のコマンドでコピーして使ってください。

```bash
$ cp ./config_template.json ./config.json
```

**例:**

`twitter`オブジェクトには、SpoonにログインするためのTwitterのIDとパスワードを設定します。

ハーコメなどの読み上げを有効にしたい場合は`voicevox`オブジェクトを設定します。読み上げには[WEB版VOICEVOX API](https://voicevox.su-shiki.com/su-shikiapis/)が使用されます。

それ以外の設定はデフォルト値のままで大丈夫です。

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
        "should_comment_guide": true,
        "should_call_over": true,
        "message_tunnel_file": "~/ramdisk/tunnel.txt",
        "live": {
            "enabled": false,
            "start_url": "https://www.spooncast.net/jp/live/broadcast",
            "genre": "勉強",
            "title": "一緒に勉強しよう!!",
            "tags": [
                "勉強"
            ],
            "pinned_comment": "hello\nworld",
            "bg_image": "~/Downloads/bg.png",
            "bgm": {
                "enabled": false,
                "path": "~/Music/bgm/piano.mp3",
                "volume": 0.03
            }
        }
    },
    "selenium": {
        "webdriver_port": 4444,
        "implicit_timeout_ms": 5000,
        "profile_path": null,
        "should_maximize_window": false
    },
    "voicevox": {
        "enabled": false,
        "should_skip_non_japanese": true,
        "url": "https://api.su-shiki.com/v2/voicevox/audio/",
        "api_key": "z-3_93p-77751-X",
        "speaker": "a8cc6d22-aad0-4ab8-bf1e-2f843924164a",
        "speed": 1.0,
        "output_dir": "./wav",
        "timeout_sec": 10
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

### 2.3.1 通常の実行

```bash
$ geckodriver
$ cargo run --release
```

### 2.3.2 連続配信

```bash
$ geckodriver
$ ./loop.sh
```

