import { useEffect, useMemo, useState } from "react"
import { FiPlus, FiSearch, FiTrash2 } from "react-icons/fi"

import { apiClient } from "../api/client"
import { filterNodes } from "../lib/tree"
import type {
  AuthPresetSummary,
  DefinitionResponse,
  EnvironmentSummary,
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

type RequestTab = "query" | "headers" | "body"
type ResponseTab = "headers" | "body"

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
  const [routeProject, setRouteProject] = useState(getProjectFromLocation())
  const [routeRequestPath, setRouteRequestPath] = useState(
    getRequestPathFromLocation()
  )
  const [project, setProject] = useState("")
  const [environmentSummaries, setEnvironmentSummaries] = useState<
    EnvironmentSummary[]
  >([])
  const [environments, setEnvironments] = useState<string[]>([])
  const [environment, setEnvironment] = useState("")
  const [tree, setTree] = useState<RequestTreeNode[]>([])
  const [filterText, setFilterText] = useState("")
  const [expandedDirectories, setExpandedDirectories] = useState<
    Record<string, boolean>
  >({})
  const [draft, setDraft] = useState<DraftState>(emptyDraft)
  const [response, setResponse] = useState<SendResponse | null>(null)
  const [authInputMode, setAuthInputMode] = useState<"preset" | "manual">(
    "preset"
  )
  const [authPresets, setAuthPresets] = useState<AuthPresetSummary[]>([])
  const [selectedAuthPreset, setSelectedAuthPreset] = useState("__manual__")
  const [authId, setAuthId] = useState("")
  const [authPassword, setAuthPassword] = useState("")
  const [authExpanded, setAuthExpanded] = useState(false)
  const [requestTab, setRequestTab] = useState<RequestTab>("body")
  const [responseTab, setResponseTab] = useState<ResponseTab>("body")
  const [error, setError] = useState("")
  const [loading, setLoading] = useState(true)
  const [sending, setSending] = useState(false)

  useEffect(() => {
    void bootstrap()
  }, [])

  useEffect(() => {
    if (typeof window === "undefined") {
      return
    }

    const onPopState = () => {
      setRouteProject(getProjectFromLocation())
      setRouteRequestPath(getRequestPathFromLocation())
    }

    window.addEventListener("popstate", onPopState)
    return () => window.removeEventListener("popstate", onPopState)
  }, [])

  useEffect(() => {
    if (!project) {
      return
    }
    void loadProjectData(project)
  }, [project])

  useEffect(() => {
    if (!project || !routeRequestPath) {
      return
    }

    if (!hasRequestPath(tree, routeRequestPath)) {
      return
    }

    void openDefinition(routeRequestPath, { syncUrl: false })
  }, [project, routeRequestPath, tree])

  useEffect(() => {
    if (!projects.length) {
      return
    }

    if (routeProject && projects.includes(routeProject)) {
      setProject(routeProject)
      return
    }

    setProject("")
  }, [projects, routeProject])

  useEffect(() => {
    if (typeof document === "undefined") {
      return
    }

    document.title =
      draft.name || (routeProject ? `vurl - ${routeProject}` : "vurl")
  }, [draft.name, routeProject])

  useEffect(() => {
    const summary = environmentSummaries.find(
      (item) => item.name === environment
    )
    const presets = summary?.auth_presets ?? []
    setAuthPresets(presets)

    if (presets.length === 0) {
      setAuthInputMode("manual")
      setSelectedAuthPreset("__manual__")
      return
    }

    setAuthInputMode("preset")
    setSelectedAuthPreset((current) =>
      current !== "__manual__" &&
      presets.some((preset) => preset.name === current)
        ? current
        : presets[0].name
    )
  }, [environment, environmentSummaries])

  useEffect(() => {
    if (authInputMode === "preset") {
      setAuthId("")
      setAuthPassword("")
    }
  }, [authInputMode])

  async function bootstrap() {
    try {
      setLoading(true)
      const runtime = await apiClient.runtime()
      const nextProjects = runtime.projects.map((item) => item.name)
      setProjects(nextProjects)

      if (routeProject && nextProjects.includes(routeProject)) {
        setProject(routeProject)
      } else if (routeProject) {
        setProject("")
      } else if (nextProjects.length > 0) {
        setProject("")
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
      setEnvironmentSummaries(envs)
      const nextEnvironments = envs.map((item) => item.name)
      setEnvironments(nextEnvironments)
      setEnvironment((current) =>
        current && nextEnvironments.includes(current)
          ? current
          : (nextEnvironments[0] ?? "")
      )
      setTree(treeResponse.nodes)
      setExpandedDirectories({})
      setDraft((current) => ({ ...current, project: nextProject }))
    } catch (cause) {
      setError(toErrorMessage(cause))
    }
  }

  async function openDefinition(
    path: string,
    options?: {
      syncUrl?: boolean
    }
  ) {
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
      setAuthExpanded(false)
      setRequestTab("body")
      setResponseTab("body")
      setResponse(null)
      if (options?.syncUrl !== false) {
        updateRequestPathInLocation(project, path)
        setRouteRequestPath(path)
      }
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
        auth_input_mode: authInputMode,
        auth_preset_name:
          authInputMode === "preset" && selectedAuthPreset !== "__manual__"
            ? selectedAuthPreset
            : undefined,
        auth_credentials: {
          id: authId,
          password: authPassword
        }
      })
      setResponse(result)
      setResponseTab("body")
    } catch (cause) {
      setError(toErrorMessage(cause))
    } finally {
      setSending(false)
    }
  }

  const filteredTree = useMemo(
    () => filterNodes(tree, filterText.trim().toLowerCase()),
    [tree, filterText]
  )
  const isFiltering = filterText.trim().length > 0

  const responseContentType = getResponseContentType(response)
  const formattedResponseBody = formatResponseBody(response)
  const responseImageSource =
    response && isImageContentType(responseContentType) && response.body_base64
      ? `data:${responseContentType};base64,${response.body_base64}`
      : null

  const routeProjectExists = !routeProject || projects.includes(routeProject)

  if (!routeProject) {
    return (
      <div className="project-home">
        <div className="project-home-card">
          <div className="panel-title">vurl</div>
          <p className="project-home-copy">Project を選択してください。</p>
          {error ? <div className="error-banner">{error}</div> : null}
          <div className="project-list">
            {projects.map((item) => (
              <button
                key={item}
                className="project-link"
                onClick={() => openProject(item)}
                type="button"
              >
                {item}
              </button>
            ))}
          </div>
        </div>
      </div>
    )
  }

  if (!routeProjectExists) {
    return (
      <div className="project-home">
        <div className="project-home-card">
          <div className="panel-title">vurl</div>
          <div className="error-banner">project not found: {routeProject}</div>
          <div className="project-list">
            {projects.map((item) => (
              <button
                key={item}
                className="project-link"
                onClick={() => openProject(item)}
                type="button"
              >
                {item}
              </button>
            ))}
          </div>
        </div>
      </div>
    )
  }

  return (
    <div className="app-shell">
      <aside className="app-sidebar">
        <div className="sidebar-header">
          <label className="search-field">
            <span className="search-icon" aria-hidden="true">
              <FiSearch />
            </span>
            <input
              value={filterText}
              onChange={(event) => setFilterText(event.target.value)}
              placeholder="path or name"
            />
          </label>
        </div>

        <div className="tree-list">
          {loading ? <div className="muted">loading...</div> : null}
          {filteredTree.map((node) => (
            <TreeNodeView
              key={`${node.type}:${node.path}`}
              node={node}
              onOpen={openDefinition}
              project={project}
              expandedDirectories={expandedDirectories}
              onToggleDirectory={(path) =>
                setExpandedDirectories((current) => ({
                  ...current,
                  [path]: !current[path]
                }))
              }
              forceExpanded={isFiltering}
            />
          ))}
        </div>
      </aside>

      <main className="app-main">
        <section className="app-panel app-request">
          <form
            className="panel-layout"
            onSubmit={(event) => {
              event.preventDefault()
              void sendRequest()
            }}
          >
            <div className="panel-fixed">
              {error ? <div className="error-banner">{error}</div> : null}

              <div className="request-heading">
                <span
                  className={`method-tag request-method method-${draft.method.toLowerCase()}`}
                >
                  {draft.method || "-"}
                </span>
                <div className="request-name">{draft.name || "-"}</div>
              </div>

              <input
                className="request-path-input"
                value={draft.urlPath}
                onChange={(event) =>
                  setDraft((current) => ({
                    ...current,
                    urlPath: event.target.value
                  }))
                }
                placeholder="path"
              />

              {draft.auth ? (
                <div className="auth-panel">
                  <button
                    className={`auth-toggle${authExpanded ? " is-expanded" : ""}`}
                    onClick={() => setAuthExpanded((current) => !current)}
                    type="button"
                  >
                    <span>Authentication</span>
                    <span className="tree-dir-caret" aria-hidden="true">
                      {authExpanded ? "▾" : "▸"}
                    </span>
                  </button>

                  <div
                    className={`auth-collapse${authExpanded ? " is-expanded" : ""}`}
                  >
                    <div className="auth-collapse-inner">
                      <div className="auth-mode-row">
                        <label className="radio-pill">
                          <input
                            checked={authInputMode === "preset"}
                            disabled={authPresets.length === 0}
                            name="auth-mode"
                            onChange={() => setAuthInputMode("preset")}
                            type="radio"
                          />
                          <span>preset</span>
                        </label>
                        <label className="radio-pill">
                          <input
                            checked={authInputMode === "manual"}
                            name="auth-mode"
                            onChange={() => setAuthInputMode("manual")}
                            type="radio"
                          />
                          <span>manual</span>
                        </label>
                      </div>

                      <div className="auth-inputs">
                        {authInputMode === "preset" ? (
                          <select
                            className="request-inline-select"
                            value={selectedAuthPreset}
                            onChange={(event) =>
                              setSelectedAuthPreset(event.target.value)
                            }
                          >
                            {authPresets.map((preset) => (
                              <option key={preset.name} value={preset.name}>
                                {preset.name}
                              </option>
                            ))}
                          </select>
                        ) : (
                          <>
                            <input
                              className="request-inline-input"
                              onChange={(event) => {
                                setAuthId(event.target.value)
                              }}
                              placeholder="ID"
                              value={authId}
                            />
                            <input
                              className="request-inline-input"
                              onChange={(event) => {
                                setAuthPassword(event.target.value)
                              }}
                              placeholder="Password"
                              type="text"
                              value={authPassword}
                            />
                          </>
                        )}
                      </div>
                    </div>
                  </div>
                </div>
              ) : null}
            </div>

            <div className="panel-scroll">
              <div className="tab-bar">
                <TabButton
                  active={requestTab === "query"}
                  label="Query"
                  onClick={() => setRequestTab("query")}
                />
                <TabButton
                  active={requestTab === "headers"}
                  label="Headers"
                  onClick={() => setRequestTab("headers")}
                />
                <TabButton
                  active={requestTab === "body"}
                  label="Body"
                  onClick={() => setRequestTab("body")}
                />
              </div>

              <div className="tab-panel">
                {requestTab === "query" ? (
                  <KeyValueEditor
                    items={draft.query}
                    onChange={(items) =>
                      setDraft((current) => ({ ...current, query: items }))
                    }
                  />
                ) : null}

                {requestTab === "headers" ? (
                  <KeyValueEditor
                    items={draft.headers}
                    onChange={(items) =>
                      setDraft((current) => ({ ...current, headers: items }))
                    }
                  />
                ) : null}

                {requestTab === "body" ? (
                  draft.body.type === "json" ? (
                    <div className="card fill-card">
                      <textarea
                        className="body-textarea fill-area"
                        value={draft.body.text}
                        onChange={(event) =>
                          setDraft((current) => ({
                            ...current,
                            body: { type: "json", text: event.target.value }
                          }))
                        }
                      />
                    </div>
                  ) : (
                    <KeyValueEditor
                      items={draft.body.form}
                      onChange={(items) =>
                        setDraft((current) => ({
                          ...current,
                          body: { type: "form", form: items }
                        }))
                      }
                    />
                  )
                ) : null}
              </div>
            </div>

            <div className="request-footer">
              <select
                className="request-environment-select"
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
              <button
                className="primary-button request-send"
                disabled={sending}
                type="submit"
              >
                {sending ? "Sending..." : "Send"}
              </button>
            </div>
          </form>
        </section>

        <section className="app-panel app-response">
          <div className="panel-layout">
            <div className="panel-scroll">
              <div className="tab-bar response-tab-bar">
                <div
                  className={`status-pill ${statusToneClass(response?.status)}`}
                >
                  {response?.status ?? "-"}
                </div>
                <TabButton
                  active={responseTab === "headers"}
                  label="Headers"
                  onClick={() => setResponseTab("headers")}
                />
                <TabButton
                  active={responseTab === "body"}
                  label="Body"
                  onClick={() => setResponseTab("body")}
                />
              </div>

              <div className="tab-panel">
                {responseTab === "headers" ? (
                  <div className="card fill-card">
                    <pre className="response-block fill-area">
                      {response?.headers
                        .map((header) => `${header.key}: ${header.value}`)
                        .join("\n") ?? ""}
                    </pre>
                  </div>
                ) : null}

                {responseTab === "body" ? (
                  <div className="card fill-card">
                    {responseImageSource ? (
                      <div className="response-image-wrap fill-area">
                        <img
                          alt="response body"
                          className="response-image"
                          src={responseImageSource}
                        />
                      </div>
                    ) : isJsonContentType(responseContentType) ? (
                      <pre className="response-block response-json fill-area">
                        <JsonHighlightedText text={formattedResponseBody} />
                      </pre>
                    ) : (
                      <pre className="response-block fill-area">
                        {formattedResponseBody}
                      </pre>
                    )}
                  </div>
                ) : null}
              </div>
            </div>
          </div>
        </section>
      </main>
    </div>
  )
}

