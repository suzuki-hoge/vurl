# vurl

`vurl` は、React frontend と Rust backend で構成した、個人用の API 実行ツールです。

関連ドキュメント:

- [Architecture](./docs/architecture.md)
- [Spec Backend](./docs/spec-backend.md)
- [Spec Frontend](./docs/spec-frontend.md)
- [Spec YAML](./docs/spec-yaml.md)
- [Memo](./docs/memo.md)

## 動作前提

- macOS
- Rust インストール済み
- `nodenv` で Node.js `24.0.0`
- frontend のパッケージマネージャは `yarn`
- 定義ルートは `$HOME/.vurl`

## 初回セットアップ

```sh
make -C src/frontend dev
```

別ターミナルで:

```sh
make -C src/backend build
make -C src/frontend build
source bin/vurl.zsh
```

## 起動方法

通常利用:

```sh
vurl
vurl --no-open
```

`vurl` は以下を行います。

- release build の `vurl-backend` をバックグラウンド起動する
- 起動済みでなければ `http://127.0.0.1:1357` をブラウザで開く
- backend が frontend の `dist` を静的配信する
- YAML の反映は画面上の `Reload YAML` ボタンから行う

`vurl --no-open` は backend を通常どおり起動しますが、ブラウザは開きません。

補助コマンド:

```sh
vurl --down
vurl --log-dir
vurl --log-dir <project>
vurl --edit
```

- `vurl --down`: バックグラウンドの `vurl-backend` を停止
- `vurl --log-dir`: `$HOME/.vurl/logs` に移動
- `vurl --log-dir <project>`: `$HOME/.vurl/logs/<project>` に移動
- `vurl --edit`: リポジトリルートを `IntelliJ IDEA 2.app` で開く

## 開発用コマンド

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

リポジトリ全体:

```sh
make build-all
make fix-all
```

## YAML 再読込

YAML を編集したあとは、画面左上の `Reload YAML` ボタンで backend に再読込を要求します。

- reload 成功時はブラウザ全体を再読み込み
- YAML の書式エラーなどで reload 失敗時は画面にエラー表示
