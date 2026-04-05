# backend

`src/backend` は `vurl-backend` のルートです。

現時点では以下だけ整備しています。

- Cargo プロジェクトの土台
- backend の実装土台
- Actix/Reqwest/Serde まわりの依存定義
- バックエンド層のディレクトリ構成

想定レイヤ:

- `app`: 起動構成
- `cli`: CLI 引数
- `config`: パス解決と設定
- `domain`: YAML スキーマに対応する中核型
- `errors`: アプリ固有エラー
- `handlers`: Actix ハンドラ
- `logging`: tracing とログ出力
- `models`: API 入出力モデル
- `runtime`: 実行時コンテキスト
- `services`: リクエスト送信、認証、ログ
- `state`: Actix の共有状態
