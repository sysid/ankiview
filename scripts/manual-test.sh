#!/usr/bin/env bash
# Manual test script for ankiview card update features.
# Uses an isolated Anki collection in ~/xxx/ankiview-test/ — never touches production.
#
# Prerequisites:
#   - Anki must NOT be running
#   - ankiview must be built: make build-fast
#   - Test fixture must exist: make init-env
#
# Usage:
#   ./scripts/manual-test.sh          # run all tests
#   ./scripts/manual-test.sh collect   # run only collect tag-merge tests
#   ./scripts/manual-test.sh tag       # run only tag add/remove/replace tests
#   ./scripts/manual-test.sh edit      # run only edit tests (opens $EDITOR)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
ANKIVIEW="${PROJECT_ROOT}/ankiview/target/debug/ankiview"
TEST_BASE="${HOME}/xxx/ankiview-test"
COLLECTION="${TEST_BASE}/User 1/collection.anki2"
SAMPLE_MD="${PROJECT_ROOT}/ankiview/examples/sample-notes.md"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

passed=0
failed=0
skipped=0

# ── Helpers ──────────────────────────────────────────────────────────────────

die()  { echo -e "${RED}FATAL: $*${NC}" >&2; exit 1; }
info() { echo -e "${CYAN}▶ $*${NC}"; }
pass() { echo -e "  ${GREEN}✓ $*${NC}"; ((passed++)); }
fail() { echo -e "  ${RED}✗ $*${NC}"; ((failed++)); }
skip() { echo -e "  ${YELLOW}⊘ $*${NC}"; ((skipped++)); }
sep()  { echo "────────────────────────────────────────────────────────────"; }

run_ankiview() {
    "$ANKIVIEW" -c "$COLLECTION" "$@"
}

# Reset test environment: fresh copy of fixture collection
reset_env() {
    info "Resetting test environment → ${TEST_BASE}"
    rm -rf "${TEST_BASE}"
    mkdir -p "${TEST_BASE}"
    cp -r "${PROJECT_ROOT}/ankiview/tests/fixtures/test_collection/"* "${TEST_BASE}/"

    # Reset sample-notes.md to original (strip any injected IDs)
    if [[ -f "${SAMPLE_MD}.ori" ]]; then
        cp -f "${SAMPLE_MD}.ori" "$SAMPLE_MD"
    fi
}

# ── Preflight ────────────────────────────────────────────────────────────────

preflight() {
    [[ -f "$ANKIVIEW" ]] || die "Binary not found. Run: make build-fast"

    if pgrep -q -i anki 2>/dev/null; then
        die "Anki is running. Close it first."
    fi

    reset_env
    info "Preflight OK"
    sep
}

# ── Test: collect with tag merge ─────────────────────────────────────────────

