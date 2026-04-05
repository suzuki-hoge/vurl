# Frontend Spec

## 画面構成

3 ペイン構成:

- 左: サイドバー
- 中央: リクエストエリア
- 右: レスポンスエリア

## サイドバー

表示:

- project 選択
- environment 選択
- filter 入力
- リクエストツリー
- 新規ログファイルボタン

操作:

- project 切替
- environment 切替
- フィルタ
- リクエスト定義を開く

## リクエストエリア

表示:

- HTTP method
- path
- query
- headers
- body
- auth ON/OFF
- id / password 入力
- 送信ボタン

body 編集:

- `form`: key-value 編集
- `json`: フリーテキスト編集

## レスポンスエリア

表示:

- status code
- response headers
- response body
- 認証リトライ有無
- 現在のログファイル

## 接続先

- production では `window.location.origin` を API 接続先にする
- `VITE_BACKEND_URL` がある場合はそれで上書きできる

## 現状の制約

- 単一タブ前提
- ページ状態の永続化なし
- UI レイアウトと入力 UX は今後改善余地あり
