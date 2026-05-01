# Documentation Maintenance

この文書は、実装変更時にどのドキュメントを更新すべきかを判断するための対応表です。タスク完了時は必ずここを確認します。

## 基本ルール

- 文書は現行実装だけを書く
- 変更したコードの外部仕様、操作仕様、設計説明に影響があるなら対応文書を更新する
- 更新不要と判断した場合も、その理由をタスク完了時に明示する

## 更新対応表

- 起動方法、前提環境、開発コマンド、利用手順を変えた
  - 更新: `README.md`
  - 必要なら更新: `AGENTS.md`, `docs/architecture.md`

- AI エージェントに最初に読ませるべき文書や運用ルールを変えた
  - 更新: `AGENTS.md`
  - 必要なら更新: `docs/doc-maintenance.md`

- システム構成、責務分割、実行構成、状態管理を変えた
  - 更新: `docs/architecture.md`
  - 必要なら更新: `README.md`, `AGENTS.md`

- backend API の endpoint、request/response shape、timeout、reload、ログ挙動を変えた
  - 更新: `docs/backend-api.md`
  - 必要なら更新: `docs/architecture.md`, `README.md`

- frontend の画面構成、操作方法、URL 仕様、送信 UX、response 表示を変えた
  - 更新: `docs/frontend-ui.md`
  - 必要なら更新: `README.md`, `docs/architecture.md`

- YAML schema、認証 schema、変数展開仕様、validation rule を変えた
  - 更新: `docs/yaml-spec.md`
  - 必要なら更新: `docs/backend-api.md`, `docs/architecture.md`

- YAML を新規作成するための推奨手順、サンプル、書き方を変えた
  - 更新: `docs/yaml-authoring-guide.md`
  - 必要なら更新: `docs/yaml-spec.md`

- ドキュメント一覧や導線を変えた
  - 更新: `docs/index.md`
  - 必要なら更新: `README.md`, `AGENTS.md`

## タスク完了時チェック

1. 変更したコードに対して、上の対応表で対象文書を洗い出す
2. 対象文書を更新する
3. 最終報告で、更新した文書を列挙する
4. 更新不要の文書がある場合は、その理由を短く書く

## この文書自体を更新すべきとき

- 新しい種類の変更が増えた
- 既存の対応表では更新漏れが起きた
- 文書構成を変更した
