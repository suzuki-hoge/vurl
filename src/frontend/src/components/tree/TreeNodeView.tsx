import type { RequestTreeNode } from "../../types/api"
import { buildRequestUrl } from "../../lib/location"
import { Button } from "../ui/Button"

export function TreeNodeView(props: {
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
        <Button
          className={`tree-dir-button${isExpanded ? " is-expanded" : ""}`}
          onClick={() => onToggleDirectory(node.path)}
          type="button"
        >
          <span className="tree-dir-caret" aria-hidden="true">
            {isExpanded ? "▾" : "▸"}
          </span>
          <span className="tree-dir">{node.name}</span>
        </Button>
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
