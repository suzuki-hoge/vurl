# Architecture

## 概要

`vurl` は次の 3 要素で構成する。

- React + Vite frontend
- Rust + Actix backend
- `$HOME/.vurl` 配下の YAML / ログ

production 利用時は、backend が frontend の build 成果物も静的配信する。

## 実行構成

production:

- backend: `127.0.0.1:1357`
- frontend: 独立サーバなし
- ブラウザは `http://127.0.0.1:1357`

development:

- backend: `make -C src/backend dev`
- frontend: `make -C src/frontend dev`
- frontend dev server は `1357`

## ディレクトリ

```text
src/
  backend/
  frontend/

$HOME/.vurl/
  defs/
    <project>/
      requests/
      environments/
        auth.yaml
  logs/
    <project>/
      <yyyymmddhhmmss>.md
```

## Backend

backend の責務:

- YAML 読込
- プロジェクト / 環境 / リクエスト一覧 API
- リクエスト実行
- 変数展開
- 認証
- ログ出力
- frontend `dist` の静的配信

主要ディレクトリ:

- `src/backend/src/app`
- `src/backend/src/handlers`
- `src/backend/src/services`
- `src/backend/src/runtime`
- `src/backend/src/domain`
- `src/backend/src/process`

## Frontend

frontend の責務:

- プロジェクト選択
- 環境選択
- リクエストツリー表示
- リクエスト編集
- 実行
- レスポンス表示
- 新規ログファイル作成

主要ディレクトリ:

- `src/frontend/src/app`
- `src/frontend/src/api`
- `src/frontend/src/components`
- `src/frontend/src/features`
- `src/frontend/src/lib`
- `src/frontend/src/types`

## Supervisor

`vurl-backend` は親プロセスと子プロセスに分かれる。

- 親: 標準入力を監視する
- 子: Actix サーバとして待ち受ける

親プロセスの操作:

- `c`: 全 YAML チェック
- `r`: backend 子プロセス再起動
- `q`: 終了

## 設定と状態

永続状態:

- YAML 定義
- Markdown ログ

メモリ状態:

- 認証で更新された環境変数
- 現在のアクティブログファイル

## 認証フロー

- `auth: true` のリクエストは送信前に認証を実行する
- `fixed` は認証リクエストを発生させず、固定値を環境変数へ反映する
- `http` は認証 API を呼び、レスポンスから環境変数へ inject する
- 本リクエストが `401` または `403` の場合は、1 回だけ再認証してリトライする
