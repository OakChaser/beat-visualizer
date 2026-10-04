#!/usr/bin/env bash
# Download the Beat This! ONNX models into models/.
#
# The mel front end and the small beat model are committed in the upstream
# beat-this-rs repository; the full-accuracy FP32 beat model is a release
# asset of the `model-large` tag. Nothing here needs Python or torch.
#
#   scripts/fetch-models.sh          # mel + small beat model (~10 MB)
#   scripts/fetch-models.sh --full   # also the full beat model (~83 MB)
set -euo pipefail

REPO="danigb/beat-this-rs"
RAW="https://raw.githubusercontent.com/${REPO}/main/models"
RELEASE="https://github.com/${REPO}/releases/download/model-large"

MEL_MODEL="mel_spectrogram.onnx"
SMALL_MODEL="beat_this_small.onnx"
FULL_MODEL="beat_this.onnx"

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
DEST="${SCRIPT_DIR}/../models"
mkdir -p "${DEST}"

fetch() {
    local url="$1" out="$2"
    if [[ -f "${out}" ]]; then
        echo "==> $(basename "${out}") already present, skipping"
        return
    fi
    echo "==> downloading $(basename "${out}")"
    curl -fL --retry 3 --progress-bar -o "${out}" "${url}"
}

verify() {
    local file="$1"
    if [[ -f "${file}.sha256" ]]; then
        echo "==> verifying $(basename "${file}")"
        if command -v sha256sum >/dev/null 2>&1; then
            (cd "$(dirname "${file}")" && sha256sum -c "$(basename "${file}").sha256")
        else
            (cd "$(dirname "${file}")" && shasum -a 256 -c "$(basename "${file}").sha256")
        fi
        rm -f "${file}.sha256"
    fi
}

fetch "${RAW}/${MEL_MODEL}" "${DEST}/${MEL_MODEL}"
fetch "${RAW}/${SMALL_MODEL}" "${DEST}/${SMALL_MODEL}"

if [[ "${1:-}" == "--full" ]]; then
    fetch "${RELEASE}/${FULL_MODEL}" "${DEST}/${FULL_MODEL}"
    fetch "${RELEASE}/${FULL_MODEL}.sha256" "${DEST}/${FULL_MODEL}.sha256"
    verify "${DEST}/${FULL_MODEL}"
fi

echo
echo "Models in ${DEST}:"
ls -lh "${DEST}"
echo
echo "The app picks up the full beat model when it is present; otherwise it uses the small one."
echo "Override the directory with BEAT_MODEL_DIR=/path/to/models."