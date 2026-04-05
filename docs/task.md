# vurl 実装計画

## 1. 前提

- Git 操作はユーザーが行う
- Rust は macOS にインストール済みである
- Node.js は `nodenv` で `24.0.0` を使用する
- frontend のパッケージマネージャは `yarn` を使う
- 定義ルートは `$HOME/.vurl` 固定である

## 2. 現在地

現時点で、MVP の骨格は概ね実装済みである。

実装済み:

- `src/backend` に Rust + Actix の backend 骨格
- `src/frontend` に React + Vite の frontend 骨格
- YAML 読込
- プロジェクト一覧、環境一覧、ツリー、定義取得 API
- リクエスト送信
- 変数展開
- `fixed` / `http` 認証
- `401/403` 時の再認証リトライ
- Markdown ログ出力
- 親プロセスによる `c` / `r` / `q` 制御
- `bin/vurl.zsh` の土台
- frontend/backend/ルートの Makefile
- backend から frontend `dist` を静的配信
- Rust の基本 unit test

確認済み:

- `cargo check --manifest-path src/backend/Cargo.toml`
- `cargo test --manifest-path src/backend/Cargo.toml`
- `yarn build` in `src/frontend`
- supervisor の `c` / `r` 動作
- `runtime/tree/definition` API の基本応答

## 3. 完了済みフェーズ

### 3.1 フェーズ 1: リポジトリ初期化

完了:

- `src/frontend` と `src/backend` を作成
- `.node-version` を追加
- frontend/backend のルート構成を整備

### 3.2 フェーズ 2: Rust のスキーマ定義

完了:

- リクエスト YAML 用構造体
- 環境 YAML 用構造体
- 認証 YAML 用構造体
- Serde による YAML パース

### 3.3 フェーズ 3: 定義ルート読込

完了:

- `{root}/defs` `{root}/logs` 解決
- プロジェクト一覧
- リクエストツリー構築
- 環境一覧取得
- `c` による全 YAML チェック

### 3.4 フェーズ 4: Actix API

完了:

- `GET /api/projects`
- `GET /api/environments`
- `GET /api/tree`
- `GET /api/definition`
- `POST /api/send`
- `POST /api/logs/new`
- `GET /api/runtime`

補足:

- `POST /api/projects/select` は採用していない
- クライアントが project を毎回送る設計にしている

### 3.5 フェーズ 5: リクエスト実行基盤

完了:

- 変数展開
- 未定義変数エラー
- query/header/body 送信
- `json` / `form` body 対応
- VPN 切替後も再起動なしで再送可能な前提の実装

### 3.6 フェーズ 6: 認証

完了:

- `auth: true/false` 解釈
- `fixed` モード
- `default` マッピング
- `http` モード
- `response.inject` による環境変数更新
- `401/403` 時の再認証 + 1 回リトライ

### 3.7 フェーズ 7: ログ

完了:

- JST ベースの日次ログ切替
- 手動新規ログ作成
- `.md` 出力
- 1 リクエスト 1 コードフェンス形式
- `mask` 置換

### 3.8 フェーズ 8: 親子プロセス構成

完了:

- 親プロセスで標準入力監視
- `c` で全 YAML チェック
- `r` で backend 子プロセス再起動
- `q` で停止

### 3.9 フェーズ 9: React UI

概ね完了:

- 3 ペインレイアウト
- サイドバーのフィルタとツリー表示
- 環境切替
- リクエスト編集
- `form` key-value 編集
- `json` フリーテキスト編集
- レスポンス表示
- 認証 ON/OFF と ID/PW 入力
- 新規ログファイル作成

未完:

- 画面の責務分割
- エラー表示の磨き込み
- 実利用時の UX 改善

### 3.10 フェーズ 10: zsh 連携

概ね完了:

- `bin/vurl.zsh`
- `vurl` 関数
- `-l` / `-y`
- frontend/backend/ルート Makefile

未完:

- 実運用向けの細かい挙動調整
- `vurl` 起動時の本番配信フロー確認
- `bin/vurl.zsh` の固定値見直し
- production の固定待受 `127.0.0.1:1357` 確認

## 4. 残タスク

ここから先は、新規機能追加よりも仕上げと整理の比率が高い。

### 4.1 最優先

- `POST /api/send` の成功系を、実際に叩けるモック API 付きで確認する
- frontend から backend への実ブラウザ疎通を手動確認する
- ログの実出力内容を sample で確認する

### 4.2 リファクタリング

- `src/frontend/src/app/App.tsx` を `components/` と `features/` に分割する
- backend の service 層をもう少し責務ごとに薄くする
- API 入出力型と domain 型の境界を整理する
- supervisor と child 起動周りのコードを整理する

### 4.3 テスト強化

- `POST /api/send` の integration test
- 認証リトライの test
- ログ出力の test
- `fixed.default` の test
- `http` 認証の test

### 4.4 UX / UI 改善

- リクエスト選択状態の視覚強化
- エラー表示の見やすさ改善
- body 編集 UI の使い勝手改善
- response header 表示の整形
- mobile 時のレイアウト改善

### 4.5 運用まわり

- `bin/vurl.zsh` の利用前提を README にまとめる
- frontend build と backend 起動手順の整理
- 実際の `$HOME/.vurl` 前提の動作確認
- production の固定待受 `127.0.0.1:1357` の動作確認

## 5. 実装上の注意

- Git は触らない
- YAML 編集機能は UI に入れない
- ログには `mask` がある変数だけマスク値を出す
- `mask` 未指定の値はそのまま出る
- 未定義変数は送信前エラーにする
- `c` は全プロジェクト全 YAML をチェックする
- `r` は backend のみ再起動する

## 6. 次に着手するなら

1. モック API を用意して `POST /api/send` の正常系を確認する
2. frontend を実ブラウザで開いて end-to-end の手動確認をする
3. `App.tsx` を分割して frontend をリファクタリングする
4. 認証とログの integration test を追加する
