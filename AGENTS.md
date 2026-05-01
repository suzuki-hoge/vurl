# vurl AGENTS.md

- 日本語で対応すること。
- 最初に読む基本ファイルは `README.md` と `docs/doc-maintenance.md` とする。
- 追加で読む文書は、変更対象に応じて必要最小限に絞ること。

変更対象ごとの追加読込:

- YAML 定義仕様、認証仕様、変数展開: `docs/yaml-spec.md`
- YAML を新規作成するための手順や例: `docs/yaml-authoring-guide.md`
- backend API やログ、reload、送信挙動: `docs/backend-api.md`
- frontend UI や画面操作、URL 状態: `docs/frontend-ui.md`
- 全体設計や責務分割: `docs/architecture.md`

ドキュメント更新ルール:

- コードや仕様を変更したら、`docs/doc-maintenance.md` の対応表に従って更新対象文書を確認すること。
- タスク完了時は「どの文書を更新したか」または「なぜ更新不要か」を明示すること。
- 文書には後方互換のための過去事情や廃止済み仕様を残さず、現行実装だけを書くこと。
