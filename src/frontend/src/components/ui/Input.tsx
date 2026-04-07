import type { InputHTMLAttributes } from "react"

type Props = InputHTMLAttributes<HTMLInputElement>

export function Input(props: Props) {
  return <input {...props} />
}
