import { Button } from "./Button"

export function TabButton(props: {
  active: boolean
  label: string
  onClick: () => void
}) {
  const { active, label, onClick } = props

  return (
    <Button
      className={`tab-button${active ? " is-active" : ""}`}
      onClick={onClick}
      type="button"
    >
      {label}
    </Button>
  )
}