test_collect() {
    info "TEST GROUP: collect tag merge"

    # Create a temp markdown file with tags
    local md_file
    md_file=$(mktemp "${TEST_BASE}/test-tags-XXXXXX.md")
    cat > "$md_file" <<'EOF'
---
Deck: Default
Tags: physics quantum

1. What is superposition?
> A particle existing in multiple states simultaneously
---
EOF

    # First collect: creates the note
    local output
    output=$(run_ankiview collect "$md_file" 2>&1) || { fail "collect (create) failed: $output"; return; }
    pass "collect created note"

    # Verify ID was injected
    if grep -q '<!--ID:' "$md_file"; then
        pass "ID injected into markdown"
    else
        fail "ID not injected into markdown"
        return
    fi

    # Extract the note ID
    local note_id
    note_id=$(grep -o '<!--ID:[0-9]*-->' "$md_file" | grep -o '[0-9]*')
    info "  Created note ID: $note_id"

    # Verify tags via view --json
    local json
    json=$(run_ankiview view "$note_id" --json 2>&1)
    if echo "$json" | grep -q '"physics"'; then
        pass "initial tags present (physics)"
    else
        fail "initial tags missing: $json"
    fi

    # Modify tags in markdown (add 'review')
    sed -i '' 's/Tags: physics quantum/Tags: physics quantum review/' "$md_file"

    # Re-collect: should merge tags
    output=$(run_ankiview collect "$md_file" -f 2>&1) || { fail "collect (update) failed: $output"; return; }
    pass "collect updated note"

    # Verify tags merged
    json=$(run_ankiview view "$note_id" --json 2>&1)
    if echo "$json" | grep -q '"review"'; then
        pass "tag 'review' merged after collect"
    else
        fail "tag 'review' NOT merged: $json"
    fi

    # Verify merge-only: remove tag from markdown, re-collect — tag should persist
    sed -i '' 's/Tags: physics quantum review/Tags: physics/' "$md_file"
    output=$(run_ankiview collect "$md_file" -f 2>&1)
    json=$(run_ankiview view "$note_id" --json 2>&1)
    if echo "$json" | grep -q '"review"'; then
        pass "merge-only: 'review' preserved after markdown removal"
    else
        fail "merge-only violated: 'review' was removed"
    fi

    rm -f "$md_file"
    sep
}

# ── Test: tag add / remove ───────────────────────────────────────────────────

test_tag() {
    info "TEST GROUP: tag add / remove / replace"

    # First create a note to work with
    local md_file
    md_file=$(mktemp "${TEST_BASE}/test-tagcli-XXXXXX.md")
    cat > "$md_file" <<'EOF'
---
Deck: Default
Tags: baseline

1. CLI tag test question
> CLI tag test answer
---
EOF

    run_ankiview collect "$md_file" >/dev/null 2>&1
    local note_id
    note_id=$(grep -o '<!--ID:[0-9]*-->' "$md_file" | grep -o '[0-9]*')
    [[ -n "$note_id" ]] || { fail "failed to create test note"; return; }
    info "  Working with note ID: $note_id"

    # tag add
    local output
    output=$(run_ankiview tag add "$note_id" "urgent" 2>&1)
    if echo "$output" | grep -qi "added"; then
        pass "tag add reported success"
    else
        fail "tag add output unexpected: $output"
    fi

    local json
    json=$(run_ankiview view "$note_id" --json 2>&1)
    if echo "$json" | grep -q '"urgent"'; then
        pass "tag 'urgent' visible after add"
    else
        fail "tag 'urgent' not visible: $json"
    fi

    # tag add hierarchical
    output=$(run_ankiview tag add "$note_id" "topic::math::algebra" 2>&1)
    json=$(run_ankiview view "$note_id" --json 2>&1)
    if echo "$json" | grep -q 'topic::math::algebra'; then
        pass "hierarchical tag added"
    else
        fail "hierarchical tag not visible: $json"
    fi

    # tag remove
    output=$(run_ankiview tag remove "$note_id" "urgent" 2>&1)
    json=$(run_ankiview view "$note_id" --json 2>&1)
    if echo "$json" | grep -q '"urgent"'; then
        fail "tag 'urgent' still present after remove"
    else
        pass "tag 'urgent' removed"
    fi

    # tag add on nonexistent note
    output=$(run_ankiview tag add 99999999999 "test" 2>&1) && {
        fail "tag add on nonexistent note should fail"
    } || {
        pass "tag add on nonexistent note → error"
    }

    # tag replace: create two more notes
    local md2 md3
    md2=$(mktemp "${TEST_BASE}/test-replace2-XXXXXX.md")
    md3=$(mktemp "${TEST_BASE}/test-replace3-XXXXXX.md")
    cat > "$md2" <<'EOF'
---
Deck: Default
Tags: old-tag

1. Replace test note 2
> Answer 2
---
EOF
    cat > "$md3" <<'EOF'
---
Deck: Default
Tags: old-tag

1. Replace test note 3
> Answer 3
---
EOF
    run_ankiview collect "$md2" >/dev/null 2>&1
    run_ankiview collect "$md3" >/dev/null 2>&1

    # tag replace (rename)
    output=$(run_ankiview tag replace --old "old-tag" --new "new-tag" 2>&1)
    if echo "$output" | grep -q "2 note"; then
        pass "tag replace renamed on 2 notes"
    else
        fail "tag replace output unexpected: $output"
    fi

    # tag replace: bulk add
    output=$(run_ankiview tag replace --old "" --new "batch-2026" 2>&1)
    if echo "$output" | grep -qi "added.*note"; then
        pass "tag replace bulk-add worked"
    else
        fail "tag replace bulk-add output: $output"
    fi

    # tag replace: bulk remove
    output=$(run_ankiview tag replace --old "batch-2026" --new "" 2>&1)
    if echo "$output" | grep -qi "removed.*note"; then
        pass "tag replace bulk-remove worked"
    else
        fail "tag replace bulk-remove output: $output"
    fi

    # tag replace: both empty → error
    output=$(run_ankiview tag replace --old "" --new "" 2>&1) && {
        fail "tag replace both-empty should fail"
    } || {
        pass "tag replace both-empty → error"
    }

    rm -f "$md_file" "$md2" "$md3"
    sep
}

