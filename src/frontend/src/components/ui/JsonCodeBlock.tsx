import { useRef, type TextareaHTMLAttributes, type UIEvent } from "react"

import { JsonHighlightedText } from "../request/JsonHighlightedText"
import { Textarea } from "./Textarea"

type Props = {
  text: string
  editable?: boolean
  onChange?: TextareaHTMLAttributes<HTMLTextAreaElement>["onChange"]
}

export function JsonCodeBlock(props: Props) {
  const { text, editable = false, onChange } = props
  const highlightRef = useRef<HTMLPreElement | null>(null)

  function syncScroll(event: UIEvent<HTMLTextAreaElement>) {
    const highlight = highlightRef.current
    if (!highlight) {
      return
    }

    highlight.scrollTop = event.currentTarget.scrollTop
    highlight.scrollLeft = event.currentTarget.scrollLeft
  }

  if (!editable) {
    return (
      <pre className="response-block response-json fill-area">
        <JsonHighlightedText text={text} />
      </pre>
    )
  }

  return (
    <div className="card fill-card json-code-card">
      <pre
        ref={highlightRef}
        aria-hidden="true"
        className="response-block response-json json-code-highlight fill-area"
      >
        <span className="json-code-line">
          <JsonHighlightedText text={text} />
        </span>
      </pre>
      <Textarea
        className="body-textarea json-code-textarea fill-area"
        onChange={onChange}
        onScroll={syncScroll}
        value={text}
        wrap="off"
      />
    </div>
  )
}
