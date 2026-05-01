# Architecture

## 概要

`vurl` は次の 3 要素で構成されます。

- React + Vite frontend
- Rust + Actix Web backend
- `$HOME/.vurl` 配下の YAML 定義と Markdown ログ

通常利用では backend が frontend の build 成果物を静的配信し、ブラウザ入口は `http://127.0.0.1:1357` です。

## 実行構成

通常利用:

- backend: `127.0.0.1:1357`
- frontend: 独立サーバなし
- ブラウザ: `http://127.0.0.1:1357`

開発時:

- backend: `make -C src/backend dev`
- frontend: `make -C src/frontend dev`
- frontend 開発サーバは `3000`
- backend は frontend 開発サーバからの CORS を許可する

## データ配置

```text
src/
  backend/
  frontend/

$HOME/.vurl/
  defs/
    <project>/
      requests/
        **/*.yaml
      environments/
        <environment>.yaml
        auth.yaml
  logs/
    <project>/
      <timestamp>.md
```

## 責務分割

backend の責務:

- `$HOME/.vurl/defs` のロード
- project / environment / request 一覧 API の提供
- request 定義の読み出し
- 変数展開
- 認証実行
- HTTP リクエスト送信
- Markdown ログ出力
- frontend `dist` の静的配信
- YAML reload

frontend の責務:

- project 選択
- request ツリー表示とフィルタ
- request 定義の読込
- environment 選択
- 認証入力モード切替
- request 編集と送信
- response 表示
- YAML reload 実行

## 実行時状態

永続化されるもの:

- YAML 定義
- Markdown ログ

backend メモリに保持するもの:

- project ごとの定義キャッシュ
- environment ごとの実行時 variable 値
- project ごとの現在アクティブなログファイル

認証で更新された variable は backend メモリ上の環境状態に反映され、以後の送信に使われます。

## 認証と送信の流れ

1. frontend が request 定義を読込む
2. ユーザが environment と認証入力モードを選ぶ
3. `auth: true` かつ認証が必要な場合、backend が認証を実行する
4. request の `path` / `query` / `headers` / `body` を変数展開する
5. backend が HTTP 送信する
6. backend が request / response を Markdown ログへ追記する
7. frontend が response を表示する

認証方式:

- `fixed`: 認証 API は呼ばず、ID に応じた variable 更新だけを行う
- `http`: 認証 API を呼び、レスポンスから variable を inject する

本リクエストが `401` または `403` を返した場合は、1 回だけ再認証して再送します。

## ログ

ログは `$HOME/.vurl/logs/<project>/` に `.md` で出力します。

- 日次ログは `YYYYMMDD000000.md`
- 手動新規ログは `YYYYMMDDHHMMSS.md`
- 1 回の request/response を 1 つのコードフェンスで記録
- 通常 request ログでは mask 対象値を置換
- 認証 request の raw ログは mask せずに記録

## 主要コード

- backend 起動: `src/backend/src/app`
- backend API: `src/backend/src/handlers`
- backend 実行処理: `src/backend/src/services`
- YAML ロードと状態管理: `src/backend/src/runtime`, `src/backend/src/state`
- YAML 型: `src/backend/src/domain`
- frontend エントリ: `src/frontend/src/app`
- frontend API クライアント: `src/frontend/src/api`
- frontend UI: `src/frontend/src/components`
- frontend 状態変換: `src/frontend/src/features`
