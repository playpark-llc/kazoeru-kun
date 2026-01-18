#!/bin/sh
set -eu

dir="${1:-}"
if [ -z "$dir" ]; then
  echo "usage: $0 <dir>" >&2
  exit 2
fi

# On Docker Desktop/OrbStack bind mounts, container uid/gid often map to the host user.
# If the directory was created by another uid (e.g. a prior root-owned build), it may be
# non-writable and will cause tools like trunk to fail. Removing it is safe because it is
# a build artifact directory.
mkdir -p "$dir"

# Sanity check (round 1): if we can't create a file, try to recreate the directory.
probe="$dir/.write-probe"
if ! : >"$probe" 2>/dev/null; then
  echo "[ensure-writable-dir] '$dir' is not writable; removing and recreating it" >&2
  rm -rf "$dir"
  mkdir -p "$dir"
fi

# Sanity check (round 2): fail early with a clear message if it's still not writable.
if ! : >"$probe" 2>/dev/null; then
  echo "[ensure-writable-dir] '$dir' is still not writable (check ownership/permissions on the host)" >&2
  ls -ld "$dir" >&2 || true
  exit 1
fi
rm -f "$probe"
