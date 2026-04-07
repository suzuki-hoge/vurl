import { describe, expect, test } from "vitest"

import type { RequestDefinition } from "../../types/api"
import {
  buildEffectiveHeaders,
  definitionToDraft,
  emptyDraft,
  persistHeaders,
  sanitizeHeaderPairs,
  sanitizePairs,
  shouldInjectJsonContentType
} from "./model"

describe("request model", () => {
  test("definitionToDraft converts editable headers", () => {
    const definition: RequestDefinition = {
      name: "Get User",
      method: "POST",
      path: "/users",
      auth: true,
      request: {
        query: [{ key: "page", value: "1" }],
        headers: [{ key: "X-Test", value: "ok" }],
        body: { type: "json", text: '{"ok":true}' }
      }
    }

    expect(
      definitionToDraft("project-1", "local", "users/get.yaml", definition)
    ).toMatchObject({
      project: "project-1",
      environment: "local",
      definitionPath: "users/get.yaml",
      headers: [{ key: "X-Test", value: "ok", state: "editable" }]
    })
  })

  test("buildEffectiveHeaders injects json content type once", () => {
    const draft = {
      ...emptyDraft,
      body: { type: "json" as const, text: "{}" },
      headers: [{ key: "X-Test", value: "1", state: "editable" as const }]
    }

    expect(buildEffectiveHeaders(draft)).toEqual([
      { key: "Content-Type", value: "application/json", state: "locked" },
      { key: "X-Test", value: "1", state: "editable" }
    ])
    expect(shouldInjectJsonContentType(draft)).toBe(true)
  })

  test("buildEffectiveHeaders does not inject when content-type already exists", () => {
    const draft = {
      ...emptyDraft,
      headers: [
        {
          key: " content-type ",
          value: "application/ld+json",
          state: "editable" as const
        }
      ]
    }

    expect(shouldInjectJsonContentType(draft)).toBe(false)
    expect(buildEffectiveHeaders(draft)).toBe(draft.headers)
  })

  test("persistHeaders removes locked injected header only", () => {
    const draft = {
      ...emptyDraft,
      body: { type: "json" as const, text: "{}" },
      headers: [{ key: "X-Test", value: "1", state: "editable" as const }]
    }

    const persisted = persistHeaders(draft, [
      { key: "Content-Type", value: "application/json", state: "locked" },
      { key: "X-Test", value: "1", state: "editable" }
    ])

    expect(persisted).toEqual([
      { key: "X-Test", value: "1", state: "editable" }
    ])
  })

  test("sanitize helpers drop blank keys and disabled headers", () => {
    expect(
      sanitizePairs([
        { key: "", value: "skip" },
        { key: "q", value: "ok" }
      ])
    ).toEqual([{ key: "q", value: "ok" }])

    expect(
      sanitizeHeaderPairs([
        { key: "", value: "skip", state: "editable" },
        { key: "X-On", value: "1", state: "editable" },
        { key: "X-Off", value: "2", state: "off" }
      ])
    ).toEqual([{ key: "X-On", value: "1" }])
  })
})
