// Copyright 2026 yurvon-screamo
// SPDX-License-Identifier: MIT
//
// ASWebAuthenticationSession wrapper for OAuth flows.
//
// Replaces the tauri-plugin-opener + desktop-callback.html + custom-scheme
// flow on iOS. ASWebAuthenticationSession opens Safari in a dedicated auth
// context, intercepts the server-side 303 to `origa://auth/callback`
// (redirect-initiated navigation — JavaScript hops from an interstitial page
// are NOT intercepted), and returns the callback URL via the completion
// handler — no CFBundleURLTypes needed.
//
// `prefersEphemeralWebBrowserSession` is intentionally `false`: Google OAuth
// rejects ephemeral (incognito-like) browser sessions, blocking login.

import AuthenticationServices
import SwiftRs
import Tauri
import UIKit
import WebKit

struct StartAuthArgs: Decodable {
    let url: String
    let callbackScheme: String
}

class AsWebAuthPlugin: Plugin, ASWebAuthenticationPresentationContextProviding {
    /// Strong reference to the active session. If this property is nil'd
    /// (or the plugin is deallocated) the session's completion handler
    /// will never fire, hanging the OAuth flow indefinitely.
    private var session: ASWebAuthenticationSession?

    @objc public func startAuth(_ invoke: Invoke) throws {
        let args = try invoke.parseArgs(StartAuthArgs.self)
        guard let url = URL(string: args.url) else {
            invoke.reject("Invalid URL")
            return
        }

        let session = ASWebAuthenticationSession(
            url: url,
            callbackURLScheme: args.callbackScheme
        ) { callbackURL, error in
            // Clear the strong reference so the session can be deallocated.
            self.session = nil

            if let error = error as? ASWebAuthenticationSessionError,
               error.code == .canceledLogin {
                invoke.reject("cancelled")
            } else if let error = error {
                invoke.reject("session failed: \(error.localizedDescription)")
            } else if let callbackURL = callbackURL {
                invoke.resolve(["url": callbackURL.absoluteString])
            } else {
                invoke.reject("no callback URL")
            }
        }

        session.presentationContextProvider = self
        // Google OAuth rejects ephemeral (incognito) sessions. Keep this false
        // so that Google and Yandex logins both work.
        session.prefersEphemeralWebBrowserSession = false

        self.session = session

        DispatchQueue.main.async {
            session.start()
        }
    }

    // MARK: - ASWebAuthenticationPresentationContextProviding

    func presentationAnchor(
        for session: ASWebAuthenticationSession
    ) -> ASPresentationAnchor {
        // Return the app's key window so Safari's auth sheet appears on top
        // of the WebView, not a blank anchor. Uses the foreground active scene
        // to avoid returning a backgrounded window.
        let scenes = UIApplication.shared.connectedScenes
            .compactMap { $0 as? UIWindowScene }
            .filter { $0.activationState == .foregroundActive }

        for scene in scenes {
            for window in scene.windows where window.isKeyWindow {
                return window
            }
        }

        // Fallback: first window of the first scene.
        if let scene = scenes.first, let window = scene.windows.first {
            return window
        }

        // Last resort: empty anchor. The auth session may fail to display,
        // but this should be unreachable in a normal app lifecycle.
        return ASPresentationAnchor()
    }
}

@_cdecl("init_plugin_aswebauth")
func initPlugin() -> Plugin {
    return AsWebAuthPlugin()
}
