# sample/project-1

サンプルのルートディレクトリは `sample/` です。

- 起動例: `vurl-backend --root /.../sample`
- プロジェクト名: `project-1`
- 定義ルート: `sample/defs/project-1`
- ログ出力先想定: `sample/logs/project-1`
- 環境: `local`, `stg`
- 認証: `local` は `fixed`, `stg` は `http`

## request YAML

最小構成は次です。

- `name`: 一覧表示名
- `method`: HTTP メソッド
- `path`: 環境の `base_url` を除いたパス
- `auth`: 認証 ON/OFF のデフォルト値
- `request.query`: query 初期値
- `request.headers`: header 初期値
- `request.body`: `json` または `form`

`auth` を `bool` にしたのは、今の要件だと「このリクエストは認証対象か」のフラグだけで足りるためです。

## environment YAML

今のサンプルは、環境ごとの差分を `constants` と `variables` に分けています。

- `constants`: UI や認証処理で更新しない固定値
- `variables`: 実行中に更新されうる値
- `variables.*.mask`: ログに出す置換値
- `mask` は任意項目で、未指定ならマスクしない

この形の理由:

- `auth_token` のような更新値と `base_url` のような固定値を分けられる
- ログマスクを変数ごとに持てる
- 変数展開時に「更新対象かどうか」を判定しやすい

確定事項:

1. `constants` と `variables` は分ける
2. `variables.{key}.mask` は任意項目とする

## auth.yaml

今のサンプルは「環境ごとに認証方式が違う」をそのまま表現するため、`environments.<env_name>` の下に設定を置いています。

- `local.mode: fixed`
- `stg.mode: http`

`fixed` モードの意味:

- React で入力した ID/PW を受け取る
- 設定済みの対応表を見て、結果として保存する変数を決める
- ローカル検証用の簡易認証を想定している
- `default` を持てるので、実質的な固定値返却もできる

`http` モードの意味:

- 認証 API に実際に送信する
- レスポンスから必要な値を取り出して環境変数へ保存する
- 保存先は `response.inject` で指定する

この形の理由:

- 環境ごとに認証方式を変えられる
- `fixed` と `http` を同じ `auth.yaml` に共存できる
- バックエンドは「現在環境の設定だけ選ぶ」実装にしやすい

確定事項:

1. `environments.local` のようなマップ形式にする
2. `fixed` はマッピング形式とし、`default` を持てる
3. `response.save` と `response.inject` は統合し、名前は `response.inject` にする
