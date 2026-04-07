import { afterEach, describe, expect, test, vi } from "vitest"

import {
  buildRequestUrl,
  getProjectFromLocation,
  getRequestPathFromLocation,
  hasRequestPath,
  openProject
} from "./location"

describe("location helpers", () => {
  afterEach(() => {
    vi.unstubAllGlobals()
  })

  test("reads project and request path from window location", () => {
    vi.stubGlobal("window", {
      location: {
        pathname: "/project-1",
        search: "?path=requests%2Fget-user.yaml"
      }
    })

    expect(getProjectFromLocation()).toBe("project-1")
    expect(getRequestPathFromLocation()).toBe("requests/get-user.yaml")
  })

  test("buildRequestUrl encodes project and path", () => {
    expect(buildRequestUrl("project 1", "requests/get user.yaml")).toBe(
      "/project%201?path=requests%2Fget%20user.yaml"
    )
  })

  test("openProject navigates to encoded project path", () => {
    const assign = vi.fn()
    vi.stubGlobal("window", {
      location: {
        assign
      }
    })

    openProject("project 1")
    expect(assign).toHaveBeenCalledWith("/project%201")
  })

  test("hasRequestPath walks nested directories", () => {
    expect(
      hasRequestPath(
        [
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
        ],
        "users/get-user.yaml"
      )
    ).toBe(true)
  })
})
