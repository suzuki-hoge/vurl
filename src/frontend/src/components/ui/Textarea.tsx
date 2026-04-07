import type { TextareaHTMLAttributes } from "react"

type Props = TextareaHTMLAttributes<HTMLTextAreaElement>

export function Textarea(props: Props) {
  return <textarea {...props} />
}
