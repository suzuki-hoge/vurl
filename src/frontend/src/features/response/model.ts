import type { SendResponse } from "../../types/api"

export type JsonToken = {
  type:
    | "key"
    | "string"
    | "number"
    | "boolean"
    | "null"
    | "punctuation"
    | "plain"
  value: string
}

export function getResponseContentType(response: SendResponse | null): string {
  if (!response) {
    return ""
  }

  if (response.content_type) {
    return response.content_type
  }

  return (
    response.headers.find(
      (header) => header.key.toLowerCase() === "content-type"
    )?.value ?? ""
  )
}

export function formatResponseBody(response: SendResponse | null): string {
  if (!response) {
    return ""
  }

  const contentType = getResponseContentType(response)
  if (!isJsonContentType(contentType)) {
    return response.body
  }

  try {
    return JSON.stringify(JSON.parse(response.body), null, 2)
  } catch {
    return response.body
  }
}

export function isJsonContentType(contentType: string): boolean {
  const mimeType = contentType.split(";")[0]?.trim().toLowerCase() ?? ""
  return mimeType === "application/json" || mimeType.endsWith("+json")
}

export function isImageContentType(contentType: string): boolean {
  const mimeType = contentType.split(";")[0]?.trim().toLowerCase() ?? ""
  return mimeType.startsWith("image/")
}

export function statusToneClass(status?: number): string {
  if (!status) {
    return ""
  }
  if (status >= 200 && status < 400) {
    return "is-success"
  }
  if (status >= 400 && status < 600) {
    return "is-error"
  }
  return ""
}

export function highlightJson(text: string): JsonToken[] {
  const tokens: JsonToken[] = []
  const pattern =
    /("(?:\\.|[^"\\])*")(\s*:)?|\b-?(?:0|[1-9]\d*)(?:\.\d+)?(?:[eE][+-]?\d+)?\b|\btrue\b|\bfalse\b|\bnull\b|[{}[\],:]/g

  let lastIndex = 0
  for (const match of text.matchAll(pattern)) {
    const index = match.index ?? 0
    if (index > lastIndex) {
      tokens.push({ type: "plain", value: text.slice(lastIndex, index) })
    }

    const value = match[0]
    if (match[1]) {
      tokens.push({
        type: match[2] ? "key" : "string",
        value: match[1]
      })
      if (match[2]) {
        tokens.push({ type: "punctuation", value: match[2] })
      }
    } else if (value === "true" || value === "false") {
      tokens.push({ type: "boolean", value })
    } else if (value === "null") {
      tokens.push({ type: "null", value })
    } else if (/^[{}[\],:]$/.test(value)) {
      tokens.push({ type: "punctuation", value })
    } else {
      tokens.push({ type: "number", value })
    }

    lastIndex = index + value.length
  }

  if (lastIndex < text.length) {
    tokens.push({ type: "plain", value: text.slice(lastIndex) })
  }

  return tokens
}
