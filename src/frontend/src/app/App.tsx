import { useEffect, useMemo, useState } from "react"
import toast from "react-hot-toast"

import { apiClient } from "../api/client"
import { HeaderEditor } from "../components/request/HeaderEditor"
import { KeyValueEditor } from "../components/request/KeyValueEditor"
import { TreeNodeView } from "../components/tree/TreeNodeView"
import { Button } from "../components/ui/Button"
import { IconButton } from "../components/ui/IconButton"
import { Input } from "../components/ui/Input"
import { JsonCodeBlock } from "../components/ui/JsonCodeBlock"
import { Select } from "../components/ui/Select"
import { Tab, TabList } from "../components/ui/Tabs"
import {
  buildEffectiveHeaders,
  definitionToDraft,
  emptyDraft,
  persistHeaders,
  sanitizeHeaderPairs,
  sanitizePairs,
  type DraftState
} from "../features/request/model"
import {
  formatResponseBody,
  getResponseContentType,
  isImageContentType,
  isJsonContentType,
  statusToneClass
} from "../features/response/model"
import {
  getProjectFromLocation,
  getRequestPathFromLocation,
  hasRequestPath,
  openProject,
  updateRequestPathInLocation
} from "../lib/location"
import { formatMethodLabel } from "../lib/http"
import { filterNodes } from "../lib/tree"
import type {
  AuthPresetSummary,
  DefinitionResponse,
  EnvironmentSummary,
  ResponseNotification,
  RequestTreeNode,
  SendResponse
} from "../types/api"
import "./App.scss"

type RequestTab = "query" | "headers" | "body"
type ResponseTab = "code" | "headers" | "body"

