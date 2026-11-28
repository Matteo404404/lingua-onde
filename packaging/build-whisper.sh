#!/usr/bin/env bash
# Builds whisper.cpp's command-line program (whisper-cli) for this machine and
# packs it the way src/speech.rs downloads it: the program and whisper.cpp's
# license at the top of one archive (.zip on Windows, .tar.gz elsewhere).
# CPU only (no GPU, BLAS or OpenMP); its libraries linked in, and on Linux
# libstdc++ too, so the program needs nothing beyond the system's C library.
# x86_64 builds use AVX2 (processors from 2013 on).
# Used by .github/workflows/release.yml and check-platforms.yml.
#
# usage: packaging/build-whisper.sh <archive>
set -euo pipefail

TAG=v1.9.5
out="$(pwd)/$1"
work="${RUNNER_TEMP:-${TMPDIR:-/tmp}}/whisper.cpp"
rm -rf "$work"
git clone --quiet --depth 1 --branch "$TAG" https://github.com/ggml-org/whisper.cpp "$work"

opts=(
  -DCMAKE_BUILD_TYPE=Release -DBUILD_SHARED_LIBS=OFF
  -DGGML_NATIVE=OFF -DGGML_OPENMP=OFF -DGGML_BLAS=OFF -DGGML_METAL=OFF
  -DGGML_ACCELERATE=OFF -DGGML_CCACHE=OFF
  -DWHISPER_BUILD_TESTS=OFF -DWHISPER_BUILD_SERVER=OFF -DWHISPER_SDL2=OFF -DWHISPER_CURL=OFF
)
case "$(uname -s)" in
  Linux) opts+=("-DCMAKE_EXE_LINKER_FLAGS=-static-libstdc++ -static-libgcc") ;;
  Darwin) opts+=(-DCMAKE_OSX_DEPLOYMENT_TARGET=11.0) ;;
  # the static C runtime, as the app uses (whisper.cpp asks for CMake 3.5,
  # so the policy that reads the runtime setting is switched on here)
  MINGW* | MSYS*) opts+=(-DCMAKE_POLICY_DEFAULT_CMP0091=NEW -DCMAKE_MSVC_RUNTIME_LIBRARY=MultiThreaded) ;;
esac
cmake -S "$work" -B "$work/build" "${opts[@]}"
cmake --build "$work/build" --config Release --target whisper-cli -j 4

stage="$work/stage"
mkdir -p "$stage"
bin=$(find "$work/build/bin" -type f \( -name whisper-cli -o -name whisper-cli.exe \) | head -n 1)
cp "$bin" "$stage/"
cp "$work/LICENSE" "$stage/whisper.cpp-LICENSE.txt"
if [ "$(uname -s)" = Darwin ]; then
  # an ad-hoc signature, as the app has
  codesign --force --sign - "$stage/whisper-cli"
  otool -L "$stage/whisper-cli"
fi
# it starts: no library missing
"$stage/$(basename "$bin")" -h >/dev/null 2>&1

rm -f "$out"
case "$out" in
  *.zip) (cd "$stage" && 7z a -tzip "$out" ./* >/dev/null) ;;
  *) tar -czf "$out" -C "$stage" "$(basename "$bin")" whisper.cpp-LICENSE.txt ;;
esac
ls -l "$out"