function TreeNodeView(props: {
  node: RequestTreeNode
  onOpen: (path: string) => void | Promise<void>
  project: string
  expandedDirectories: Record<string, boolean>
  onToggleDirectory: (path: string) => void
  forceExpanded: boolean
}) {
  const {
    node,
    onOpen,
    project,
    expandedDirectories,
    onToggleDirectory,
    forceExpanded
  } = props

  if (node.type === "directory") {
    const isExpanded = forceExpanded || expandedDirectories[node.path] === true

    return (
      <div className="tree-node">
        <button
          className={`tree-dir-button${isExpanded ? " is-expanded" : ""}`}
          onClick={() => onToggleDirectory(node.path)}
          type="button"
        >
          <span className="tree-dir-caret" aria-hidden="true">
            {isExpanded ? "▾" : "▸"}
          </span>
          <span className="tree-dir">{node.name}</span>
        </button>
        <div
          className={`tree-children-wrap${isExpanded ? " is-expanded" : ""}`}
        >
          <div className="tree-children">
            {node.children.map((child) => (
              <TreeNodeView
                key={`${child.type}:${child.path}`}
                node={child}
                onOpen={onOpen}
                project={project}
                expandedDirectories={expandedDirectories}
                onToggleDirectory={onToggleDirectory}
                forceExpanded={forceExpanded}
              />
            ))}
          </div>
        </div>
      </div>
    )
  }

  const href = buildRequestUrl(project, node.path)

  return (
    <a
      className={`tree-request method-${node.method.toLowerCase()}`}
      href={href}
      onClick={(event) => {
        if (
          event.defaultPrevented ||
          event.metaKey ||
          event.ctrlKey ||
          event.shiftKey ||
          event.altKey ||
          event.button !== 0
        ) {
          return
        }

        event.preventDefault()
        void onOpen(node.path)
      }}
    >
      <span className={`method-tag method-${node.method.toLowerCase()}`}>
        {node.method}
      </span>
      <span>{node.title}</span>
    </a>
  )
}

