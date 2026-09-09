#! /bin/bash
set -uo pipefail

function log {
    echo "[${MODULE}] $*"
}

function print_banner {
    local title=" $1 " width=50
    local pad=$(( (width - ${#title}) / 2 ))
    (( pad < 0 )) && pad=0
    local bar
    printf -v bar '%*s' "$pad" ''
    bar=${bar// /=}
    printf '%s%s%s\n' "$bar" "$title" "$bar"
}

function get-base-commit-id {
    git log -1 --format='%H' origin/aspeed-dev-2.1-rt
}

function ci-check-commit-list {
    base_commit=$(get-base-commit-id)
    git log --reverse --format='%H' "${base_commit}..HEAD"
}

function check-format {
    echo -ne "[${MODULE}] check caliptra-mcu-sw coding style..."

    if cargo fmt --check -q; then
        echo "done"
    else
        echo "failed"
        return 1
    fi
}

function check-commit-title {
    commit_id=$1
    commit_title=$(git log -1 --format='%s' $commit_id)
    allowed_prefixes=(
        "ast1040: drivers: "
        "ast1040: capsules: "
        "ast1040: app: "
        "ast1040: "
        "ast2705: drivers: "
        "ast2705: capsules: "
        "ast2705: app: "
        "ast2705: "
        "drivers: "
        "capsules: "
        "components: "
        "veer: "
        "app: "
        "common: "
        "registers: "
        "builder: "
        "tools: "
    )

    for p in "${allowed_prefixes[@]}"; do
        [[ "$commit_title" == "$p"* ]] && {
            log "$commit_id ...ok"
            return 0
        }
    done

    log "$commit_id $commit_title invalid."
    return 1
}

function check-commit-body {
    commit_id=$1
    commit_message=$(git log -1 --format='%B' "$commit_id")

    if grep -qi 'ast2755' <<<"$commit_message"; then
        log "$commit_id title/body contains 'ast2755'."
        return 1
    fi

    if git show "$commit_id" | grep '^+' | grep -v '^+++' | grep -qi 'ast2755'; then
        log "$commit_id source code diff contains 'ast2755'."
        return 1
    fi

    log "$commit_id ...ok"

    return 0
}

function check-commit-path {
    commit_id=$1
    change_file_list=$(git show --name-only --format='' "$commit_id")

    if grep -qE '(^|/)(rom)(/|$)' <<<"$change_file_list"; then
        log "$commit_id changes a file under rom/:"
        log "$(grep -E '(^|/)(rom)(/|$)' <<<"$change_file_list")"
        return 1
    fi

    log "$commit_id ...ok"

    return 0
}

function check-commit-scope {
    commit_id=$1
    change_file_list=$(git show --name-only --format='' "$commit_id")

    if grep -qE '^registers/' <<<"$change_file_list" \
        && grep -qvE '^registers/' <<<"$change_file_list"; then
        log "$commit_id mixes registers/ changes with other paths:"
        log "$(grep -vE '^registers/' <<<"$change_file_list")"
        return 1
    fi

    if grep -qE '^common/' <<<"$change_file_list" \
        && grep -qvE '^common/' <<<"$change_file_list"; then
        log "$commit_id mixes common/ changes with other paths:"
        log "$(grep -vE '^common/' <<<"$change_file_list")"
        return 1
    fi

    log "$commit_id ...ok"

    return 0
}

function check-item {
    check_cb=$1

    print_banner "$check_cb"

    "$check_cb"

    if [[ $? -eq 0 ]]; then
        log "$check_cb pass."
        return 0
    else
        log "$check_cb fail."
        return 1
    fi
}

function check-multi-item {
    check_cb=$1
    commit_list="$2"
    rc=0

    print_banner "$check_cb"

    for id in $commit_list; do
        "$check_cb" "$id" || { rc=1; break; }
    done

    if [[ $rc -eq 0 ]]; then
        log "$check_cb pass."
        return 0
    else
        log "$check_cb fail."
        return 1
    fi
}

##########################################################
# Global Variables
##########################################################
MODULE="SS-PRE-CI"
DIR="$(dirname "$(realpath "$0")")"
CPTRA_MCU_DIR=$DIR/../

##########################################################
# Main Flow
##########################################################
result=0

pushd .
cd $CPTRA_MCU_DIR

# Get all checked commit list
commit_list=$(ci-check-commit-list)

# Check coding style
check-item check-format || result=1

# Check commit message title
check-multi-item check-commit-title "$commit_list" || result=1

# Check commit message body
check-multi-item check-commit-body "$commit_list" || result=1

# Check commit path
check-multi-item check-commit-path "$commit_list" || result=1

# Check commit scope: registers/common changes must not mix with other paths
check-multi-item check-commit-scope "$commit_list" || result=1

popd

exit $result
