import { highlightJson } from "../../features/response/model"

export function JsonHighlightedText(props: { text: string }) {
  const tokens = highlightJson(props.text)

  return (
    <>
      {tokens.map((token, index) => (
        <span key={index} className={`json-token json-token-${token.type}`}>
          {token.value}
        </span>
      ))}
    </>
  )
}