function KeyValueEditor(props: {
  items: RequestKeyValue[]
  onChange: (items: RequestKeyValue[]) => void
}) {
  const { items, onChange } = props

  return (
    <div className="card fill-card">
      <div className="kv-list">
        {items.map((item, index) => (
          <div className="kv-row" key={index}>
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
              aria-label="Remove row"
              className="icon-button danger"
              onClick={() =>
                onChange(
                  items.filter((_, currentIndex) => currentIndex !== index)
                )
              }
              type="button"
            >
              <FiTrash2 />
            </button>
          </div>
        ))}
      </div>

      <div className="kv-footer">
        <button
          aria-label="Add row"
          className="icon-button"
          onClick={() => onChange([...items, { key: "", value: "" }])}
          type="button"
        >
          <FiPlus />
        </button>
      </div>
    </div>
  )
}

function TabButton(props: {
  active: boolean
  label: string
  onClick: () => void
}) {
  const { active, label, onClick } = props

  return (
    <button
      className={`tab-button${active ? " is-active" : ""}`}
      onClick={onClick}
      type="button"
    >
      {label}
    </button>
  )
}

function JsonHighlightedText(props: { text: string }) {
  const { text } = props
  const tokens = highlightJson(text)

  return (
    <>
      {tokens.map((token, index) => (
        <span key={index} className={`json-token json-token-${token.type}`}>
          {token.value}
        </span>
      ))}
    </>
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

function getResponseContentType(response: SendResponse | null): string {
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

function formatResponseBody(response: SendResponse | null): string {
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

function isJsonContentType(contentType: string): boolean {
  const mimeType = contentType.split(";")[0]?.trim().toLowerCase() ?? ""
  return mimeType === "application/json" || mimeType.endsWith("+json")
}

function isImageContentType(contentType: string): boolean {
  const mimeType = contentType.split(";")[0]?.trim().toLowerCase() ?? ""
  return mimeType.startsWith("image/")
}

function statusToneClass(status?: number): string {
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

function highlightJson(text: string): Array<{
  type:
    | "key"
    | "string"
    | "number"
    | "boolean"
    | "null"
    | "punctuation"
    | "plain"
  value: string
}> {
  const tokens: Array<{
    type:
      | "key"
      | "string"
      | "number"
      | "boolean"
      | "null"
      | "punctuation"
      | "plain"
    value: string
  }> = []

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

function sanitizePairs(items: RequestKeyValue[]): RequestKeyValue[] {
  return items.filter((item) => item.key.trim() !== "")
}

function getProjectFromLocation(): string | null {
  if (typeof window === "undefined") {
    return null
  }

  const path = window.location.pathname.replace(/^\/+|\/+$/g, "")
  if (!path) {
    return null
  }

  const [project] = path.split("/")
  return project ? decodeURIComponent(project) : null
}

function getRequestPathFromLocation(): string | null {
  if (typeof window === "undefined") {
    return null
  }

  const path = new URLSearchParams(window.location.search).get("path")
  return path || null
}

function openProject(project: string) {
  if (typeof window === "undefined") {
    return
  }

  window.location.assign(`/${encodeURIComponent(project)}`)
}

function buildRequestUrl(project: string, path: string): string {
  return `/${encodeURIComponent(project)}?path=${encodeURIComponent(path)}`
}

function updateRequestPathInLocation(project: string, path: string) {
  if (typeof window === "undefined") {
    return
  }

  window.history.pushState({}, "", buildRequestUrl(project, path))
}

function hasRequestPath(nodes: RequestTreeNode[], targetPath: string): boolean {
  for (const node of nodes) {
    if (node.type === "request" && node.path === targetPath) {
      return true
    }
    if (
      node.type === "directory" &&
      hasRequestPath(node.children, targetPath)
    ) {
      return true
    }
  }
  return false
}

function toErrorMessage(cause: unknown): string {
  if (cause instanceof Error) {
    return cause.message
  }
  return String(cause)
}
