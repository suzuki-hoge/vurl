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

  test("filters request nodes by space-separated AND terms regardless of order", () => {
    const nodes: RequestTreeNode[] = [
      {
        type: "directory",
        name: "logs",
        path: "logs",
        children: [
          {
            type: "request",
            name: "sleep-add.yaml",
            path: "logs/sleep-add.yaml",
            title: "sleep add",
            method: "POST"
          },
          {
            type: "request",
            name: "sleep-edit.yaml",
            path: "logs/sleep-edit.yaml",
            title: "sleep edit",
            method: "PATCH"
          },
          {
            type: "request",
            name: "meal-add.yaml",
            path: "logs/meal-add.yaml",
            title: "meal add",
            method: "POST"
          }
        ]
      }
    ]

    expect(filterNodes(nodes, "sleep")[0]).toMatchObject({
      type: "directory",
      children: [
        { path: "logs/sleep-add.yaml" },
        { path: "logs/sleep-edit.yaml" }
      ]
    })
    expect(filterNodes(nodes, "add")[0]).toMatchObject({
      type: "directory",
      children: [
        { path: "logs/sleep-add.yaml" },
        { path: "logs/meal-add.yaml" }
      ]
    })
    expect(filterNodes(nodes, "add sleep")[0]).toMatchObject({
      type: "directory",
      children: [{ path: "logs/sleep-add.yaml" }]
    })
  })
})
