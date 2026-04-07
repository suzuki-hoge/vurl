import type { DraftHeader } from "../../features/request/model"
import { IconButton } from "../ui/IconButton"
import { Input } from "../ui/Input"

export function HeaderEditor(props: {
  items: DraftHeader[]
  onChange: (items: DraftHeader[]) => void
  onAdd: () => void
}) {
  const { items, onChange, onAdd } = props

  return (
    <div className="card fill-card">
      <div className="kv-list">
        {items.map((item, index) => {
          const isLocked = item.state === "locked"
          const isToggleable = item.state !== "locked"
          const isEnabled = item.state !== "off"

          return (
            <div
              className={`kv-row kv-row-header${item.state === "off" ? " is-off" : ""}`}
              key={index}
            >
              <Input
                placeholder="key"
                readOnly={isLocked}
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
                readOnly={isLocked}
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
              {isToggleable ? (
                <IconButton
                  aria-label={isEnabled ? "Disable header" : "Enable header"}
                  className={`header-toggle${isEnabled ? " is-enabled" : ""}`}
                  icon={isEnabled ? "check-square" : "square"}
                  onClick={() =>
                    onChange(
                      items.map((current, currentIndex) =>
                        currentIndex === index
                          ? {
                              ...current,
                              state:
                                current.state === "off" ? "editable" : "off"
                            }
                          : current
                      )
                    )
                  }
                />
              ) : (
                <div className="header-state">fixed</div>
              )}
            </div>
          )
        })}
      </div>

      <div className="kv-footer">
        <IconButton aria-label="Add row" icon="plus" onClick={onAdd} />
      </div>
    </div>
  )
}
