#!/bin/sh

set -eu

tracked_files=$(git ls-files)
staged_files=$(git diff --cached --name-only --diff-filter=ACMR || true)

files_to_check=$(printf '%s\n%s\n' "$tracked_files" "$staged_files" | sed '/^$/d' | sort -u)

forbidden_pattern='(^|/)\.DS_Store$|(^|/)node_modules/|(^|/)target/|(^|/)site/|(^|/)dist/'

violations=$(printf '%s\n' "$files_to_check" | grep -E "$forbidden_pattern" || true)

if [ -n "$violations" ]; then
    echo "❌ Repository hygiene check failed. Forbidden tracked or staged paths:"
    printf '%s\n' "$violations"
    echo ""
    echo "Allowed publish assets should be generated during build/publish, not committed."
    exit 1
fi

echo "✅ Repository hygiene check passed!"
