function vurl() {
  local root="$HOME/.vurl"
  local script_file="${${(%):-%x}:A}"
  local repo_root="${script_file:h:h}"
  local backend_bin="${repo_root}/src/backend/target/release/vurl-backend"
  local frontend_url="http://127.0.0.1:1357"

  case "${1:-}" in
    -l)
      cd "${root}/logs" || return 1
      return 0
      ;;
    -y)
      cd "${root}/defs" || return 1
      return 0
      ;;
    "")
      if command -v open >/dev/null 2>&1; then
        open "${frontend_url}" >/dev/null 2>&1 &
      else
        print "${frontend_url}"
      fi
      "${backend_bin}"
      return $?
      ;;
    *)
      print "usage: vurl | vurl -l | vurl -y" >&2
      return 1
      ;;
  esac
}
