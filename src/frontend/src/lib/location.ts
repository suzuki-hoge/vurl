import type { RequestTreeNode } from "../types/api"

export function getProjectFromLocation(): string | null {
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

export function getRequestPathFromLocation(): string | null {
  if (typeof window === "undefined") {
    return null
  }

  const path = new URLSearchParams(window.location.search).get("path")
  return path || null
}

export function openProject(project: string) {
  if (typeof window === "undefined") {
    return
  }

  window.location.assign(`/${encodeURIComponent(project)}`)
}

export function buildRequestUrl(project: string, path: string): string {
  return `/${encodeURIComponent(project)}?path=${encodeURIComponent(path)}`
}

export function updateRequestPathInLocation(project: string, path: string) {
  if (typeof window === "undefined") {
    return
  }

  window.history.pushState({}, "", buildRequestUrl(project, path))
}

export function hasRequestPath(
  nodes: RequestTreeNode[],
  targetPath: string
): boolean {
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
