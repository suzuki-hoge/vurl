# YAML Spec

## 配置

```text
$HOME/.vurl/
  defs/
    <project>/
      requests/
        **/*.yaml
      environments/
        <environment>.yaml
        auth.yaml
```

1 project は `defs/<project>/` 配下に配置します。

## request 定義

1 request = 1 YAML です。

例:

```yaml
name: Get User
method: GET
path: /users/{{user_id}}
auth: true
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

必須項目:

- `name: string`
- `method: string`
- `path: string`
- `auth: bool`
- `request.query: KeyValueEntry[]`
- `request.headers: KeyValueEntry[]`
- `request.body`

`query` と `headers` は省略時に空配列として扱われます。

### request.body

`json`:

```yaml
body:
  type: json
  text: "{\"name\":\"alice\"}"
```

`form`:

```yaml
body:
  type: form
  form:
    - key: mode
      items:
        - value: "0"
          description: normal
          default: true
        - value: "1"
          description: dry-run
    - key: memo
      value: hello
```

`form` field のルール:

- `items` が空なら `value` は必須
- `items` があるなら `value` は書けない
- `items` があるなら `default: true` はちょうど 1 つ必須
- `enabled` は省略時 `true`

`form` field schema:

- `key: string`
- `value?: string`
- `enabled?: bool`
- `items:`
  - `value: string`
  - `description: string`
  - `default?: bool`

## environment 定義

例:

```yaml
name: local
order: 1
constants:
  base_url:
    value: http://localhost:18080
  tenant_id:
    value: tenant-a
variables:
  auth_token:
    value: ""
    mask: xxx
  user_id:
    value: "42"
```

項目:

- `name: string`
- `order?: u32`
- `constants.<key>.value: string`
- `constants.<key>.mask?: string`
- `variables.<key>.value: string`
- `variables.<key>.mask?: string`

意味:

- `constants`: 実行中に backend が更新しない値
- `variables`: backend 実行時に更新されうる値
- `mask`: ログに出すときの置換文字列

`base_url` は request 送信時に必須です。

## auth 定義

file 名は `environments/auth.yaml` 固定です。

top-level:

```yaml
environments:
  local:
    ...
```

各 environment 名に対して認証設定を 1 つ持ちます。

### fixed

例:

```yaml
environments:
  local:
    mode: fixed
    credentials:
      presets:
        - name: preset-user
          id: known-user
    mappings:
      items:
        - id: known-user
          variables:
            auth_token: fixed-token
      default:
        variables:
          auth_token: default-token
```

項目:

- `credentials.presets[]`
  - `name: string`
  - `id: string`
  - `password?: string`
- `mappings.items[]`
  - `id: string`
  - `variables: map<string, string>`
- `mappings.default.variables: map<string, string>`

意味:

- 認証 API は呼ばない
- 入力された `id` に一致する mapping を探し、対応 variable を更新する
- 一致しなければ `default` を使う
- `default` もなければ認証エラー

### http

例:

```yaml
environments:
  staging:
    mode: http
    credentials:
      presets:
        - name: qa-user
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
          - key: tenant
            value: "{{tenant_id}}"
    response:
      inject:
        - from: $.token
          to: auth_token
```

項目:

- `credentials?:`
  - `presets[]`
- `request.method: string`
- `request.url: string`
- `request.headers: KeyValueEntry[]`
- `request.body`
- `response.inject[]`
  - `from: string`
  - `to: string`

意味:

- `request` を認証 API 向け request として送る
- `response.inject` に従って response JSON から値を抜き出し、environment variable を更新する

`from` の制約:

- `$.foo.bar` のような単純な object path だけ対応
- `$.` で始まらない path は非対応

## 変数展開

使用できる構文:

- `{{name}}`
- `{{auth.id}}`
- `{{auth.password}}`

解決対象:

- request の `path`
- request の `query[].value`
- request の `headers[].value`
- request body
- http auth の `request.url`
- http auth の `request.headers[].value`
- http auth body

参照順:

- `auth.id`, `auth.password`
- environment variables
- environment constants

未定義値がある場合は送信前エラーです。

## environment の実行時更新

- backend は environment ごとに runtime state を持つ
- `fixed` / `http` 認証で variable が更新される
- reload 実行時は YAML を再読込し、runtime state も作り直される
