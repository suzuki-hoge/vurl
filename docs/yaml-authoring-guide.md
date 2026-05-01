# YAML Authoring Guide

この文書は、新しい project の YAML 定義一式をゼロから作るための実践ガイドです。正式な項目定義は [yaml-spec.md](./yaml-spec.md) を参照してください。

## 最小構成

新しい project を `my-project` とすると、最低限必要なのは次の 3 つです。

```text
$HOME/.vurl/defs/my-project/
  requests/
    ping.yaml
  environments/
    local.yaml
    auth.yaml
```

## 手順 1: environment を作る

`$HOME/.vurl/defs/my-project/environments/local.yaml`

```yaml
name: local
order: 1
constants:
  base_url:
    value: http://localhost:8080
variables: {}
```

最初は `base_url` だけあれば十分です。認証や request で参照する値があれば `variables` や `constants` を追加します。

## 手順 2: auth.yaml を作る

認証不要でも `auth.yaml` は必須です。最小の `fixed` を置くのが簡単です。

```yaml
environments:
  local:
    mode: fixed
    credentials:
      presets: []
    mappings:
      items: []
      default:
        variables: {}
```

request 側で `auth: false` ならこの設定は実質使われませんが、backend の起動には必要です。

## 手順 3: request を 1 つ作る

`$HOME/.vurl/defs/my-project/requests/ping.yaml`

```yaml
name: Ping
method: GET
path: /ping
auth: false
request:
  query: []
  headers: []
  body:
    type: json
    text: ""
```

これで `Reload YAML` 後に request が見える状態になります。

## query / headers を使う

```yaml
request:
  query:
    - key: verbose
      value: "true"
  headers:
    - key: Accept
      value: application/json
  body:
    type: json
    text: ""
```

固定値だけでなく、`{{name}}` 形式で environment 値も参照できます。

## JSON body を使う

```yaml
body:
  type: json
  text: |
    {
      "user_id": "{{user_id}}",
      "dry_run": true
    }
```

注意:

- `text` は文字列として扱われる
- JSON として妥当かどうかは backend では検証しない
- frontend は JSON body に対して `Content-Type: application/json` を自動補完する

## form body を使う

```yaml
body:
  type: form
  form:
    - key: name
      value: alice
    - key: mode
      items:
        - value: normal
          description: normal
          default: true
        - value: dry-run
          description: dry-run
```

使い分け:

- 単純な入力欄にしたい項目は `value`
- 選択肢にしたい項目は `items`

## 固定認証を使う

ローカル検証用に token を切り替えたいだけなら `fixed` が最も簡単です。

environment:

```yaml
variables:
  auth_token:
    value: ""
    mask: xxx
```

auth:

```yaml
environments:
  local:
    mode: fixed
    credentials:
      presets:
        - name: alice
          id: alice
    mappings:
      items:
        - id: alice
          variables:
            auth_token: token-for-alice
      default:
        variables:
          auth_token: fallback-token
```

request:

```yaml
auth: true
request:
  headers:
    - key: Authorization
      value: Bearer {{auth_token}}
```

## HTTP 認証を使う

ログイン API から token を取得する場合は `http` を使います。

environment:

```yaml
constants:
  base_url:
    value: https://api.example.test
variables:
  auth_token:
    value: ""
    mask: xxx
```

auth:

```yaml
environments:
  stg:
    mode: http
    credentials:
      presets:
        - name: qa
          id: qa-user
          password: qa-password
    request:
      method: POST
      url: "{{base_url}}/auth/login"
      headers:
        - key: Content-Type
          value: application/x-www-form-urlencoded
      body:
        type: form
        form:
          - key: id
            value: "{{auth.id}}"
          - key: password
            value: "{{auth.password}}"
    response:
      inject:
        - from: $.token
          to: auth_token
```

request:

```yaml
auth: true
request:
  headers:
    - key: Authorization
      value: Bearer {{auth_token}}
```

## よくあるミス

- `auth.yaml` を置いていない
- `base_url` を定義していない
- `form` field で `items` と `value` を同時に書いている
- `items` で `default: true` が 0 個または 2 個以上ある
- `{{name}}` が environment に存在しない
- `response.inject.from` を `$.` 以外で書いている

## まず確認すべき動作

1. `vurl` を起動する
2. project が表示されることを確認する
3. request を開いて `Send` する
4. 必要なら `Reload YAML` で再読込する
5. `$HOME/.vurl/logs/<project>/` にログが出ることを確認する
