#!/usr/bin/env bash
# Build the TUNAS Windows release: stage files, make the installer, zip it.
set -euo pipefail
VER="${1:?version}"
ROOT=/home/claude/tunas
B=/home/claude/build
STAGE=$B/stage
OUT=$ROOT/release
rm -rf "$STAGE"; mkdir -p "$STAGE"/{resources,engine/$VER/cpu,engine/$VER/vulkan} "$OUT"

EXE=$ROOT/app/src-tauri/target/x86_64-pc-windows-gnu/release/tunas.exe
cp "$EXE" "$STAGE/TUNAS.exe"
# WebView2Loader.dll is needed by the GNU build if it was produced.
WV=$(find $ROOT/app/src-tauri/target/x86_64-pc-windows-gnu/release -name WebView2Loader.dll | head -1 || true)
[ -n "$WV" ] && cp "$WV" "$STAGE/"
cp $ROOT/app/src-tauri/icons/icon.ico "$STAGE/TUNAS.ico"
cp $ROOT/app/src-tauri/resources/catalog.json "$STAGE/resources/"

RT_DIRS="/usr/lib/gcc/x86_64-w64-mingw32/13-posix /usr/x86_64-w64-mingw32/lib"
copy_rt() {
  for dll in libgcc_s_seh-1.dll libstdc++-6.dll libwinpthread-1.dll libgomp-1.dll; do
    for d in $RT_DIRS; do [ -f "$d/$dll" ] && { cp "$d/$dll" "$1/"; break; }; done
  done
}
for kind in cpu vulkan; do
  D=$STAGE/engine/$VER/$kind
  cp $B/llama-cpu/bin/llama-server.exe $B/llama-cpu/bin/*.dll "$D/"
  copy_rt "$D"
done
cp $B/llama-vk/bin/ggml-vulkan.dll "$STAGE/engine/$VER/vulkan/"
find "$STAGE/engine" -name "*.dll" -exec x86_64-w64-mingw32-strip --strip-unneeded {} \;
x86_64-w64-mingw32-strip --strip-unneeded "$STAGE"/engine/*/*/llama-server.exe

cat > "$STAGE/THIRD-PARTY-NOTICES.txt" <<'EOF'
TUNAS includes the following open-source components:

llama.cpp / ggml - MIT License - Copyright (c) 2023-2026 The ggml authors - https://github.com/ggml-org/llama.cpp
Tauri - MIT or Apache-2.0 - https://tauri.app
Svelte - MIT - https://svelte.dev
GCC runtime libraries (libgcc, libstdc++, libgomp) - GPL with the GCC Runtime Library Exception
mingw-w64 winpthreads - MIT / BSD-style licence
Plus Jakarta Sans font - SIL Open Font License 1.1

AI models are not part of this installer. Each model keeps its own licence, shown in TUNAS on the Models page.

S.A.L.T CI and its components are proprietary technology of S.A.L.T. CI TECHNOLOGY, distributed and commercialised by Pemangkin Teknologi.
EOF
cp /home/claude/ggml-org/llama.cpp/LICENSE "$STAGE/LICENSE-llama.cpp.txt"

SETUP=$OUT/TUNAS_Setup_${VER}.exe
makensis -V2 -DVERSION="$VER" -DSTAGE="$STAGE" -DOUTFILE="$SETUP" $ROOT/installer/tunas.nsi
( cd $OUT && rm -f TUNAS_${VER}.zip && cp $ROOT/docs/TUNAS_Release_Notes.md . && zip -q -j TUNAS_${VER}.zip TUNAS_Setup_${VER}.exe TUNAS_Release_Notes.md && rm TUNAS_Release_Notes.md )
ls -la $OUT
sha256sum $OUT/TUNAS_${VER}.zip