export function App() {
  const [projects, setProjects] = useState<string[]>([])
  const [routeProject, setRouteProject] = useState(getProjectFromLocation())
  const [routeRequestPath, setRouteRequestPath] = useState(
    getRequestPathFromLocation()
  )
  const [project, setProject] = useState(() => getProjectFromLocation() ?? "")
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
  const [reloading, setReloading] = useState(false)
  const [sending, setSending] = useState(false)
  const effectiveHeaders = useMemo(() => buildEffectiveHeaders(draft), [draft])

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
        headers: sanitizeHeaderPairs(effectiveHeaders),
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
      for (const notification of result.notifications) {
        showToast(notification)
      }
      if (
        !result.notifications.some(
          (notification) => notification.code === "timeout"
        )
      ) {
        showReceivedToast(result.status)
      }
    } catch (cause) {
      setError(toErrorMessage(cause))
    } finally {
      setSending(false)
    }
  }

  async function reloadYaml() {
    try {
      setReloading(true)
      setError("")
      await apiClient.reload()
      window.location.reload()
    } catch (cause) {
      setError(toErrorMessage(cause))
    } finally {
      setReloading(false)
    }
  }

  function showToast(notification: ResponseNotification) {
    if (notification.code === "authenticated") {
      toast("Authenticated", {
        icon: "🔐",
        duration: 4000
      })
      return
    }

    if (notification.code === "timeout") {
      toast.error("Timed out")
      return
    }

    if (notification.kind === "error") {
      toast.error(notification.message)
      return
    }

    toast.success(notification.message)
  }

  function showReceivedToast(status: number) {
    const message = `${status} Received`
    if (status >= 400) {
      toast.error(message)
      return
    }

    toast.success(message)
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

  if (!project) {
    return (
      <div className="project-home">
        <div className="project-home-card">
          <div className="panel-title">vurl</div>
          <p className="project-home-copy">Project を選択してください。</p>
          {error ? <div className="error-banner">{error}</div> : null}
          <div className="project-list">
            {projects.map((item) => (
              <Button
                key={item}
                className="project-link"
                onClick={() => openProject(item)}
                type="button"
              >
                {item}
              </Button>
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
          <Input
            icon="search"
            value={filterText}
            onChange={(event) => setFilterText(event.target.value)}
            placeholder="path or name"
            wrapperClassName="search-field"
          />
          <IconButton
            className="sidebar-reload"
            disabled={reloading}
            icon="reload"
            onClick={() => {
              void reloadYaml()
            }}
            aria-label="Reload YAML"
            title="Reload YAML"
          />
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
                  {draft.method ? formatMethodLabel(draft.method) : "-"}
                </span>
                <div className="request-name">{draft.name || "-"}</div>
              </div>

              <Input
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
                  <Button
                    className={`auth-toggle${authExpanded ? " is-expanded" : ""}`}
                    onClick={() => setAuthExpanded((current) => !current)}
                    type="button"
                  >
                    <span>Authentication</span>
                    <span className="tree-dir-caret" aria-hidden="true">
                      {authExpanded ? "▾" : "▸"}
                    </span>
                  </Button>

                  <div
                    className={`auth-collapse${authExpanded ? " is-expanded" : ""}`}
                  >
                    <div className="auth-collapse-inner">
                      <div className="auth-mode-row">
                        <label className="radio-pill">
                          <Input
                            checked={authInputMode === "preset"}
                            disabled={authPresets.length === 0}
                            name="auth-mode"
                            onChange={() => setAuthInputMode("preset")}
                            type="radio"
                          />
                          <span>preset</span>
                        </label>
                        <label className="radio-pill">
                          <Input
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
                          <Select
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
                          </Select>
                        ) : (
                          <>
                            <Input
                              className="request-inline-input"
                              onChange={(event) => {
                                setAuthId(event.target.value)
                              }}
                              placeholder="ID"
                              value={authId}
                            />
                            <Input
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

            <div className="panel-scroll panel-scroll-tabs">
              <TabList>
                <Tab
                  active={requestTab === "query"}
                  onClick={() => setRequestTab("query")}
                >
                  Query
                </Tab>
                <Tab
                  active={requestTab === "headers"}
                  onClick={() => setRequestTab("headers")}
                >
                  Headers
                </Tab>
                <Tab
                  active={requestTab === "body"}
                  onClick={() => setRequestTab("body")}
                >
                  Body
                </Tab>
              </TabList>

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
                  <HeaderEditor
                    items={effectiveHeaders}
                    onAdd={() =>
                      setDraft((current) => ({
                        ...current,
                        headers: [
                          ...current.headers,
                          { key: "", value: "", state: "editable" }
                        ]
                      }))
                    }
                    onChange={(items) =>
                      setDraft((current) => ({
                        ...current,
                        headers: persistHeaders(current, items)
                      }))
                    }
                  />
                ) : null}

                {requestTab === "body" ? (
                  draft.body.type === "json" ? (
                    <JsonCodeBlock
                      editable
                      text={draft.body.text}
                      onChange={(event) =>
                        setDraft((current) => ({
                          ...current,
                          body: { type: "json", text: event.target.value }
                        }))
                      }
                    />
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
              <Select
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
              </Select>
              <Button
                className="primary-button request-send"
                disabled={sending}
                type="submit"
                variant="primary"
              >
                Send
              </Button>
            </div>
          </form>
        </section>

        <section className="app-panel app-response">
          <div className="panel-layout">
            <div className="panel-scroll panel-scroll-tabs">
              <TabList>
                <Tab
                  active={responseTab === "code"}
                  onClick={() => setResponseTab("code")}
                >
                  Code
                </Tab>
                <Tab
                  active={responseTab === "headers"}
                  onClick={() => setResponseTab("headers")}
                >
                  Headers
                </Tab>
                <Tab
                  active={responseTab === "body"}
                  onClick={() => setResponseTab("body")}
                >
                  Body
                </Tab>
              </TabList>

              <div className="tab-panel">
                {responseTab === "code" ? (
                  <div className="card fill-card">
                    <pre
                      className={`response-block fill-area ${statusToneClass(response?.status)}`}
                    >
                      {response?.status ?? "-"}
                    </pre>
                  </div>
                ) : null}

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
                      <JsonCodeBlock text={formattedResponseBody} />
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

function toErrorMessage(cause: unknown): string {
  if (cause instanceof Error) {
    return cause.message
  }
  return String(cause)
}
