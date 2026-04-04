# vurl 仕様

## 1. 目的

`vurl` は、Postman 風の API 実行ツールである。

- UI は React で構築する
- API 実行本体は Rust + Actix で構築する
- リクエスト定義は `1 request = 1 YAML` で管理する
- 環境切替、認証、自動リトライ、ログ保存を備える

## 2. 想定ユースケース

- 保存済みリクエスト定義を一覧から選んで実行する
- 同じリクエスト定義を `local` `stg` `prod` などの環境で切り替えて実行する
- 環境変数を URL、query、header、body に埋め込む
- 認証が必要な API は、必要時のみ自動認証して実行する
- リクエストログとレスポンス本文ログを Markdown 形式で保存する

## 3. スコープ

### 3.1 初期スコープ

- HTTP メソッド: `GET` `POST` `PUT` `PATCH` `DELETE`
- body 形式: `x-www-form-urlencoded` `application/json`
- 保存済み定義の一覧表示
- フィルタリング
- 定義の読み込み
- リクエスト編集
- レスポンス表示
- プロジェクト切替
- 環境切替
- 認証のオンオフ
- 認証失敗時の 1 回だけの再認証 + リトライ
- curl 形式ログ出力

### 3.2 初期スコープ外

- YAML の作成、削除、複製を UI から行う機能
- 認証定義の UI 編集
- GraphQL、multipart/form-data、file upload
- Cookie 管理の高度機能
- 履歴一覧やタブ管理

## 4. 画面仕様

画面は 3 ペイン構成とする。

- 左: サイドバー
- 中央: リクエストエリア
- 右: レスポンスエリア

### 4.1 サイドバー

表示内容:

- プロジェクト切替 UI
- 環境切替 UI
- 保存済みリクエスト定義のツリー表示
- フィルタ入力
- 新規ログファイル作成ボタン

できること:

- プロジェクトの切替
- フィルタ文字列による絞り込み
- YAML 定義のクリック読み込み
- ディレクトリ階層に沿ったツリー表示

表示ルール:

- プロジェクト配下の任意階層ディレクトリをそのまま表示する
- YAML ファイルのみリクエスト定義として表示する
- フィルタはファイル名、相対パス、`name` を対象にしてよい

### 4.2 リクエストエリア

表示内容:

- HTTP メソッド
- URL
- query parameter 一覧
- header 一覧
- body 入力
- 認証 ON/OFF
- ID 入力欄
- PW 入力欄
- 実行ボタン

できること:

- YAML 定義を読み込んで初期値を表示する
- query parameter を編集する
- header を編集する
- body を編集する
- body 形式を `form` または `json` として扱う
- `form` は key-value 編集とする
- `json` はフリーテキスト編集とする
- 認証 ON/OFF を切り替える
- ID/PW を入力して送信時に認証へ利用する

補足:

- URL、query、header、body の各値には環境変数埋め込みを許可する
- 変数展開は送信直前に Rust 側で行う

### 4.3 レスポンスエリア

表示内容:

- status code
- response header
- response body

できること:

- 実行結果の表示
- エラー時の内容表示
- 認証リトライが発生したかの補助表示

## 5. データ管理

## 5.1 プロジェクト

プロジェクトは、リクエスト定義群を束ねる単位である。

- 複数プロジェクトを持てる
- 現在アクティブなプロジェクトを切り替えられる
- プロジェクトごとにリクエスト定義 YAML のルートディレクトリを持つ
- プロジェクトごとに認証定義を持つ
- サイドバーは現在プロジェクト配下のみ表示する

ディレクトリ構成:

```text
~/.vurl/
  logs/
    <project>/
      <yyyymmddhhmmss>.md
  defs/
    <project>/
      requests/
        <name>.yaml
      environments/
        <name>.yaml
        auth.yaml
```

## 5.2 環境

環境は、同じリクエスト定義に対する実行先や値差分を切り替える単位である。

- 例: `local` `dev` `stg` `prod`
- 環境ごとに変数と定数を定義できる
- 変数は認証レスポンスなどで更新されうる
- 定数は固定値として扱う
- 各変数はログ出力時マスク値を持てる
- `mask` は任意項目であり、未指定時はマスクしない

用途:

- base URL 切替
- 認証トークンの保存
- tenant ID などの固定値切替

環境定義ファイル形式:

