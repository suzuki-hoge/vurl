import type { DraftFormField } from "../../features/request/model"
import { IconButton } from "../ui/IconButton"
import { Input } from "../ui/Input"
import { Select } from "../ui/Select"

export function BodyFormEditor(props: {
  items: DraftFormField[]
  onChange: (items: DraftFormField[]) => void
}) {
  const { items, onChange } = props

  return (
    <div className="card fill-card">
      <div className="kv-list">
        {items.map((item, index) => {
          const isEnabled = item.enabled

          return (
            <div
              className={`kv-row kv-row-body${isEnabled ? "" : " is-off"}`}
              key={item.key}
            >
              <Input readOnly value={item.key} />
              {item.items ? (
                <Select
                  disabled={!isEnabled}
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
                >
                  {item.items.map((option) => (
                    <option key={option.value} value={option.value}>
                      {option.description}
                    </option>
                  ))}
                </Select>
              ) : (
                <Input
                  disabled={!isEnabled}
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
              )}
              <IconButton
                aria-label={isEnabled ? "Disable field" : "Enable field"}
                className={`header-toggle${isEnabled ? " is-enabled" : ""}`}
                icon={isEnabled ? "check-square" : "square"}
                onClick={() =>
                  onChange(
                    items.map((current, currentIndex) =>
                      currentIndex === index
                        ? { ...current, enabled: !current.enabled }
                        : current
                    )
                  )
                }
              />
            </div>
          )
        })}
      </div>
    </div>
  )
}
