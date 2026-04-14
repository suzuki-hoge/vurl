typeset -g _VURL_SCRIPT_FILE="${${(%):-%N}:A}"
typeset -g _VURL_REPO_ROOT="${_VURL_SCRIPT_FILE:h:h}"

function _vurl_is_pid_alive() {
  local pid="${1:-}"
  [[ -n "${pid}" ]] || return 1
  kill -0 "${pid}" >/dev/null 2>&1
}

function _vurl_read_file() {
  local file_path="${1:-}"
  [[ -f "${file_path}" ]] || return 1
  IFS= read -r REPLY < "${file_path}" || return 1
}

function _vurl_cleanup_runtime() {
  local pid_file="${1}"
  rm -f "${pid_file}"
}

function _vurl_is_running() {
  local pid_file="${1}"
  if ! _vurl_read_file "${pid_file}"; then
    return 1
  fi

  _vurl_is_pid_alive "${REPLY}"
}

function _vurl_start() {
  local runtime_dir="${1}"
  local backend_bin="${2}"
  local frontend_url="${3}"
  local pid_file="${4}"
  local backend_log_file="${5}"

  if _vurl_is_running "${pid_file}"; then
    print -r -- "vurl is already running."
    return 0
  fi

  _vurl_cleanup_runtime "${pid_file}"
  mkdir -p "${runtime_dir}" || return 1

  print -r -- "Booting vurl..."
  "${backend_bin}" >> "${backend_log_file}" 2>&1 &
  local backend_pid=$!
  print -r -- "${backend_pid}" > "${pid_file}"

  sleep 0.2
  if ! _vurl_is_pid_alive "${backend_pid}"; then
    _vurl_cleanup_runtime "${pid_file}"
    print "failed to start vurl-backend" >&2
    return 1
  fi

  if command -v open >/dev/null 2>&1; then
    open "${frontend_url}"
  fi

  return 0
}

function _vurl_stop() {
  local pid_file="${1}"

  if ! _vurl_is_running "${pid_file}"; then
    _vurl_cleanup_runtime "${pid_file}"
    print -r -- "vurl process not found."
    return 0
  fi

  local backend_pid
  _vurl_read_file "${pid_file}" || return 1
  backend_pid="${REPLY}"

  print -r -- "Shutting down vurl."
  kill "${backend_pid}" >/dev/null 2>&1 || true

  local retry
  for retry in 1 2 3 4 5 6 7 8 9 10; do
    if ! _vurl_is_pid_alive "${backend_pid}"; then
      break
    fi
    sleep 0.1
  done

  if _vurl_is_pid_alive "${backend_pid}"; then
    kill -9 "${backend_pid}" >/dev/null 2>&1 || true
    wait "${backend_pid}" 2>/dev/null || true
  fi

  _vurl_cleanup_runtime "${pid_file}"
}

function _vurl_print_help() {
  local out_fd="${1:-1}"
  cat >&${out_fd} <<'EOF'
Usage:
  vurl
  vurl -d | --down
  vurl -l | --log-dir [project]
  vurl -e | --edit
  vurl -h | --help
EOF
}

function vurl() {
  emulate -L zsh
  unsetopt monitor

  local root="${HOME}/.vurl"
  local repo_root="${_VURL_REPO_ROOT}"
  local runtime_dir="${root}/run"
  local backend_bin="${repo_root}/src/backend/target/release/vurl-backend"
  local frontend_url="http://127.0.0.1:1357"
  local pid_file="${runtime_dir}/backend.pid"
  local backend_log_file="${runtime_dir}/backend.log"

  if [[ ! -x "${backend_bin}" ]]; then
    print "missing backend binary: ${backend_bin}" >&2
    return 1
  fi

  case "${1:-}" in
    "")
      _vurl_start \
        "${runtime_dir}" \
        "${backend_bin}" \
        "${frontend_url}" \
        "${pid_file}" \
        "${backend_log_file}"
      return $?
      ;;
    -d|--down)
      _vurl_stop "${pid_file}"
      return $?
      ;;
    -l|--log-dir)
      if [[ -n "${2:-}" ]]; then
        cd "${root}/logs/${2}" || return 1
        return 0
      fi

      cd "${root}/logs" || return 1
      return 0
      ;;
    -e|--edit)
      if ! command -v open >/dev/null 2>&1; then
        print "open command is not available" >&2
        return 1
      fi

      open -n -a "IntelliJ IDEA 2.app" --args "${root}"
      return $?
      ;;
    -h|--help)
      _vurl_print_help 1
      return 0
      ;;
    *)
      print -r -- "unknown option: ${1}" >&2
      _vurl_print_help 2
      return 1
      ;;
  esac
}
