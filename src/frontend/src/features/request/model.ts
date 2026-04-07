import type { RequestDefinition, RequestKeyValue } from "../../types/api"

export type DraftHeaderState = "editable" | "locked" | "off"

export type DraftHeader = {
  key: string
  value: string
  state: DraftHeaderState
}

export type DraftState = {
  project: string
  environment: string
  definitionPath: string
  name: string
  method: string
  urlPath: string
  auth: boolean
  query: RequestKeyValue[]
  headers: DraftHeader[]
  body:
    | {
        type: "json"
        text: string
      }
    | {
        type: "form"
        form: RequestKeyValue[]
      }
}

export const emptyDraft: DraftState = {
  project: "",
  environment: "",
  definitionPath: "",
  name: "",
  method: "GET",
  urlPath: "",
  auth: false,
  query: [],
  headers: [],
  body: { type: "json", text: "" }
}

export function definitionToDraft(
  project: string,
  environment: string,
  path: string,
  definition: RequestDefinition
): DraftState {
  return {
    project,
    environment,
    definitionPath: path,
    name: definition.name,
    method: definition.method,
    urlPath: definition.path,
    auth: definition.auth,
    query: definition.request.query,
    headers: definition.request.headers.map((header) => ({
      ...header,
      state: "editable"
    })),
    body:
      definition.request.body.type === "json"
        ? { type: "json", text: definition.request.body.text }
        : { type: "form", form: definition.request.body.form }
  }
}

export function sanitizePairs(items: RequestKeyValue[]): RequestKeyValue[] {
  return items.filter((item) => item.key.trim() !== "")
}

export function sanitizeHeaderPairs(items: DraftHeader[]): RequestKeyValue[] {
  return items
    .filter((item) => item.state !== "off")
    .filter((item) => item.key.trim() !== "")
    .map((item) => ({ key: item.key, value: item.value }))
}

export function buildEffectiveHeaders(draft: DraftState): DraftHeader[] {
  if (!shouldInjectJsonContentType(draft)) {
    return draft.headers
  }

  return [
    { key: "Content-Type", value: "application/json", state: "locked" },
    ...draft.headers
  ]
}

export function persistHeaders(
  draft: DraftState,
  items: DraftHeader[]
): DraftHeader[] {
  if (!shouldInjectJsonContentType(draft)) {
    return items
  }

  return items.filter(
    (item) =>
      !(
        item.state === "locked" &&
        item.key.trim().toLowerCase() === "content-type" &&
        item.value === "application/json"
      )
  )
}

export function shouldInjectJsonContentType(draft: DraftState): boolean {
  return draft.body.type === "json" && !hasHeader(draft.headers, "content-type")
}

export function hasHeader(items: DraftHeader[], key: string): boolean {
  const normalizedKey = key.trim().toLowerCase()
  return items.some((item) => item.key.trim().toLowerCase() === normalizedKey)
}
