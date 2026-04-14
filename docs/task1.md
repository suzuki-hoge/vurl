# Task 1

## 方針

- backend の対話 supervisor は廃止する
- YAML 反映は React 画面の reload ボタンから行う
- reload ボタン押下時は、まず backend の reload API の成否を受け取る
- backend が成功を返したときだけ `window.location.reload()` でブラウザ全体を再読み込みする
- backend が失敗を返したときは、YAML 書式エラーなどの内容を画面に表示し、ブラウザ再読み込みはしない
- `vurl --reboot` は削除する

## 目的

- backend の stdin / FIFO / supervisor コマンド入力をなくして構造を単純化する
- YAML を編集したあとの反映導線を UI に寄せる
- reload 失敗時に壊れた状態へ切り替えず、直前の正常状態を維持する

## backend の変更

### 1. supervisor 廃止

- `src/backend/src/process/supervisor.rs` を削除する
- `src/backend/src/main.rs` から parent / child の分岐を削除し、常に通常の HTTP server を起動する
- `src/backend/src/cli/mod.rs` から `--child` を削除する

### 2. RuntimeStore の差し替え可能化

- `AppState` から現在の store を差し替えられるようにする
- `Arc<RuntimeStore>` 固定ではなく、`RwLock<Arc<RuntimeStore>>` 相当の形で保持する
- 各 handler はリクエストごとに現在の store を取得して使う
- `send` 実行中は取得済みの `Arc<RuntimeStore>` を使い続ける

これにより、reload 実行中でも進行中リクエストはその時点の store で完走できる。

### 3. reload API 追加

- `POST /api/reload` を追加する
- 実装内容:
  - `AppPaths::from_default_root()` から新しい `RuntimeStore` を構築する
  - 構築成功時のみ store を差し替える
  - 構築失敗時は現在の store を維持したままエラーを返す

- 返却内容:
  - success
  - message
  - 必要なら project count 程度

### 4. エラー方針

- YAML parse error
- project 構成不備
- auth.yaml 不備

これらは reload API の失敗として返す。

- HTTP status は `400` か `500` のどちらかに統一する
- frontend ではレスポンス本文の message をそのまま表示する

### 5. runtime 状態の扱い

- reload 成功時は `RuntimeStore` を新規作成する
- そのため environment の runtime variables は YAML 初期値へ戻る
- active log 状態も引き継がない

これは「YAML 定義を読み直した結果を、そのまま新しい runtime 全体へ置き換える」という挙動にするため。

## frontend の変更

### 1. reload ボタン追加

- 画面上に `Reload YAML` ボタンを追加する
- 配置候補はサイドバー上部
- 意味としては「現在 project ではなく、defs 全体を再読み込みする」

### 2. reload 処理

- `apiClient.reload()` を追加する
- ボタン押下時のフロー:
  1. reload API を呼ぶ
  2. 失敗したらエラー表示して終了
  3. 成功したら `window.location.reload()` を呼ぶ

### 3. 画面側の責務

- reload 成功後の state 再構築はブラウザ全体 reload に任せる
- React 内で project / tree / draft / response を手作業で同期し直さない
- これにより、reload 後の state 破綻や同期漏れを避ける

### 4. UI/UX

- reload 中はボタンを disabled にする
- 成功 toast は不要
- 失敗時のみエラー表示または toast を出す

成功時は即ブラウザ再読み込みに入るため、成功メッセージを見せる価値が薄い。

## vurl コマンドの変更

### 1. reboot 削除

- `vurl --reboot` を削除する
- usage からも削除する
- README からも削除する

### 2. supervisor 前提の削除

- FIFO
- supervisor pid
- stdin へのコマンド送信

これらを `bin/vurl.zsh` から削除する。

### 3. 起動方式の簡素化

- `vurl`:
  - backend 未起動ならバックグラウンド起動
  - ブラウザを開く
- `vurl --down`:
  - backend を停止
- `vurl --log-dir [project]`:
  - 維持
- `vurl --edit`:
  - 維持

PID 管理は単純な backend 単体プロセス前提に戻す。

## ドキュメント更新

- `README.md`
  - `--reboot` を削除
  - reload ボタン前提の説明に更新
- 必要なら `docs/spec-backend.md`
  - supervisor 記述を削除
  - reload API を追記

## テスト観点

### backend

- 正常な YAML で reload 成功
- 不正 YAML で reload 失敗
- reload 失敗時に現在の store が維持される
- reload 後に project/tree/definition の内容が更新される

### frontend

- reload ボタン押下で API を呼ぶ
- API 成功時に `window.location.reload()` が呼ばれる
- API 失敗時にエラー表示され、`window.location.reload()` は呼ばれない

### CLI

- `vurl` で backend がバックグラウンド起動する
- `vurl --down` で停止できる
- `vurl --reboot` が使えない

## 受け入れ条件

- backend に対話入力を要求するコードが残っていない
- YAML を編集したあと、画面の reload ボタンで反映できる
- YAML が壊れている場合、reload 失敗が画面に表示され、現在の backend 状態は維持される
- reload 成功時はブラウザ全体が再読み込みされる
- `vurl --reboot` は削除されている
