# Backend API

## 概要

backend は `127.0.0.1:1357` で待ち受け、YAML 定義の読込、request 実行、認証、ログ出力、frontend の静的配信を担当します。

固定値:

- host: `127.0.0.1`
- port: `1357`
- root: `$HOME/.vurl`
- request timeout: `3000ms`

## エンドポイント一覧

- `GET /api/runtime`
- `GET /api/projects`
- `GET /api/environments?project=<project>`
- `GET /api/tree?project=<project>`
- `GET /api/definition?project=<project>&path=<path>`
- `POST /api/send`
- `POST /api/logs/new`
- `POST /api/reload`

`/` と `/{path:.*}` は frontend build 成果物の配信に使います。

## GET /api/runtime

返却内容:

- `root`: backend が見ているルートディレクトリ
- `projects`: project 一覧
- `backend_url`: frontend が参照する backend URL

## GET /api/projects

project 名一覧を返します。名前順です。

## GET /api/environments

query:

- `project`

返却内容:

- environment 一覧
- 各 environment で選択可能な `auth_presets`

environment の並び順:

- `order` があるものを昇順
- `order` が同じ、または未指定同士なら名前順

## GET /api/tree

query:

- `project`

返却内容:

- request ツリー
- node は `directory` または `request`

request node が持つ情報:

- `name`: ファイル名
- `path`: project ルートからの相対 path
- `title`: YAML の `name`
- `method`: YAML の `method`

## GET /api/definition

query:

- `project`
- `path`

返却内容:

- `path`
- `definition`: request YAML をそのまま表す定義

## POST /api/send

request body:

- `project`
- `environment`
- `path`
- `method`
- `url_path`
- `query`
- `headers`
- `body`
- `auth_enabled`
- `auth_input_mode`: `preset` または `manual`
- `auth_preset_name`
- `auth_credentials.id`
- `auth_credentials.password`

送信前処理:

- environment 実行時状態を取得
- 必要なら認証を実行
- `url_path`, `query`, `headers`, `body` を変数展開
- `constants.base_url.value` を使って URL を組み立て

認証の実行条件:

- request の `auth_enabled` が `true`
- `fixed` は毎回認証を実行
- `http` は `response.inject.to` のいずれかが未設定または空文字なら事前認証を実行

再試行:

- 本 request のレスポンスが `401` または `403` のときだけ 1 回再認証して再送

成功レスポンス:

- `status`
- `headers`
- `content_type`
- `body`
- `body_base64`
- `retried_auth`
- `notifications`
- `current_log_file`

`body_base64`:

- 画像などバイナリ応答を frontend で表示するための補助値

`notifications`:

- `authenticated`: 自動認証を実行した
- `timeout`: request timeout
- `generic`: 汎用通知

エラー時:

- timeout は `500` と JSON body を返す
- それ以外の送信エラーは `400` と `{ "message": "..." }` を返す

## POST /api/logs/new

request body:

- `project`

返却内容:

- `project`
- `current_log_file`

`$HOME/.vurl/logs/<project>/YYYYMMDDHHMMSS.md` を新規作成し、その file を active log に設定します。

## POST /api/reload

YAML を再読込して store を置き換えます。

成功時:

- `success: true`
- `message`
- `project_count`

失敗時:

- `400`
- `{ "message": "..." }`

reload に失敗した場合、以前の store は維持されます。

## ログ出力

request 実行時は `$HOME/.vurl/logs/<project>/` に Markdown を追記します。

- 通常 request は mask 適用後の `curl` と response body を記録
- auth request は raw の `curl` と response body を記録
- JSON response body は pretty print して記録
- response headers はログへ出さない

## backend が持つ仕様境界

この文書は外部仕様を扱います。YAML の詳細 schema は [yaml-spec.md](./yaml-spec.md)、全体設計は [architecture.md](./architecture.md) を参照してください。