- YAML とする
- 配置は `~/.vurl/defs/<project>/environments/<name>.yaml`
- マスク設定を含む

## 5.3 YAML 定義

前提:

- `1 request = 1 YAML`
- 作成、削除、複製は当面 UI では行わない
- ファイル操作はユーザーが Vim などで直接行う
- 同一メソッド + 同一パスの定義が複数存在してもよい
- 配置は `~/.vurl/defs/<project>/requests/<name>.yaml`

最低限必要な項目:

```yaml
name: Get User
method: GET
path: /api/users/{user_id}
auth: true
request:
  query:
    - key: verbose
      value: "1"
  headers:
    - key: Accept
      value: application/json
  body:
    type: json
    text: |
      {}
```

項目:

- `name`: 表示名
- `path`: 環境の base URL を除いたパス
- `auth`: 認証を使うかどうかの真偽値
- `request.query`: 初期 query
- `request.headers`: 初期 header
- `request.body.type`: `json` or `form`
- `request.body.text`: `json` 用初期本文
- `request.body.form`: `form` 用初期 key-value

`form` 例:

```yaml
request:
  body:
    type: form
    form:
      - key: username
        value: "{{login_id}}"
      - key: password
        value: "{{login_password}}"
```

## 6. 変数展開仕様

埋め込み構文は単一形式に寄せる。

候補:

- `{{base_url}}`
- `{{token}}`
- `{{user_id}}`

展開対象:

- URL
- path
- query value
- header value
- body value

値の優先順位案:

1. リクエスト画面で現在入力中の値
2. 環境変数
3. 環境定数
4. YAML の初期値

未定義変数があった場合:

- 送信前エラーとして扱う
- どの項目で未解決かを UI に表示する

## 7. 認証仕様

要件:

- 認証の要否はリクエストエリアでオンオフできる
- ID/PW 入力欄がある
- 認証定義はプロジェクトごとかつ環境ごとである
- 認証定義ファイルは `~/.vurl/defs/<project>/environments/auth.yaml` とする
- 認証成功時、レスポンスの一部を所定の環境変数へ保存する
- 認証トークン切れなどで失敗した場合、自動で 1 回だけ再認証してリトライする
- 認証方式として「固定値返却モード」と「HTTP リクエストモード」を持つ

動作案:

1. 実行時に認証 ON なら、必要に応じて認証情報を付与する
2. 本リクエストが認証切れ相当の失敗を返したら再認証する
3. 認証成功後、保存ルールに従って環境変数を更新する
4. 元のリクエストを 1 回だけ再送する

認証失敗判定:

- HTTP `401`
- HTTP `403`

認証定義の設定項目:

- 対象環境名
- モード
- 認証 URL
- メソッド
- body テンプレート
- ID/PW の埋め込み位置
- レスポンスのどの項目をどの環境変数へ inject するか

モード:

- `fixed`: React で入力した ID/PW に応じて固定値を返す
- `http`: 認証 API に実リクエストを送る
- `fixed` は `default` マッピングを持てる

inject 例:

- レスポンス JSON の `token` を `auth_token` に保存
- 以後の header `Authorization: Bearer {{auth_token}}` に利用

固定値返却モード例:

- `local` では ID/PW を受け取り、設定済みの固定トークンを `auth_token` に inject する
- `stg` では `/auth` に送信して返却値を `auth_token` に inject する

## 8. ログ仕様

要件:

- JST で日付が変わるごとにログファイルを新規作成する
- デフォルトファイル名は `yyyymmdd000000`
- クライアントから手動で新規ログファイルを作成できる
- 手動作成時は `yyyymmddhhmmss`
- リクエストは curl 形式で保存する
- レスポンス本文はそのまま文字列として保存する
- ログはリクエスト部分をコピペでそのまま実行できること
- 拡張子は `.md` とする
- 1 リクエストごとにコードフェンスで囲む

設計:

- 1 つのログファイルに複数回の実行ログを追記する
- 1 リクエストごとに JST タイムスタンプをコメントする
- マスク対象変数は実値でなく事前設定のマスク値を出力する
- レスポンス本文は JSON でも平文でも、加工せず文字列として保存する
- curl とレスポンス本文は同一コードフェンス内に連続して書く

出力イメージ:

