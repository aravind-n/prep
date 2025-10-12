#!/bin/bash
set -euo pipefail

# Usage: sh scripts/ci/github_macos_build.sh <arch> <artifact_dir>
#   arch: x86_64 | aarch64
#   artifact_dir: macos-x86 | macos-aarch64

ARCH="${1:?arch required (x86_64|aarch64)}"
OUT_DIR="${2:?artifact dir required (macos-x86|macos-aarch64)}"

: "${GH_TOKEN:?set GH_TOKEN}"         # GitHub PAT: Actions read/write, Contents read
: "${GH_REPO:?set GH_REPO}"
: "${GL_REPO_URL:?set GL_REPO_URL}"
GH_WORKFLOW_FILE="${GH_WORKFLOW_FILE:-build.yml}"
GH_WORKFLOW_REF="${GH_WORKFLOW_REF:-master}"
GH_WEB_BASE="${GH_WEB_BASE:-https://github.com}"

# Use MR source branch if present, otherwise current ref
REF="${CI_MERGE_REQUEST_SOURCE_BRANCH_NAME:-${CI_COMMIT_REF_NAME}}"

# Unique correlation id so we can deterministically find the run
CORR="gl-${CI_PIPELINE_ID:-unknown}-${CI_JOB_ID:-unknown}-${ARCH}"

echo "Dispatching GitHub workflow: repo=${GH_REPO} workflow=${GH_WORKFLOW_FILE} ref=${GH_WORKFLOW_REF} arch=${ARCH} corr=${CORR}"

# Trigger workflow_dispatch
curl -sS -X POST \
  -H "Authorization: Bearer ${GH_TOKEN}" \
  -H "Accept: application/vnd.github+json" \
  "https://api.github.com/repos/${GH_REPO}/actions/workflows/${GH_WORKFLOW_FILE}/dispatches" \
  -d @- <<EOF
{
  "ref": "${GH_WORKFLOW_REF}",
  "inputs": {
    "arch": "${ARCH}",
    "git_url": "${GL_REPO_URL}",
    "ref": "${REF}",
    "sha": "${CI_COMMIT_SHA}",
    "bin_name": "${CI_PROJECT_NAME}",
    "corr": "${CORR}"
  }
}
EOF

# Find the matching run by correlation id (requires GH workflow to set run-name to inputs.corr)
echo "Polling for workflow run with correlation id: ${CORR}"
RUN_ID=""
for i in $(seq 1 60); do
  RUN_ID="$(curl -sS -H "Authorization: Bearer ${GH_TOKEN}" \
    "https://api.github.com/repos/${GH_REPO}/actions/workflows/${GH_WORKFLOW_FILE}/runs?event=workflow_dispatch&per_page=50" \
    | jq -r --arg C "$CORR" '.workflow_runs[] | select(.display_title==$C) | .id' | head -n1)"
  if [[ -n "$RUN_ID" ]]; then break; fi
  sleep 2
done
[[ -n "$RUN_ID" ]] || { echo "No matching workflow run found for corr=${CORR}"; exit 1; }
echo "Run ID: $RUN_ID"
echo "GitHub Actions run: ${GH_WEB_BASE}/${GH_REPO}/actions/runs/${RUN_ID}"

# Wait for completion
for i in $(seq 1 180); do
  RESP="$(curl -sS -H "Authorization: Bearer ${GH_TOKEN}" \
    "https://api.github.com/repos/${GH_REPO}/actions/runs/${RUN_ID}")"
  STATUS="$(echo "$RESP" | jq -r '.status')"
  CONCLUSION="$(echo "$RESP" | jq -r '.conclusion')"
  echo "Status: $STATUS / $CONCLUSION"
  if [[ "$STATUS" == "completed" ]]; then
    [[ "$CONCLUSION" == "success" ]] || { echo "GitHub run failed"; exit 1; }
    break
  fi
  sleep 5
done

# Download the artifact named exactly as the arch we requested
ARTS_JSON="$(curl -sS -H "Authorization: Bearer ${GH_TOKEN}" \
  "https://api.github.com/repos/${GH_REPO}/actions/runs/${RUN_ID}/artifacts?per_page=50")"

DL_URL="$(echo "$ARTS_JSON" | jq -r --arg N "$ARCH" '.artifacts[] | select(.name==$N) | .archive_download_url' | head -n1)"
[[ -n "$DL_URL" && "$DL_URL" != "null" ]] || { echo "No artifact named '$ARCH' found"; echo "$ARTS_JSON" | jq; exit 1; }

mkdir -p "out/${OUT_DIR}"
ZIP="/tmp/${OUT_DIR}-${CORR}.zip"
curl -sSL -H "Authorization: Bearer ${GH_TOKEN}" "$DL_URL" -o "$ZIP"
unzip -o "$ZIP" -d "out/${OUT_DIR}" >/dev/null
rm -f "$ZIP"

echo "Artifacts in out/${OUT_DIR}:"
ls -al "out/${OUT_DIR}"
