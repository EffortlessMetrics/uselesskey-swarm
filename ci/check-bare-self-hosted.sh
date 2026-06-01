#!/usr/bin/env bash
set -euo pipefail

bad=0
workflow_dir="${1:-.github/workflows}"

echo "Checking workflow runner and action-ref hygiene in ${workflow_dir}..."

if [ ! -d "${workflow_dir}" ]; then
  echo "Workflow directory not found: ${workflow_dir}" >&2
  exit 1
fi

workflow_grep() {
  grep -RInE --include='*.yml' --include='*.yaml' "$1" "${workflow_dir}"
}

if workflow_grep 'runs-on:[[:space:]]*\[[^]]*self-hosted[^]]*linux[^]]*x64[^]]*\]'; then
  echo "Bare inline self-hosted/linux/x64 runs-on is forbidden." >&2
  bad=1
fi

while IFS=: read -r file line text; do
  echo "$file:$line: mutable action ref is forbidden: $text" >&2
  bad=1
done < <(workflow_grep '^[[:space:]]*(-[[:space:]]*)?uses:[[:space:]]*['"'"'"]?[^'"'"'"[:space:]#]+@(main|master)['"'"'"]?([[:space:]#]|$)' || true)

while IFS=: read -r file line _; do
  window="$(sed -n "${line},$((line+16))p" "$file")"

  if printf '%s\n' "$window" | grep -Eq '^[[:space:]]*-[[:space:]]*linux[[:space:]]*$' &&
     printf '%s\n' "$window" | grep -Eq '^[[:space:]]*-[[:space:]]*x64[[:space:]]*$' &&
     ! printf '%s\n' "$window" | grep -Eq 'group:[[:space:]]*em-ci-' &&
     ! printf '%s\n' "$window" | grep -Eq '^[[:space:]]*-[[:space:]]*(em-ci|ci-nano|policy-nano|workflow-nano|rust-tiny|rust-medium|rust-large|rust-16gb|cx23|cx33|cx43|cx53|cpx42)[[:space:]]*$'; then
    echo "$file:$line: bare self-hosted block lacks group/capacity labels" >&2
    bad=1
  fi
done < <(workflow_grep '^[[:space:]]*-[[:space:]]*self-hosted[[:space:]]*$' || true)

exit "$bad"
