# Backend Spec

## 役割

- `$HOME/.vurl/defs` の YAML をロードする
- 実行時の環境変数状態を保持する
- React から受けた定義を実行する
- 認証とログ出力を行う
- frontend build を静的配信する

## API

- `GET /api/runtime`
- `GET /api/projects`
- `GET /api/environments?project=<project>`
- `GET /api/tree?project=<project>`
- `GET /api/definition?project=<project>&path=<path>`
- `POST /api/send`
- `POST /api/logs/new`

## リクエスト実行

- `base_url` は環境定義の `constants.base_url.value` を使う
- `path`, `query`, `headers`, `body` の値を送信直前に展開する
- 未定義変数がある場合は送信前エラーにする
- body は `json` と `form` を扱う

## 変数解決

解決対象:

- `path`
- `query.value`
- `headers.value`
- `body`

参照元:

- 認証入力: `{{auth.id}}`, `{{auth.password}}`
- 環境変数
- 環境定数

未定義変数はエラーにする。

## 認証

認証はリクエスト単位の `auth: true/false` で切り替える。

`auth: true` の場合:

- 送信前に認証を実行する
- `fixed` は固定値を環境変数へ反映する
- `http` は認証 API を呼ぶ

認証失敗判定:

- `401`
- `403`

本リクエストで `401` または `403` を受けた場合は、1 回だけ再認証して再送する。

## ログ

- ログ先は `$HOME/.vurl/logs/<project>/`
- 日付切替時は JST 基準で新しい日次ファイルを作る
- 手動でも新規ログファイルを作れる
- 拡張子は `.md`
- 1 リクエストごとに 1 つのコードフェンスで記録する

ログ内容:

- 実行可能な `curl` 形式のリクエスト
- HTTP `status code`
- レスポンス本文の生文字列

`response headers` はログへ出さない。

## 静的配信

- backend は frontend の `dist` を配信する
- production のブラウザ入口は `http://127.0.0.1:1357`

## CLI と起動

公開インターフェースは `bin/vurl.zsh` から使う `vurl` 関数を前提とする。

backend 自体は:

- supervisor として起動する
- 子プロセスとして Actix サーバを起動する

固定値:

- root: `$HOME/.vurl`
- host: `127.0.0.1`
- port: `1357`
