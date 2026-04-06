import type { RequestTreeNode } from "../types/api"

export function filterNodes(
  nodes: RequestTreeNode[],
  filter: string
): RequestTreeNode[] {
  if (!filter) {
    return nodes
  }

  const result: RequestTreeNode[] = []

  for (const node of nodes) {
    if (node.type === "request") {
      const haystack = `${node.path} ${node.name}`.toLowerCase()
      if (haystack.includes(filter)) {
        result.push(node)
      }
      continue
    }

    const children = filterNodes(node.children, filter)
    if (children.length === 0 && !node.path.toLowerCase().includes(filter)) {
      continue
    }

    result.push({ ...node, children })
  }

  return result
}
