# Frontend UI

## 概要

frontend は 1 画面で project 選択、request 選択、request 編集、送信、response 表示を行います。production では backend と同じ origin から配信されます。

API 接続先:

- 既定値: `window.location.origin`
- 上書き: `VITE_BACKEND_URL`

## 画面構成

project 未選択時:

- project 一覧だけを表示

project 選択後:

- 左: request ツリー
- 中: request 編集
- 右: response 表示

## URL と画面状態

URL 仕様:

- pathname 先頭要素を project 名として扱う
- query parameter `path` を request path として扱う

例:

- `/<project>`
- `/<project>?path=users%2Fget-user.yaml`

挙動:

- ブラウザ再読込時は URL から project と request を復元する
- `history.pushState` で request path を更新する
- `popstate` に追従する

## 左ペイン

表示:

- request フィルタ入力
- `Reload YAML` ボタン
- request ツリー

挙動:

- フィルタは path と name に対してかかる
- フィルタ中は該当ディレクトリを展開状態で扱う
- `Reload YAML` 成功時は `window.location.reload()` する

## 中央ペイン

固定領域:

- request method
- request name
- path 入力
- auth 操作

スクロール領域:

- `Query`
- `Headers`
- `Body`

### request 読込

`GET /api/definition` の結果から draft を生成します。

- `method` は表示のみ
- `name` は表示のみ
- `path` は編集可能
- `query` は key/value 編集
- `headers` は key/value 編集
- `body.type` が `json` なら JSON テキスト編集
- `body.type` が `form` なら form editor を表示

### header 補完

body が `json` で、かつ `Content-Type` header が未設定なら、frontend は送信時に `Content-Type: application/json` を自動挿入します。この header は UI 上では locked 扱いです。

### auth UI

request 定義の `auth` が `true` のときだけ表示します。

入力モード:

- `preset`
- `manual`

挙動:

- environment に preset がある場合、初期値は `preset`
- preset がない場合、初期値は `manual`
- `preset` では preset select を表示
- `manual` では `ID` と `Password` を表示
- `Password` input は `text`

### 送信

- 中央ペイン全体は `form`
- `Send` ボタン、または単一行 input 上の Enter で送信
- `textarea` 内の Enter は改行

## 右ペイン

固定領域:

- `Status`
- `Retried Auth`
- 現在のログファイル

スクロール領域:

- `Body`
- `Headers`
- `Code`

response body 表示:

- JSON 系 content type: 整形表示
- image 系 content type かつ `body_base64` あり: 画像表示
- それ以外: 生文字列表示

通知:

- 認証実行時: `Authenticated`
- timeout: `Timed out`
- 通常応答: `<status> Received`

## frontend の状態範囲

- project, request path は URL と同期
- request draft と response はタブごとのメモリ state
- localStorage などの永続化は持たない

## 実装上の主要ファイル

- 画面全体: `src/frontend/src/app/App.tsx`
- request draft 変換: `src/frontend/src/features/request/model.ts`
- response 表示判定: `src/frontend/src/features/response/model.ts`
- URL 同期: `src/frontend/src/lib/location.ts`
