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

## リクエスト定義

1 request = 1 YAML とする。

例:

```yaml
name: start weight
method: GET
path: /start_weight/fetch
auth: true
request:
  query:
    - key: user_token
      value: "{{user_token}}"
  headers:
    - key: X-ASKEN-TOKEN
      value: "{{asken_token}}"
  body:
    type: json
    text: ""
```

項目:

- `name`
- `method`
- `path`
- `auth`
- `request.query`
- `request.headers`
- `request.body`

body:

- `type: json` のとき `text`
- `type: form` のとき `form`

## 環境定義

例:

```yaml
name: staging
constants:
  base_url:
    value: https://example.com
variables:
  user_token:
    value: ""
    mask: xxx
```

項目:

- `name`
- `constants.<key>.value`
- `variables.<key>.value`
- `variables.<key>.mask`

ルール:

- `constants` と `variables` は分ける
- `mask` は任意
- 空文字は `""` と明示する

## 認証定義

ファイルは `environments/auth.yaml` とする。

### fixed

例:

```yaml
environments:
  azisai:
    mode: fixed
    credentials:
      presets:
        - name: preset-1
          id: "1"
    mappings:
      items:
        - id: "1"
          variables:
            user_token: "..."
            asken_token: "..."
      default:
        variables:
          user_token: "..."
          asken_token: "..."
```

ルール:

- 認証リクエストは発生しない
- マッピングに応じて環境変数を更新する
- `default` を持てる

### http

例:

```yaml
environments:
  staging:
    mode: http
    credentials:
      presets:
        - name: staging-user
          id: "1"
          password: abc
    request:
      method: POST
      url: "{{base_url}}/auth/login/2"
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
        - from: $.user_token
          to: user_token
        - from: $.asken_token
          to: asken_token
```

ルール:

- 認証 API のレスポンスから環境変数へ inject する
- `from` は JSONPath 風の `$.foo.bar` 形式を使う

## 変数埋め込み

埋め込み構文:

- `{{name}}`
- `{{auth.id}}`
- `{{auth.password}}`

未定義の場合は送信前エラーにする。
