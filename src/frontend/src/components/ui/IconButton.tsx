import type { ButtonHTMLAttributes } from "react"

import { Button } from "./Button"
import { Icon, type IconName } from "./Icon"

type Props = Omit<ButtonHTMLAttributes<HTMLButtonElement>, "children"> & {
  icon: IconName
}

export function IconButton(props: Props) {
  const { className, icon, type = "button", ...rest } = props

  return (
    <Button
      {...rest}
      className={["icon-button", className].filter(Boolean).join(" ")}
      type={type}
    >
      <Icon name={icon} />
    </Button>
  )
}
