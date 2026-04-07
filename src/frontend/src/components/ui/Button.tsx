import type { ButtonHTMLAttributes, ReactNode } from "react"

type ButtonVariant = "default" | "primary"

type Props = ButtonHTMLAttributes<HTMLButtonElement> & {
  children: ReactNode
  variant?: ButtonVariant
}

export function Button(props: Props) {
  const {
    children,
    className,
    type = "button",
    variant = "default",
    ...rest
  } = props

  const variantClass = variant === "primary" ? "primary-button" : ""

  return (
    <button
      {...rest}
      className={["ui-button", variantClass, className]
        .filter(Boolean)
        .join(" ")}
      type={type}
    >
      {children}
    </button>
  )
}