# ── Test: edit ───────────────────────────────────────────────────────────────

test_edit() {
    info "TEST GROUP: edit (interactive — opens \$EDITOR)"

    # Create a note
    local md_file
    md_file=$(mktemp "${TEST_BASE}/test-edit-XXXXXX.md")
    cat > "$md_file" <<'EOF'
---
Deck: Default
Tags: editable

1. Edit test question
> Edit test answer
---
EOF
    run_ankiview collect "$md_file" >/dev/null 2>&1
    local note_id
    note_id=$(grep -o '<!--ID:[0-9]*-->' "$md_file" | grep -o '[0-9]*')
    [[ -n "$note_id" ]] || { fail "failed to create test note for edit"; return; }

    info "  Note ID: $note_id"
    info "  This test is interactive — it will open your \$EDITOR."
    info "  Change the Back field, save and quit to verify update."
    info "  Or quit without saving to test no-change detection."
    echo ""

    run_ankiview edit "$note_id"
    local exit_code=$?

    if [[ $exit_code -eq 0 ]]; then
        pass "edit command completed (exit 0)"
        info "  Verify changes with: ankiview -c '$COLLECTION' view $note_id --json"
    else
        fail "edit command failed (exit $exit_code)"
    fi

    # Test edit on nonexistent note
    output=$(run_ankiview edit 99999999999 2>&1) && {
        fail "edit nonexistent note should fail"
    } || {
        pass "edit nonexistent note → error"
    }

    rm -f "$md_file"
    sep
}

# ── Main ─────────────────────────────────────────────────────────────────────

main() {
    local filter="${1:-all}"

    echo ""
    echo "╔══════════════════════════════════════════════════════════════╗"
    echo "║        ankiview manual test — isolated environment          ║"
    echo "║        Collection: ~/xxx/ankiview-test/                     ║"
    echo "╚══════════════════════════════════════════════════════════════╝"
    echo ""

    preflight

    case "$filter" in
        all)
            test_collect
            test_tag
            test_edit
            ;;
        collect) test_collect ;;
        tag)     test_tag ;;
        edit)    test_edit ;;
        *)       die "Unknown filter: $filter (use: all, collect, tag, edit)" ;;
    esac

    # Summary
    echo ""
    echo "╔══════════════════════════════════════╗"
    printf "║  ${GREEN}PASSED: %-4d${NC}  ${RED}FAILED: %-4d${NC}  ${YELLOW}SKIP: %-3d${NC} ║\n" "$passed" "$failed" "$skipped"
    echo "╚══════════════════════════════════════╝"
    echo ""

    if [[ $failed -gt 0 ]]; then
        exit 1
    fi
}

main "$@"
