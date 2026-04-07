import type { InputHTMLAttributes } from "react"

import { Icon, type IconName } from "./Icon"

type Props = InputHTMLAttributes<HTMLInputElement> & {
  icon?: IconName
  wrapperClassName?: string
}

export function Input(props: Props) {
  const { className, icon, type, wrapperClassName, ...rest } = props

  if (type === "radio" || type === "checkbox" || type === "hidden") {
    return <input {...rest} className={className} type={type} />
  }

  return (
    <div
      className={["ui-input", icon ? "has-icon" : "", wrapperClassName]
        .filter(Boolean)
        .join(" ")}
    >
      {icon ? (
        <span className="ui-input-icon" aria-hidden="true">
          <Icon name={icon} />
        </span>
      ) : null}
      <input
        {...rest}
        className={["ui-input-element", className].filter(Boolean).join(" ")}
        type={type}
      />
    </div>
  )
}
