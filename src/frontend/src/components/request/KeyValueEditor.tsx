import type { RequestKeyValue } from "../../types/api"
import { IconButton } from "../ui/IconButton"
import { Input } from "../ui/Input"

export function KeyValueEditor(props: {
  items: RequestKeyValue[]
  onChange: (items: RequestKeyValue[]) => void
}) {
  const { items, onChange } = props

  return (
    <div className="card fill-card">
      <div className="kv-list">
        {items.map((item, index) => (
          <div className="kv-row" key={index}>
            <Input
              placeholder="key"
              value={item.key}
              onChange={(event) =>
                onChange(
                  items.map((current, currentIndex) =>
                    currentIndex === index
                      ? { ...current, key: event.target.value }
                      : current
                  )
                )
              }
            />
            <Input
              placeholder="value"
              value={item.value}
              onChange={(event) =>
                onChange(
                  items.map((current, currentIndex) =>
                    currentIndex === index
                      ? { ...current, value: event.target.value }
                      : current
                  )
                )
              }
            />
            <IconButton
              aria-label="Remove row"
              className="danger"
              icon="trash"
              onClick={() =>
                onChange(
                  items.filter((_, currentIndex) => currentIndex !== index)
                )
              }
            />
          </div>
        ))}
      </div>

      <div className="kv-footer">
        <IconButton
          aria-label="Add row"
          icon="plus"
          onClick={() => onChange([...items, { key: "", value: "" }])}
        />
      </div>
    </div>
  )
}
