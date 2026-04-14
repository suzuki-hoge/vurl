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
  const response = await fetch(`${BACKEND_URL}${path}`, {
    headers: {
      "Content-Type": "application/json",
      ...(init?.headers ?? {})
    },
    ...init
  })

  if (!response.ok) {
    const text = await response.text()
    throw new Error(text || `HTTP ${response.status}`)
  }

  return response.json() as Promise<T>
}

async function requestSend(payload: SendRequestPayload): Promise<SendResponse> {
  const response = await fetch(`${BACKEND_URL}/api/send`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json"
    },
    body: JSON.stringify(payload)
  })

  const contentType = response.headers.get("content-type") ?? ""
  if (contentType.toLowerCase().includes("application/json")) {
    return (await response.json()) as SendResponse
  }

  const text = await response.text()
  throw new Error(text || `HTTP ${response.status}`)
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
