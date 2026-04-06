import { describe, expect, test } from "vitest"

import type { RequestTreeNode } from "../types/api"
import { filterNodes } from "./tree"

describe("filterNodes", () => {
  test("filters request nodes by name or path and keeps parent directory", () => {
    const nodes: RequestTreeNode[] = [
      {
        type: "directory",
        name: "users",
        path: "users",
        children: [
          {
            type: "request",
            name: "get-user.yaml",
            path: "users/get-user.yaml",
            title: "Get User",
            method: "GET"
          }
        ]
      }
    ]

    const filtered = filterNodes(nodes, "get-user")
    expect(filtered).toHaveLength(1)
    expect(filtered[0]?.type).toBe("directory")
  })
})
