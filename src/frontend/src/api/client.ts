import type {
  DefinitionResponse,
  EnvironmentSummary,
  ProjectSummary,
  ReloadResponse,
  RuntimeInfo,
  SendRequestPayload,
  SendResponse,
  TreeResponse
} from "../types/api"

const BACKEND_URL =
  (import.meta.env.VITE_BACKEND_URL as string | undefined) ??
  (typeof window !== "undefined"
    ? window.location.origin
    : "http://127.0.0.1:1357")

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const response = await fetchWithError(`${BACKEND_URL}${path}`, {
    headers: {
      "Content-Type": "application/json",
      ...(init?.headers ?? {})
    },
    ...init
  })

  if (!response.ok) {
    throw await buildResponseError(response)
  }

  return response.json() as Promise<T>
}

async function requestSend(payload: SendRequestPayload): Promise<SendResponse> {
  const response = await fetchWithError(`${BACKEND_URL}/api/send`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json"
    },
    body: JSON.stringify(payload)
  })

  const contentType = response.headers.get("content-type") ?? ""
  if (!response.ok) {
    throw await buildResponseError(response)
  }

  if (contentType.toLowerCase().includes("application/json")) {
    return (await response.json()) as SendResponse
  }

  const text = await response.text()
  throw new Error(text || `HTTP ${response.status}`)
}

async function fetchWithError(
  input: RequestInfo | URL,
  init?: RequestInit
): Promise<Response> {
  try {
    return await fetch(input, init)
  } catch (cause) {
    const detail = cause instanceof Error ? cause.message : String(cause)
    throw new Error(
      `Request failed: ${typeof input === "string" ? input : input.toString()}\n${detail}`
    )
  }
}

async function buildResponseError(response: Response): Promise<Error> {
  const text = await response.text()
  const message = parseErrorMessage(text)
  return new Error(
    [`HTTP ${response.status} ${response.statusText}`.trim(), message]
      .filter(Boolean)
      .join("\n\n")
  )
}

function parseErrorMessage(text: string): string {
  if (!text.trim()) {
    return ""
  }

  try {
    const parsed = JSON.parse(text) as { message?: unknown }
    if (typeof parsed.message === "string") {
      return parsed.message
    }
  } catch {
    return text
  }

  return text
}

export const apiClient = {
  runtime: () => request<RuntimeInfo>("/api/runtime"),
  projects: () => request<ProjectSummary[]>("/api/projects"),
  environments: (project: string) =>
    request<EnvironmentSummary[]>(
      `/api/environments?project=${encodeURIComponent(project)}`
    ),
  tree: (project: string) =>
    request<TreeResponse>(`/api/tree?project=${encodeURIComponent(project)}`),
  definition: (project: string, path: string) =>
    request<DefinitionResponse>(
      `/api/definition?project=${encodeURIComponent(project)}&path=${encodeURIComponent(path)}`
    ),
  reload: () =>
    request<ReloadResponse>("/api/reload", {
      method: "POST"
    }),
  send: (payload: SendRequestPayload) => requestSend(payload)
}
