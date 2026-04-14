import {
  FiChevronDown,
  FiCheckSquare,
  FiPlus,
  FiSearch,
  FiSquare,
  FiTrash2
} from "react-icons/fi"
import { RxReload } from "react-icons/rx"

const icons = {
  "chevron-down": FiChevronDown,
  "check-square": FiCheckSquare,
  plus: FiPlus,
  reload: RxReload,
  search: FiSearch,
  square: FiSquare,
  trash: FiTrash2
} as const

export type IconName = keyof typeof icons

export function Icon(props: { name: IconName }) {
  const Component = icons[props.name]
  return <Component />
}
