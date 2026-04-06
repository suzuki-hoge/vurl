import type {
  DefinitionResponse,
  EnvironmentSummary,
  ProjectSummary,
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
  send: (payload: SendRequestPayload) =>
    request<SendResponse>("/api/send", {
      method: "POST",
      body: JSON.stringify(payload)
    })
}
