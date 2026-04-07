import type { SelectHTMLAttributes } from "react"

import { Icon } from "./Icon"

type Props = SelectHTMLAttributes<HTMLSelectElement>

export function Select(props: Props) {
  const { className, ...rest } = props

  return (
    <div className="ui-select">
      <select
        {...rest}
        className={["ui-select-element", className].filter(Boolean).join(" ")}
      />
      <span className="ui-select-icon" aria-hidden="true">
        <Icon name="chevron-down" />
      </span>
    </div>
  )
}
