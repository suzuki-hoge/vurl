export function formatMethodLabel(method: string): string {
  if (method.toUpperCase() === "DELETE") {
    return "DEL"
  }

  return method
}
