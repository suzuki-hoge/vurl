import type { SelectHTMLAttributes } from "react"

type Props = SelectHTMLAttributes<HTMLSelectElement>

export function Select(props: Props) {
  return <select {...props} />
}
