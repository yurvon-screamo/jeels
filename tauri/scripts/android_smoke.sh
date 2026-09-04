#!/usr/bin/env bash
# Android smoke test: install a debug APK on a booted emulator, launch the
# app, and assert that the process stays alive AND that the startup
# invariants observable via logcat hold:
#   1. the JavaVM/Application publication into ndk-context succeeded
#      (marker: "[android-context] published");
#   2. rustls-platform-verifier initialized (marker: "[rustls]
#      platform-verifier initialized for Android");
#   3. the ndk-context panic text is absent.
# Both markers are emitted through android.util.Log (JNI) — the app's only
# reliable logcat channel: tracing has no logcat backend and native
# stdout/stderr go to /dev/null regardless of debuggability (ADR-044).
#
# Usage: android_smoke.sh <apk-path> [package-id]
# Env: ADB (default "adb"), BOOT_TIMEOUT_S, LAUNCH_TIMEOUT_S, SETTLE_S.
# Requires: adb in PATH; an emulator/device already attached (boot is
# awaited here).
set -euo pipefail

ADB="${ADB:-adb}"
APK="${1:?usage: android_smoke.sh <apk-path> [package-id]}"
PACKAGE="${2:-net.uwuwu.origa}"
BOOT_TIMEOUT_S="${BOOT_TIMEOUT_S:-300}"
LAUNCH_TIMEOUT_S="${LAUNCH_TIMEOUT_S:-15}"
SETTLE_S="${SETTLE_S:-5}"

MARKER_CONTEXT='[android-context] published'
MARKER_RUSTLS='[rustls] platform-verifier initialized for Android'
PANIC_TEXT='android context was not initialized'

fail() {
    echo "::error::android-smoke: $1" >&2
    echo "--- logcat tail ---" >&2
    $ADB logcat -d 2>/dev/null | tail -n 200 >&2 || true
    exit 1
}

# adb pidof exits non-zero when the process is absent; strip whitespace from
# the (possibly CRLF-terminated) shell output.
pid_of() {
    # Keep inner whitespace (multiple pids must stay "123 456", not "123456").
    $ADB shell pidof "$PACKAGE" 2>/dev/null | tr -d '\r' || true
}

[ -f "$APK" ] || fail "APK not found: $APK"

echo "android-smoke: waiting for device + boot completion"
$ADB wait-for-device
deadline=$(( $(date +%s) + BOOT_TIMEOUT_S ))
until [ "$($ADB shell getprop sys.boot_completed 2>/dev/null | tr -d '[:space:]\r')" = "1" ]; do
    [ "$(date +%s)" -lt "$deadline" ] || fail "emulator did not finish booting in ${BOOT_TIMEOUT_S}s"
    sleep 2
done

echo "android-smoke: installing $APK"
$ADB install -r "$APK" > /dev/null 2>&1 || fail "adb install failed for $APK"
$ADB logcat -c

activity="$($ADB shell cmd package resolve-activity --brief -c android.intent.category.LAUNCHER "$PACKAGE" 2>/dev/null | tail -n 1 | tr -d '[:space:]\r' || true)"
[ -n "$activity" ] || fail "could not resolve launcher activity for $PACKAGE"
echo "android-smoke: launching $activity"
$ADB shell am start -n "$activity" > /dev/null || fail "am start failed for $activity"

deadline=$(( $(date +%s) + LAUNCH_TIMEOUT_S ))
until [ -n "$(pid_of)" ]; do
    [ "$(date +%s)" -lt "$deadline" ] || fail "process $PACKAGE did not start within ${LAUNCH_TIMEOUT_S}s"
    sleep 1
done

sleep "$SETTLE_S"
pid="$(pid_of)"
[ -n "$pid" ] || fail "process $PACKAGE died within ${SETTLE_S}s of launch"

logcat="$($ADB logcat -d 2>/dev/null || true)"

case "$logcat" in
    *"$PANIC_TEXT"*) fail "ndk-context panic text found in logcat" ;;
esac
case "$logcat" in
    *"$MARKER_CONTEXT"*) : ;;
    *) fail "context publication marker missing in logcat (app alive but startup invariant violated)" ;;
esac
case "$logcat" in
    *"$MARKER_RUSTLS"*) : ;;
    *) fail "rustls platform-verifier init marker missing in logcat (app alive but verifier not initialized)" ;;
esac

echo "android-smoke: OK (pid=$pid, publication + verifier markers present, no panic)"
