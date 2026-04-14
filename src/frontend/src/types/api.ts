export type ProjectSummary = {
  name: string
}

export type EnvironmentSummary = {
  name: string
  auth_presets: AuthPresetSummary[]
}

export type AuthPresetSummary = {
  name: string
}

export type RequestTreeNode =
  | {
      type: "directory"
      name: string
      path: string
      children: RequestTreeNode[]
    }
  | {
      type: "request"
      name: string
      path: string
      title: string
      method: string
    }

export type RequestKeyValue = {
  key: string
  value: string
}

export type RequestFormSelectItem = {
  value: string
  description: string
  default: boolean
}

export type RequestFormField = {
  key: string
  value?: string
  enabled: boolean
  items: RequestFormSelectItem[]
}

export type RequestDefinition = {
  name: string
  method: string
  path: string
  auth: boolean
  request: {
    query: RequestKeyValue[]
    headers: RequestKeyValue[]
    body:
      | {
          type: "json"
          text: string
        }
      | {
          type: "form"
          form: RequestFormField[]
        }
  }
}

export type DefinitionResponse = {
  path: string
  definition: RequestDefinition
}

export type TreeResponse = {
  project: string
  nodes: RequestTreeNode[]
}

export type RuntimeInfo = {
  root: string
  projects: ProjectSummary[]
  backend_url: string
}

export type ReloadResponse = {
  success: boolean
  message: string
  project_count: number
}

export type SendRequestPayload = {
  project: string
  environment: string
  path: string
  method: string
  url_path: string
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
  auth_enabled: boolean
  auth_input_mode: "preset" | "manual"
  auth_preset_name?: string
  auth_credentials: {
    id: string
    password: string
  }
}

export type SendResponse = {
  status: number
  headers: RequestKeyValue[]
  content_type?: string
  body: string
  body_base64?: string
  retried_auth: boolean
  notifications: ResponseNotification[]
  current_log_file: string
}

export type ResponseNotification = {
  code: "authenticated" | "timeout" | "generic"
  kind: "info" | "error"
  message: string
}
