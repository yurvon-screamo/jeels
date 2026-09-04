//! Emit diagnostics to logcat through `android.util.Log`.
//!
//! The only reliable observability channel on Android: tracing has no
//! logcat backend wired in this app (`init_tracing` registers the Sentry
//! layer only on non-desktop targets) and native stdout goes to /dev/null.
//! The CI smoke test asserts on these markers (see ADR-044).

use super::captured_java_vm;

/// Emit a message to logcat through `android.util.Log`. Best-effort by
/// contract: on ANY failure this is a silent no-op (plus an error trace
/// through the JVM-free channel); it must never disrupt initialization.
pub fn logcat_info(message: &str) {
    let Some(vm) = captured_java_vm() else {
        return;
    };
    let Ok(mut attached) = vm.attach_current_thread() else {
        return;
    };
    let env = &mut *attached;
    let logged = (|| -> jni::errors::Result<()> {
        let log_class = env.find_class("android/util/Log")?;
        let tag = env.new_string("origa")?;
        let message = env.new_string(message)?;
        env.call_static_method(
            log_class,
            "i",
            "(Ljava/lang/String;Ljava/lang/String;)I",
            &[(&tag).into(), (&message).into()],
        )?;
        Ok(())
    })();
    if let Err(e) = logged {
        tracing::error!("[android-context] logcat write failed: {e:?}");
    }
}
