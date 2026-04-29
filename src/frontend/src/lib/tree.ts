import type { RequestTreeNode } from "../types/api"

function splitFilterTerms(filter: string): string[] {
  return filter
    .toLowerCase()
    .trim()
    .split(/\s+/)
    .filter((term) => term.length > 0)
}

function matchesAllTerms(haystack: string, terms: string[]): boolean {
  return terms.every((term) => haystack.includes(term))
}

export function filterNodes(
  nodes: RequestTreeNode[],
  filter: string
): RequestTreeNode[] {
  const terms = splitFilterTerms(filter)

  if (terms.length === 0) {
    return nodes
  }

  const result: RequestTreeNode[] = []

  for (const node of nodes) {
    if (node.type === "request") {
      const haystack = `${node.path} ${node.name}`.toLowerCase()
      if (matchesAllTerms(haystack, terms)) {
        result.push(node)
      }
      continue
    }

    const children = filterNodes(node.children, filter)
    if (
      children.length === 0 &&
      !matchesAllTerms(node.path.toLowerCase(), terms)
    ) {
      continue
    }

    result.push({ ...node, children })
  }

  return result
}
