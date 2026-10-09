#!/bin/bash
#####
# Geoff Yoerger, Oct 2026
#####
#
#  Usage: import.bash <src repo> <local path>
#
#  Import a an external git repository to this one, using `git-filter-repo`
#  This script will perform the following changes:
#  - Changing the path
#  - Renaming any conventional commits to include the destination folder name as the "scope".
#    - i.e. "feat: Create problem definition" becomes "feat(hello-world): Create problem definition"
#    - Also: Adds "feat(...): " if no conventional commit is present
#
#####
set -eou pipefail
DIR=$(cd "$(dirname "$(readlink -f "${BASH_SOURCE[0]}")")" &>/dev/null && pwd -P)
msg() { [[ -n "${1:-}" ]] && echo "${1}" >&2; }
die() { msg "${1:-}"; exit 1; }
#####

main() {
    msg "Hello World!"
}

# Run main if script is invoked directly
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi
