# frontend

`src/frontend` は React フロントエンドのルートです。

現時点では以下だけ整備しています。

- `nodenv` 24 系前提の TypeScript + Vite 構成
- 3 ペインの最小レイアウト
- 今後の機能追加先になるディレクトリ構成
- スタイルはコンポーネントと対になる `.scss` に寄せる
- backend URL は `VITE_BACKEND_URL` で切り替えられる
- `.env.example` をコピーして frontend 用 `.env` を作れる
- `Makefile` で `make dev`, `make build`, `make fix` を使える

想定レイヤ:

- `app`: アプリ全体のエントリ
- `api`: バックエンド通信
- `components`: 汎用 UI とペイン別 UI
- `features`: 機能単位の状態と表示
- `lib`: 小さなユーティリティ
- `state`: グローバル状態
- `styles`: 共通トークンや mixin 置き場
- `types`: 共有型
