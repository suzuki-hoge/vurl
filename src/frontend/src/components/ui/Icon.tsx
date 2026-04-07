import {
  FiCheckSquare,
  FiPlus,
  FiSearch,
  FiSquare,
  FiTrash2
} from "react-icons/fi"

const icons = {
  "check-square": FiCheckSquare,
  plus: FiPlus,
  search: FiSearch,
  square: FiSquare,
  trash: FiTrash2
} as const

export type IconName = keyof typeof icons

export function Icon(props: { name: IconName }) {
  const Component = icons[props.name]
  return <Component />
}
