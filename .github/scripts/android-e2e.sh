#!/usr/bin/env bash
# TUNAS on an Android emulator, the way a person would use it the first time:
# setup -> download the recommended model -> ask a question -> wait for the answer.
# Screenshots, the screen's text and the log go to $OUT.
set -u
OUT=${OUT:-/tmp/shots}
APK=${1:?apk}
mkdir -p "$OUT"
n=0
shot() { n=$((n+1)); adb exec-out screencap -p > "$OUT/$(printf %02d $n)-$1.png"; dump "$1"; }
dump() {
  adb shell uiautomator dump /sdcard/ui.xml >/dev/null 2>&1
  adb shell cat /sdcard/ui.xml > "$OUT/ui-$1.xml" 2>/dev/null
  python3 - "$OUT/ui-$1.xml" > "$OUT/text-$1.txt" <<'PY'
import re,sys
x=open(sys.argv[1],errors='ignore').read()
for t in re.findall(r'(?:text|content-desc)="([^"]+)"',x):
    print(t)
PY
}
# Where the first element whose text starts with $1 is (or "EDIT" for the first text box); empty if not on screen.
find_xy() {
  adb shell uiautomator dump /sdcard/ui.xml >/dev/null 2>&1
  adb shell cat /sdcard/ui.xml | python3 -c "
import re,sys
x=sys.stdin.read(); want=sys.argv[1]
for m in re.finditer(r'<node [^>]*>',x):
    n=m.group(0)
    t=re.search(r' text=\"([^\"]*)\"',n); d=re.search(r'content-desc=\"([^\"]*)\"',n); c=re.search(r'class=\"([^\"]*)\"',n)
    label=(t.group(1) if t else '') or (d.group(1) if d else '')
    hit = (c and c.group(1)=='android.widget.EditText') if want=='EDIT' else label.strip().startswith(want)
    if hit:
        b=re.search(r'bounds=\"\[(\d+),(\d+)\]\[(\d+),(\d+)\]\"',n)
        if b:
            x1,y1,x2,y2=map(int,b.groups())
            if x2>x1 and y2>y1: print((x1+x2)//2,(y1+y2)//2); break
" "$1"
}
# If Android stopped TUNAS (it restarts apps to apply settings), start it again.
alive() {
  if ! adb shell pidof ci.salt.tunas >/dev/null 2>&1; then
    echo "TUNAS was stopped by Android; starting it again"; adb shell am start -n ci.salt.tunas/.MainActivity; sleep 20; return 1
  fi
  return 0
}
# Tap it, scrolling down to find it if needed.
tap() {
  alive || true
  local xy
  for i in 1 2 3 4; do
    xy=$(find_xy "$1")
    if [ -n "$xy" ]; then echo "tap '$1' at $xy"; adb shell input tap $xy; return 0; fi
    adb shell input swipe $((SW/2)) $((SH*3/4)) $((SW/2)) $((SH/4)) 300
    sleep 1
  done
  echo "not found: '$1'"; return 1
}

# A freshly booted test emulator updates its Google settings in the first minutes and restarts apps to
# apply them; let it settle first.
sleep 90
adb install -r "$APK"
SIZE=$(adb shell wm size | tail -1 | sed 's/.*: //' | tr -d '\r'); SW=${SIZE%x*}; SH=${SIZE#*x}; echo "screen ${SW}x${SH}"
adb shell am start -n ci.salt.tunas/.MainActivity
sleep 25
shot start

APKDIR=$(adb shell pm path ci.salt.tunas | head -1 | sed 's/^package://; s#/base.apk##' | tr -d '\r')
adb shell ls -laR "$APKDIR/lib" > "$OUT/libdir.txt" 2>&1
adb shell "$APKDIR/lib/arm64/libllama_server.so --version" > "$OUT/engine-version.txt" 2>&1
adb shell "$APKDIR/lib/arm64/libllama_server_dp.so --version" >> "$OUT/engine-version.txt" 2>&1
cat "$OUT/libdir.txt" "$OUT/engine-version.txt"

# Wait (up to $2 seconds) until text starting with $1 is on screen.
wait_for() {
  local t=0
  while [ $t -lt ${2:-90} ]; do
    [ -n "$(find_xy "$1")" ] && { echo "on screen: '$1' after ${t}s"; return 0; }
    alive || true
    sleep 5; t=$((t+5))
  done
  echo "never appeared: '$1'"; return 1
}

tap "Get started"
wait_for "Next" 120 || { tap "Get started"; wait_for "Next" 120; }
shot machine
tap "Next"
wait_for "Download and continue" 60; shot models
tap "Download and continue"
wait_for "Make it yours" 90; shot branding
# Skip sits at the bottom of the branding page.
for i in 1 2 3; do
  xy=$(find_xy "Skip"); if [ -n "$xy" ]; then adb shell input tap $xy; break; fi
  adb shell input swipe $((SW/2)) $((SH*3/4)) $((SW/2)) $((SH/4)) 300; sleep 2
done
wait_for "Open TUNAS" 60 || wait_for "Start chatting" 30; shot ready
# The finish button reads "Open TUNAS" while the model downloads and "Start chatting" after.
for i in $(seq 1 12); do
  xy=$(find_xy "Start chatting"); [ -z "$xy" ] && xy=$(find_xy "Open TUNAS")
  if [ -n "$xy" ]; then echo "tap finish at $xy"; adb shell input tap $xy; break; fi
  sleep 5
done
wait_for "EDIT" 90; sleep 5; shot chat

# Wait for the model to download and the engine to load (up to 10 minutes), then ask.
for i in $(seq 1 20); do
  sleep 30
  dump wait
  if ! grep -qiE "downloading|[0-9]+ ?%|MB of" "$OUT/text-wait.txt"; then break; fi
done
shot before-ask
tap "EDIT" ; sleep 2
adb shell input text "Hello%sin%sone%ssentence,%swhat%sis%sMalaysia%sfamous%sfor?"
sleep 2
adb shell input keyevent KEYCODE_ENTER
sleep 3
tap "Send" || true
# The answer is finished when its line of stats appears ("Qwen 3.5 0.8B · 2.1 tokens/s"), or on an error.
for i in $(seq 1 40); do
  sleep 30
  dump answer
  if grep -qiE "tokens/s|No answer was produced|Engine problem|could not start" "$OUT/text-answer.txt"; then break; fi
  [ $((i % 4)) -eq 0 ] && shot "answering-$i"
done
shot answer
adb logcat -d > "$OUT/logcat.txt"
grep -iE "RustStdoutStderr|tunas|panic|FATAL" "$OUT/logcat.txt" | grep -v glBind | tail -80
echo "--- screen text at the end ---"; cat "$OUT/text-answer.txt"
