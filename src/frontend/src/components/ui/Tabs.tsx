import type { ButtonHTMLAttributes, ReactNode } from "react"

type TabListProps = {
  children: ReactNode
  className?: string
}

type TabProps = Omit<ButtonHTMLAttributes<HTMLButtonElement>, "children"> & {
  active: boolean
  children: ReactNode
}

export function TabList(props: TabListProps) {
  const { children, className } = props

  return (
    <div
      className={["tab-list", className].filter(Boolean).join(" ")}
      role="tablist"
    >
      {children}
    </div>
  )
}

export function Tab(props: TabProps) {
  const { active, children, className, type = "button", ...rest } = props

  return (
    <button
      {...rest}
      aria-selected={active}
      className={["tab", active ? "is-active" : "", className]
        .filter(Boolean)
        .join(" ")}
      role="tab"
      tabIndex={active ? 0 : -1}
      type={type}
    >
      {children}
    </button>
  )
}
