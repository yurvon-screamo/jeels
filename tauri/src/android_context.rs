//! Android JNI context ownership: JavaVM + Application context publication.
//!
//! Tauri 2.11 (tao 0.35) stopped publishing the JavaVM and Application
//! context into the [`ndk-context`] crate global, and every reader of
//! `ndk_context::android_context()` (the rustls-platform-verifier init in
//! `lib.rs`, `tauri-plugin-tts`) panicked with "android context was not
//! initialized" on launch. We own that invariant now: the JVM calls
//! [`JNI_OnLoad`] when the native library is loaded (neither tao, wry nor
//! tauri define that symbol), we capture the JavaVM there, resolve the
//! Application context via `ActivityThread.currentApplication()`, and
//! publish both into `ndk-context` for every consumer. See ADR-044.
//!
//! Failure policy: every step degrades to `false` + `tracing::error!` +
//! logcat. Nothing here may panic — it runs across the JNI FFI boundary and
//! release profiles build with `panic = "abort"` (uncatchable).
//!
//! Single-publisher assumption (ADR-044): in this process only this module
//! calls `ndk_context::initialize_android_context`. The CAS guard protects
//! against our own re-entry (a second `JNI_OnLoad` is possible on
//! classloader splits); a hypothetical foreign publisher would abort on
//! ndk-context's `assert!(previous.is_some())` in `panic = "abort"` profiles
//! — `catch_unwind` only helps in unwind-enabled ones (debug/smoke).

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use jni::{JavaVM, objects::JObject};

mod logcat;

pub use logcat::logcat_info;

/// Raw `*mut jni_sys::JavaVM` captured by [`JNI_OnLoad`], stored as `usize`
/// so the static stays `Send + Sync` without depending on jni wrapper types.
static JAVA_VM_PTR: AtomicUsize = AtomicUsize::new(0);
/// Raw `jobject` of the leaked Application global ref (see [`publish`]).
static APP_CONTEXT_PTR: AtomicUsize = AtomicUsize::new(0);
/// Raw `jobject` of the leaked Application class-loader global ref, kept so
/// consumers can resolve app classes from arbitrary threads.
static CLASS_LOADER_PTR: AtomicUsize = AtomicUsize::new(0);
/// Guard against a second publication: `ndk_context::initialize_android_context`
/// asserts on `previous.is_some()`, and a second `JNI_OnLoad` is possible.
static PUBLISHED: AtomicBool = AtomicBool::new(false);

/// Called by the JVM once on `System.loadLibrary` — the earliest point where
/// a JavaVM is reachable, before any Tauri startup code runs. Always returns
/// `JNI_VERSION_1_6`: `JNI_ERR` would fail the `loadLibrary` call and kill
/// the app outright, which is strictly worse than degraded TLS.
#[unsafe(no_mangle)]
pub extern "system" fn JNI_OnLoad(
    vm: *mut jni::sys::JavaVM,
    _reserved: *mut core::ffi::c_void,
) -> jni::sys::jint {
    JAVA_VM_PTR.store(vm as usize, Ordering::Release);
    if ensure_initialized() {
        logcat_info("[android-context] published JavaVM+Application to ndk-context");
    }
    jni::sys::JNI_VERSION_1_6
}

/// Publish the JavaVM + Application context into `ndk-context`, idempotently.
///
/// Returns `true` when a valid context is available to readers (`java_vm`,
/// `app_context`), `false` on any failure — logging the reason, never
/// panicking. Normal flow: `JNI_OnLoad` already did the work and this is a
/// cheap no-op.
pub fn ensure_initialized() -> bool {
    if is_published() {
        return true;
    }
    let Some(vm) = captured_java_vm() else {
        tracing::error!("[android-context] JNI_OnLoad has not run; no JavaVM to publish");
        return false;
    };
    let mut attached = match vm.attach_current_thread() {
        Ok(attached) => attached,
        Err(e) => {
            tracing::error!("[android-context] failed to attach thread to JVM: {e:?}");
            return false;
        },
    };
    let env = &mut *attached;

    let Some(app) = current_application(env) else {
        tracing::error!(
            "[android-context] ActivityThread.currentApplication() returned null; \
             loadLibrary ran before Application.onCreate?"
        );
        return false;
    };
    let Ok(app_global) = env.new_global_ref(&app) else {
        tracing::error!("[android-context] failed to create Application global ref");
        return false;
    };
    let loader = env
        .call_method(&app, "getClassLoader", "()Ljava/lang/ClassLoader;", &[])
        .and_then(|loader| loader.l())
        .and_then(|loader| env.new_global_ref(&loader));
    let Ok(loader_global) = loader else {
        tracing::error!("[android-context] failed to capture Application class loader");
        return false;
    };

    publish(app_global, loader_global)
}