````md
# 2026-04-04 21:30:10 JST
```bash
curl -X GET 'http://localhost:8080/api/users/1?verbose=1' \
  -H 'Accept: application/json' \
  -H 'Authorization: Bearer xxx'
{"id":1,"name":"Alice"}
```
````

## 9. Rust バックエンド仕様

役割:

- React からリクエスト内容を受け取る
- YAML 定義を読み込む
- 環境変数を展開する
- 必要に応じて認証する
- 実際の HTTP リクエストを送信する
- レスポンスを React に返す
- ログを出力する
- VPN オンオフ後もプロセス再起動なしでリクエストできる

構成:

- 親プロセス: ターミナル標準入力を監視する
- 子プロセス: Actix サーバ

親プロセスの責務:

- `c` 入力で YAML チェックを実行する
- `c` は全プロジェクト配下の全 YAML を検査する
- `r` 入力で Actix サーバを再起動する

子プロセスの責務:

- HTTP API 提供
- 定義読込
- リクエスト送信
- ログ出力
- OS の現在ネットワーク状態に追随して毎回リクエストを解決する

必要 API:

- `GET /api/projects`
- `POST /api/projects/select`
- `GET /api/environments`
- `GET /api/tree`
- `GET /api/definition?path=...`
- `POST /api/send`
- `POST /api/logs/new`
- `GET /api/runtime`

## 10. React フロントエンド仕様

役割:

- プロジェクト一覧取得
- 現在プロジェクトのツリー取得
- YAML 定義表示
- 実行用フォーム編集
- バックエンド API 呼び出し
- レスポンス表示

状態管理の最小単位:

- currentProject
- currentEnvironment
- selectedDefinitionPath
- filterText
- requestDraft
- responseView
- authEnabled
- authCredential
- currentLogFile

## 11. CLI 仕様

要件:

- `vurl` でサーバ起動し、ブラウザで localhost を開く
- `vurl -l` でログディレクトリへ `cd` できる
- `vurl -y` で定義ディレクトリへ `cd` できる
- `zsh` 前提でよい
- Rust のプロダクションビルド名は `vurl-backend` とする
- `zsh` から `vurl.zsh` をロードして `vurl` 関数を提供する
- 引数なしの `vurl` は `vurl-backend` 起動とブラウザ open を行う
- `vurl-backend` は定義ルートディレクトリを起動引数で指定できる

実現案:

- `vurl.zsh` に `vurl` 関数を定義する
- `-l` はログディレクトリへ `cd` する
- `-y` は定義ディレクトリへ `cd` する
- 引数なしは `vurl-backend` をバックグラウンド起動し、フロントエンド URL を open する
- フロントエンドは `localhost` 上で動作し、ポートは固定しない
- `r` はバックエンドのみ再起動する
- 例: `vurl-backend --root ~/.vurl`

## 12. エラーハンドリング

初期方針:

- YAML パース失敗時は一覧読込時またはチェック時にエラー表示する
- 送信前の変数未解決は送信エラーにする
- 認証失敗後の再送は 1 回のみ
- ログ出力失敗は本リクエスト結果とは分離して UI に警告表示する
- VPN 切替直後の一時的な通信失敗は通常の通信エラーとして扱い、次回送信時は再起動不要で回復可能とする

## 13. 実装メモ

- 環境 YAML は `constants` と `variables` を分ける
- `variables.{key}.mask` は任意項目とする
- 認証定義は `environments.<name>` のマップ形式にする
- `fixed` は `mappings` と `default` を持てる
- `response.inject` は認証レスポンスの一部を環境変数へ反映する

## 14. MVP 提案

最初の実装は以下に絞るとよい。

1. 単一プロジェクト対応
2. 環境切替
3. YAML 一覧表示
4. 定義読み込み
5. `GET/POST` + `query/header/json body`
6. 送信とレスポンス表示
7. 変数展開
8. リクエスト curl ログ + レスポンス本文ログ出力

その次:

1. 複数プロジェクト対応
2. 認証 ON/OFF
3. 再認証 + リトライ
4. `form-urlencoded`
5. 親子プロセス管理

## 15. 推奨する次の作業

実装前に次を確定すると詰まりにくい。

1. リクエスト定義 YAML の Rust 構造体
2. 環境 YAML の Rust 構造体
3. 認証 YAML の Rust 構造体
4. `vurl-backend --root ...` の CLI 引数仕様
5. ログ出力フォーマッタの実装
