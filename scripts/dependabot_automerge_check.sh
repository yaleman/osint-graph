#!/usr/bin/env bash

set -euo pipefail

readonly DEPENDABOT_LOGIN="app/dependabot"

if [ "$#" -ne 1 ]; then
    echo "Usage: $0 <PR URL>" >&2
    exit 2
fi

readonly PR_URL="$1"
PR_DATA="$(gh pr view "$PR_URL" --json author,isDraft,reviewDecision,state)"

AUTHOR="$(jq -er '.author.login' <<<"${PR_DATA}")"
IS_DRAFT="$(jq -er '.isDraft | tostring' <<<"${PR_DATA}")"
REVIEW_DECISION="$(jq -er '.reviewDecision // ""' <<<"${PR_DATA}")"
STATE="$(jq -er '.state' <<<"${PR_DATA}")"

if [ "${AUTHOR}" != "${DEPENDABOT_LOGIN}" ]; then
    echo "Refusing to approve PR from ${AUTHOR}; expected ${DEPENDABOT_LOGIN}" >&2
    exit 1
fi

if [ "${STATE}" != "OPEN" ]; then
    echo "Refusing to approve PR in ${STATE} state" >&2
    exit 1
fi

if [ "${IS_DRAFT}" = "true" ]; then
    echo "Refusing to approve a draft PR" >&2
    exit 1
fi

case "${REVIEW_DECISION}" in
    APPROVED)
        echo "Dependabot PR is already approved"
        exit 0
        ;;
    CHANGES_REQUESTED)
        echo "Refusing to approve a PR with requested changes" >&2
        exit 1
        ;;
    "" | REVIEW_REQUIRED)
        ;;
    *)
        echo "Unknown review decision: ${REVIEW_DECISION}" >&2
        exit 1
        ;;
esac

# Required checks remain enforced by branch protection before auto-merge completes.
echo "Approving Dependabot PR ${PR_URL}"
gh pr review --approve "$PR_URL"