/// Whether `ndk-context` has been populated by this module.
pub fn is_published() -> bool {
    PUBLISHED.load(Ordering::Acquire)
}

/// The JavaVM captured by [`JNI_OnLoad`], if the library was loaded through
/// the JVM.
pub fn java_vm() -> Option<JavaVM> {
    captured_java_vm()
}

/// The Application context (process-lifetime global ref), after publication.
pub fn app_context() -> Option<JObject<'static>> {
    raw_object(APP_CONTEXT_PTR.load(Ordering::Acquire))
}

/// Resolve the Application object. `ActivityThread` is a hidden but stable
/// system class; `currentApplication()` is the standard reflection-based
/// accessor used across the Android ecosystem (see ADR-044).
fn current_application<'local>(env: &mut jni::JNIEnv<'local>) -> Option<JObject<'local>> {
    let activity_thread = env.find_class("android/app/ActivityThread").ok()?;
    let application = env
        .call_static_method(
            activity_thread,
            "currentApplication",
            "()Landroid/app/Application;",
            &[],
        )
        .ok()?
        .l()
        .ok()?;
    if application.as_raw().is_null() {
        return None;
    }
    Some(application)
}

/// Take ownership of both global refs, leak them (process lifetime — exactly
/// the contract `ndk-context` assumes: it stores raw pointers and never
/// releases them), and hand them to `ndk-context` under the CAS guard.
/// Pointers are stored BEFORE the flag: any reader that observes
/// `is_published() == true` always finds valid pointers (the Release on the
/// CAS publishes the stores above). A racing loser of the CAS leaks a second
/// global ref to the same Application — benign, leaking is this module's
/// contract.
fn publish(app_global: jni::objects::GlobalRef, loader_global: jni::objects::GlobalRef) -> bool {
    let app_raw = app_global.as_obj().as_raw();
    let loader_raw = loader_global.as_obj().as_raw();
    APP_CONTEXT_PTR.store(app_raw as usize, Ordering::Release);
    CLASS_LOADER_PTR.store(loader_raw as usize, Ordering::Release);
    std::mem::forget((app_global, loader_global));

    if PUBLISHED
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_err()
    {
        // Another thread won the race and owns the ndk-context publication;
        // our pointers (same underlying objects) are valid either way.
        return true;
    }

    // Safety: both pointers are process-lifetime global refs (leaked above,
    // pinned by the GC) and the JavaVM is the one the JVM handed to
    // JNI_OnLoad. Single-publisher guarantee: the CAS above.
    let publication = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
        ndk_context::initialize_android_context(
            JAVA_VM_PTR.load(Ordering::Acquire) as *mut core::ffi::c_void,
            app_raw as *mut core::ffi::c_void,
        );
    }));
    if publication.is_err() {
        tracing::error!(
            "[android-context] ndk-context reported a pre-existing publisher (panic); \
             our pointers remain valid but the owner is not this module"
        );
    }
    true
}

fn captured_java_vm() -> Option<JavaVM> {
    let raw = JAVA_VM_PTR.load(Ordering::Acquire);
    if raw == 0 {
        return None;
    }
    // Safety: stored verbatim by JNI_OnLoad from the JVM-provided pointer.
    unsafe { JavaVM::from_raw(raw as *mut jni::sys::JavaVM) }.ok()
}

/// Reconstruct a wrapper over a process-lifetime global ref. Safety: the
/// pointer was produced by `new_global_ref` and deliberately leaked in
/// [`publish`]; the GC pins it for the process lifetime, and a plain
/// `JObject` has no drop glue (nothing ever deletes it as a local ref).
fn raw_object(raw: usize) -> Option<JObject<'static>> {
    if raw == 0 {
        return None;
    }
    Some(unsafe { JObject::from_raw(raw as jni::sys::jobject) })
}
