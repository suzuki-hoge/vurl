import { describe, expect, test } from "vitest"

import type { SendResponse } from "../../types/api"
import {
  formatResponseBody,
  getResponseContentType,
  highlightJson,
  isImageContentType,
  isJsonContentType,
  statusToneClass
} from "./model"

const baseResponse: SendResponse = {
  status: 200,
  headers: [{ key: "Content-Type", value: "application/json; charset=utf-8" }],
  body: '{"ok":true}',
  retried_auth: false,
  notifications: [],
  current_log_file: "log.txt"
}

describe("response model", () => {
  test("gets content type from explicit property or headers", () => {
    expect(
      getResponseContentType({
        ...baseResponse,
        content_type: "image/png"
      })
    ).toBe("image/png")

    expect(getResponseContentType(baseResponse)).toBe(
      "application/json; charset=utf-8"
    )
  })

  test("formats json body and leaves invalid json untouched", () => {
    expect(formatResponseBody(baseResponse)).toContain("\n")
    expect(
      formatResponseBody({
        ...baseResponse,
        body: "{oops"
      })
    ).toBe("{oops")
  })

  test("detects content types and status tones", () => {
    expect(isJsonContentType("application/problem+json")).toBe(true)
    expect(isImageContentType("image/png; charset=binary")).toBe(true)
    expect(statusToneClass(204)).toBe("is-success")
    expect(statusToneClass(404)).toBe("is-error")
    expect(statusToneClass(undefined)).toBe("")
  })

  test("highlightJson tokenizes keys and literals", () => {
    expect(highlightJson('{"ok":true,"count":1}')).toEqual([
      { type: "punctuation", value: "{" },
      { type: "key", value: '"ok"' },
      { type: "punctuation", value: ":" },
      { type: "boolean", value: "true" },
      { type: "punctuation", value: "," },
      { type: "key", value: '"count"' },
      { type: "punctuation", value: ":" },
      { type: "number", value: "1" },
      { type: "punctuation", value: "}" }
    ])
  })
})
