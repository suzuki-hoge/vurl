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
```

`vurl` は以下を行います。

- release build の `vurl-backend` を起動する
- `http://127.0.0.1:1357` をブラウザで開く
- backend が frontend の `dist` を静的配信する

補助コマンド:

```sh
vurl -l
vurl -y
```

- `vurl -l`: `$HOME/.vurl/logs` に移動
- `vurl -y`: `$HOME/.vurl/defs` に移動

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

## Supervisor 操作

`vurl` で起動した backend は supervisor モードで動きます。

- `c`: 全 YAML チェック
- `r`: backend 子プロセス再起動
- `q`: 終了
