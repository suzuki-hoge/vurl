# 開発メモ

## Node

- `nodenv` で `24.0.0` を使う
- frontend は `src/frontend` で `yarn install`

## Backend

- backend ルートは `src/backend`
- 開発時ビルド確認: `cargo check --manifest-path src/backend/Cargo.toml`
- 開発時起動: `cargo run --manifest-path src/backend/Cargo.toml`
- backend の待受は `127.0.0.1:1357` 固定

## Supervisor モード

`vurl-backend` は親プロセスとして起動し、同じバイナリを子プロセスとして再起動管理する。

- `c`: 全 YAML チェック
- `r`: backend 子プロセス再起動
- `q`: 終了

## zsh

`bin/vurl.zsh` を `source` すると `vurl` 関数が使える。

- `vurl`: backend 起動 + frontend URL を open
- `vurl -l`: `logs` へ `cd`
- `vurl -y`: `defs` へ `cd`

固定値:

- root: `$HOME/.vurl`
- backend: `bin/vurl.zsh` から見た相対パスの `../src/backend/target/release/vurl-backend`
- open する URL: `http://127.0.0.1:1357`

## Make

- ルート: `make build-all`, `make fix-all`
- backend: `make -C src/backend dev|build|fix`
- frontend: `make -C src/frontend dev|build|fix`

正式運用時は frontend の production build を backend が静的配信する。
そのため production では backend が `1357` で UI と API の両方を配信する。
