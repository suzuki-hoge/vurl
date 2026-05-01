# vurl

`vurl` は、React frontend と Rust backend で構成したローカル専用の API 実行ツールです。YAML で API 定義と環境定義を管理し、認証つきリクエストの実行、レスポンス確認、Markdown ログ出力を行います。

## 動作前提

- macOS
- Rust インストール済み
- `nodenv` で Node.js `24.0.0`
- frontend のパッケージマネージャは `yarn`
- YAML とログのルートは `$HOME/.vurl`

## セットアップ

frontend 開発サーバ:

```sh
make -C src/frontend dev
```

backend / frontend build:

```sh
make -C src/backend build
make -C src/frontend build
source bin/vurl.zsh
```

## 使い方

通常起動:

```sh
vurl
vurl --no-open
```

`vurl` は release build の `vurl-backend` を起動し、`http://127.0.0.1:1357` を入口に frontend を配信します。`vurl --no-open` はブラウザを開かずに backend だけ起動します。

補助コマンド:

```sh
vurl --down
vurl --log-dir
vurl --log-dir <project>
vurl --edit
```

- `vurl --down`: 起動中の backend を停止
- `vurl --log-dir`: `$HOME/.vurl/logs` に移動
- `vurl --log-dir <project>`: `$HOME/.vurl/logs/<project>` に移動
- `vurl --edit`: リポジトリルートを `IntelliJ IDEA 2.app` で開く

## 開発コマンド

frontend:

```sh
make -C src/frontend dev
make -C src/frontend build
make -C src/frontend fix
```

backend:

```sh
make -C src/backend dev
make -C src/backend build
make -C src/backend fix
```

全体:

```sh
make build-all
make fix-all
```

## YAML 再読込

YAML を編集したあとは、画面左上の `Reload YAML` ボタンで backend に再読込を要求します。成功時はブラウザ全体を再読み込みし、失敗時はエラーを表示します。

## ドキュメント

- [docs/index.md](./docs/index.md): 文書全体の入口
- [docs/architecture.md](./docs/architecture.md): システム構成と責務
- [docs/backend-api.md](./docs/backend-api.md): backend API と実行挙動
- [docs/frontend-ui.md](./docs/frontend-ui.md): frontend UI と操作仕様
- [docs/yaml-spec.md](./docs/yaml-spec.md): YAML の正式仕様
- [docs/yaml-authoring-guide.md](./docs/yaml-authoring-guide.md): YAML をゼロから書くための実践ガイド
- [docs/doc-maintenance.md](./docs/doc-maintenance.md): 実装変更時に更新すべき文書の対応表
