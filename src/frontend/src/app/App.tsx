import { useEffect, useMemo, useState } from "react"

import { apiClient } from "../api/client"
import { filterNodes } from "../lib/tree"
import type {
  DefinitionResponse,
  RequestDefinition,
  RequestKeyValue,
  RequestTreeNode,
  SendResponse
} from "../types/api"
import "./App.scss"

type DraftState = {
  project: string
  environment: string
  definitionPath: string
  name: string
  method: string
  urlPath: string
  auth: boolean
  query: RequestKeyValue[]
  headers: RequestKeyValue[]
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

const emptyDraft: DraftState = {
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

export function App() {
  const [projects, setProjects] = useState<string[]>([])
  const [project, setProject] = useState("")
  const [environments, setEnvironments] = useState<string[]>([])
  const [environment, setEnvironment] = useState("")
  const [tree, setTree] = useState<RequestTreeNode[]>([])
  const [filterText, setFilterText] = useState("")
  const [draft, setDraft] = useState<DraftState>(emptyDraft)
  const [response, setResponse] = useState<SendResponse | null>(null)
  const [logFile, setLogFile] = useState("")
  const [authId, setAuthId] = useState("")
  const [authPassword, setAuthPassword] = useState("")
  const [error, setError] = useState("")
  const [loading, setLoading] = useState(true)
  const [sending, setSending] = useState(false)

  useEffect(() => {
    void bootstrap()
  }, [])

  useEffect(() => {
    if (!project) {
      return
    }
    void loadProjectData(project)
  }, [project])

  async function bootstrap() {
    try {
      setLoading(true)
      const runtime = await apiClient.runtime()
      const nextProjects = runtime.projects.map((item) => item.name)
      setProjects(nextProjects)
      if (nextProjects.length > 0) {
        setProject(nextProjects[0])
      }
    } catch (cause) {
      setError(toErrorMessage(cause))
    } finally {
      setLoading(false)
    }
  }

  async function loadProjectData(nextProject: string) {
    try {
      setError("")
      const [envs, treeResponse] = await Promise.all([
        apiClient.environments(nextProject),
        apiClient.tree(nextProject)
      ])
      const nextEnvironments = envs.map((item) => item.name)
      setEnvironments(nextEnvironments)
      setEnvironment((current) =>
        current && nextEnvironments.includes(current)
          ? current
          : (nextEnvironments[0] ?? "")
      )
      setTree(treeResponse.nodes)
      setDraft((current) => ({ ...current, project: nextProject }))
    } catch (cause) {
      setError(toErrorMessage(cause))
    }
  }

  async function openDefinition(path: string) {
    if (!project) {
      return
    }

    try {
      setError("")
      const payload: DefinitionResponse = await apiClient.definition(
        project,
        path
      )
      setDraft(
        definitionToDraft(
          project,
          environment,
          payload.path,
          payload.definition
        )
      )
      setResponse(null)
    } catch (cause) {
      setError(toErrorMessage(cause))
    }
  }

  async function sendRequest() {
    if (!draft.definitionPath || !project || !environment) {
      return
    }

    try {
      setSending(true)
      setError("")
      const result = await apiClient.send({
        project,
        environment,
        path: draft.definitionPath,
        method: draft.method,
        url_path: draft.urlPath,
        query: sanitizePairs(draft.query),
        headers: sanitizePairs(draft.headers),
        body:
          draft.body.type === "json"
            ? { type: "json", text: draft.body.text }
            : { type: "form", form: sanitizePairs(draft.body.form) },
        auth_enabled: draft.auth,
        auth_credentials: {
          id: authId,
          password: authPassword
        }
      })
      setResponse(result)
      setLogFile(result.current_log_file)
    } catch (cause) {
      setError(toErrorMessage(cause))
    } finally {
      setSending(false)
    }
  }

  async function createLogFile() {
    if (!project) {
      return
    }

    try {
      const result = await apiClient.createLogFile(project)
      setLogFile(result.current_log_file)
    } catch (cause) {
      setError(toErrorMessage(cause))
    }
  }

  const filteredTree = useMemo(
    () => filterNodes(tree, filterText.trim().toLowerCase()),
    [tree, filterText]
  )

  return (
    <div className="app-shell">
      <aside className="app-sidebar">
        <div className="panel-title">vurl</div>
        <label className="field">
          <span>Project</span>
          <select
            value={project}
            onChange={(event) => setProject(event.target.value)}
          >
            {projects.map((item) => (
              <option key={item} value={item}>
                {item}
              </option>
            ))}
          </select>
        </label>

        <label className="field">
          <span>Environment</span>
          <select
            value={environment}
            onChange={(event) => {
              const nextEnvironment = event.target.value
              setEnvironment(nextEnvironment)
              setDraft((current) => ({
                ...current,
                environment: nextEnvironment
              }))
            }}
          >
            {environments.map((item) => (
              <option key={item} value={item}>
                {item}
              </option>
            ))}
          </select>
        </label>

        <label className="field">
          <span>Filter</span>
          <input
            value={filterText}
            onChange={(event) => setFilterText(event.target.value)}
            placeholder="path or name"
          />
        </label>

        <button
          className="secondary-button"
          onClick={() => void createLogFile()}
        >
          New Log File
        </button>

        <div className="log-path">{logFile || "log file not created yet"}</div>

        <div className="tree-list">
          {loading ? <div className="muted">loading...</div> : null}
          {filteredTree.map((node) => (
            <TreeNodeView
              key={`${node.type}:${node.path}`}
              node={node}
              onOpen={openDefinition}
            />
          ))}
        </div>
      </aside>

      <main className="app-main">
        <section className="app-request">
          <div className="panel-title">Request</div>
          {error ? <div className="error-banner">{error}</div> : null}

          <div className="request-grid">
            <label className="field">
              <span>Name</span>
              <input value={draft.name} readOnly />
            </label>
            <label className="field">
              <span>Method</span>
              <select
                value={draft.method}
                onChange={(event) =>
                  setDraft((current) => ({
                    ...current,
                    method: event.target.value
                  }))
                }
              >
                {["GET", "POST", "PUT", "PATCH", "DELETE"].map((method) => (
                  <option key={method} value={method}>
                    {method}
                  </option>
                ))}
              </select>
            </label>
          </div>

          <label className="field">
            <span>Path</span>
            <input
              value={draft.urlPath}
              onChange={(event) =>
                setDraft((current) => ({
                  ...current,
                  urlPath: event.target.value
                }))
              }
            />
          </label>

          <div className="inline-fields">
            <label className="toggle">
              <input
                type="checkbox"
                checked={draft.auth}
                onChange={(event) =>
                  setDraft((current) => ({
                    ...current,
                    auth: event.target.checked
                  }))
                }
              />
              <span>Authentication</span>
            </label>
            <label className="field compact">
              <span>ID</span>
              <input
                value={authId}
                onChange={(event) => setAuthId(event.target.value)}
              />
            </label>
            <label className="field compact">
              <span>Password</span>
              <input
                type="password"
                value={authPassword}
                onChange={(event) => setAuthPassword(event.target.value)}
              />
            </label>
          </div>

          <KeyValueEditor
            title="Query"
            items={draft.query}
            onChange={(items) =>
              setDraft((current) => ({ ...current, query: items }))
            }
          />

          <KeyValueEditor
            title="Headers"
            items={draft.headers}
            onChange={(items) =>
              setDraft((current) => ({ ...current, headers: items }))
            }
          />

          <div className="card">
            <div className="card-header">
              <strong>Body</strong>
              <select
                value={draft.body.type}
                onChange={(event) => {
                  const nextType = event.target.value as "json" | "form"
                  setDraft((current) => ({
                    ...current,
                    body:
                      nextType === "json"
                        ? { type: "json", text: "" }
                        : { type: "form", form: [] }
                  }))
                }}
              >
                <option value="json">json</option>
                <option value="form">form</option>
              </select>
            </div>

            {draft.body.type === "json" ? (
              <textarea
                className="body-textarea"
                value={draft.body.text}
                onChange={(event) =>
                  setDraft((current) => ({
                    ...current,
                    body: { type: "json", text: event.target.value }
                  }))
                }
              />
            ) : (
              <KeyValueEditor
                title="Form Body"
                items={draft.body.form}
                onChange={(items) =>
                  setDraft((current) => ({
                    ...current,
                    body: { type: "form", form: items }
                  }))
                }
              />
            )}
          </div>

          <button
            className="primary-button"
            disabled={sending}
            onClick={() => void sendRequest()}
          >
            {sending ? "Sending..." : "Send"}
          </button>
        </section>

        <section className="app-response">
          <div className="panel-title">Response</div>
          <div className="response-meta">
            <span>Status: {response?.status ?? "-"}</span>
            <span>Retried Auth: {response?.retried_auth ? "yes" : "no"}</span>
          </div>

          <div className="card">
            <div className="card-header">
              <strong>Headers</strong>
            </div>
            <pre className="response-block">
              {response?.headers
                .map((header) => `${header.key}: ${header.value}`)
                .join("\n") ?? ""}
            </pre>
          </div>

          <div className="card response-fill">
            <div className="card-header">
              <strong>Body</strong>
            </div>
            <pre className="response-block response-fill">
              {response?.body ?? ""}
            </pre>
          </div>
        </section>
      </main>
    </div>
  )
}

function TreeNodeView(props: {
  node: RequestTreeNode
  onOpen: (path: string) => void | Promise<void>
}) {
  const { node, onOpen } = props

  if (node.type === "directory") {
    return (
      <div className="tree-node">
        <div className="tree-dir">{node.name}</div>
        <div className="tree-children">
          {node.children.map((child) => (
            <TreeNodeView
              key={`${child.type}:${child.path}`}
              node={child}
              onOpen={onOpen}
            />
          ))}
        </div>
      </div>
    )
  }

  return (
    <button className="tree-request" onClick={() => void onOpen(node.path)}>
      <span className="method-tag">{node.method}</span>
      <span>{node.title}</span>
    </button>
  )
}

function KeyValueEditor(props: {
  title: string
  items: RequestKeyValue[]
  onChange: (items: RequestKeyValue[]) => void
}) {
  const { title, items, onChange } = props

  return (
    <div className="card">
      <div className="card-header">
        <strong>{title}</strong>
        <button
          className="secondary-button"
          onClick={() => onChange([...items, { key: "", value: "" }])}
        >
          Add
        </button>
      </div>

      <div className="kv-list">
        {items.map((item, index) => (
          <div className="kv-row" key={`${title}-${index}`}>
            <input
              placeholder="key"
              value={item.key}
              onChange={(event) =>
                onChange(
                  items.map((current, currentIndex) =>
                    currentIndex === index
                      ? { ...current, key: event.target.value }
                      : current
                  )
                )
              }
            />
            <input
              placeholder="value"
              value={item.value}
              onChange={(event) =>
                onChange(
                  items.map((current, currentIndex) =>
                    currentIndex === index
                      ? { ...current, value: event.target.value }
                      : current
                  )
                )
              }
            />
            <button
              className="danger-button"
              onClick={() =>
                onChange(
                  items.filter((_, currentIndex) => currentIndex !== index)
                )
              }
            >
              Remove
            </button>
          </div>
        ))}
      </div>
    </div>
  )
}

function definitionToDraft(
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
    headers: definition.request.headers,
    body:
      definition.request.body.type === "json"
        ? { type: "json", text: definition.request.body.text }
        : { type: "form", form: definition.request.body.form }
  }
}

function sanitizePairs(items: RequestKeyValue[]): RequestKeyValue[] {
  return items.filter((item) => item.key.trim() !== "")
}

function toErrorMessage(cause: unknown): string {
  if (cause instanceof Error) {
    return cause.message
  }
  return String(cause)
}
