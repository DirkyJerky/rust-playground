#!/bin/bash
#####
# Geoff Yoerger, Oct 2026
#####
#
#  Usage: import.bash <src repo> [local path]
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

copy_git_repo() {
    set -e
    local source="${1:-}"

    if [[ -z "${source}" ]]; then
        die "Error: copy_git_repo not called with source repo"
    fi

    # Make the temporary repo
    TMP_REPO=$(mktemp -d)

    git clone "${source}" "${TMP_REPO}"

    echo "${TMP_REPO}"
}

rewrite_git_history() {
    set -e
    local tmp_repo="${1:-}"
    local target_path="${2:-}"

    if [[ -z "${tmp_repo}" ]]; then
        die "Error: rewrite_git_history is missing tmp repo"
    fi

    if [[ -z "${target_path}" ]]; then
        die "Error: rewrite_git_history is missing target path"
    fi

    local target_name=$(basename "${target_path}")

    cd "${tmp_repo}"
    ls

    local py_rename='return re.sub(rb"^([a-zA-Z0-9_-]+)(\([^)]+\))?:", rb"\1('"${target_name}"'):", message if re.match(rb"^[a-zA-Z0-9_-]+(\([^)]+\))?:", message) else b"feat: " + message)'
    git filter-repo --to-subdirectory-filter "${target_path}" --message-callback "${py_rename}"
}

merge_tmp_repo() {
    set -e
    local tmp_repo="${1:-}"
    local target_path="${2:-}"
    local dest_repo="${3:-}"

    if [[ -z "${tmp_repo}" ]]; then
        die "Error: merge_tmp_repo is missing tmp repo"
    fi

    if [[ -z "${target_path}" ]]; then
        die "Error: merge_tmp_repo is missing target path"
    fi

    local target_name=$(basename "${target_path}")
    local branch
    branch=$(git -C "${tmp_repo}" branch --show-current 2>/dev/null || true)
    if [[ -z "${branch}" ]]; then
        branch=$(git -C "${tmp_repo}" symbolic-ref --short HEAD 2>/dev/null || echo "master")
    fi

    cd "${DIR}"

    git remote remove tmp-remote 2>/dev/null || true
    git remote add tmp-remote "${tmp_repo}"
    git fetch tmp-remote

    git merge "tmp-remote/${branch}" --allow-unrelated-histories -m "Merge ${target_name}"

    git remote remove tmp-remote
}

main() {
    local source="${1:-}"
    local dest="${2:-}"

    if ! command -v git-filter-repo; then
        die "Error: git-filter-repo not installed. Please install to continue"
    fi

    if [[ -z "${source}" ]]; then
        die "Usage: $0 <src repo> [local path]"
    fi

    if [[ -z "${dest}" ]]; then
        dest=$(basename "${source}")
    fi

    if [[ ! -d "${source}" ]]; then
        die "Error: source repo is not a folder: ${source}"
    fi

    if [[ ! -d "${source}/.git" ]]; then
        die "Error: source repo is not a git repository: ${source}"
    fi

    mkdir -p "${DIR}/${dest}"

    TMP_REPO=$(copy_git_repo "${source}")
    trap 'rm -rf "$TMP_REPO"' exit

    rewrite_git_history "${TMP_REPO}" "${dest}"
    merge_tmp_repo "${TMP_REPO}" "${dest}"
}

# Run main if script is invoked directly
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi
