use super::Content;

pub static CONTENT: Content = Content {
    site_name: "Origa",
    keywords: "japanese learning app, learn japanese offline, anki alternative japanese, japanese ocr app, jlpt study tracker, japanese flashcards, kanji learning app, japanese grammar app, learn japanese free, spaced repetition japanese",
    breadcrumb_home: "Home",
    learning_resource_teaches_vocab: "Japanese vocabulary",
    learning_resource_teaches_kanji: "Kanji",
    learning_resource_teaches_grammar: "Japanese grammar",
    learning_resource_teaches_listening: "Japanese listening",

    header_features: "Features",
    header_compare: "Compare",
    header_download: "Download",
    header_integrations: "Content",
    header_blog: "Blog",
    header_docs: "Docs",

    blog_index_meta_title: "Origa Blog — Japanese Learning Field Notes",
    blog_index_meta_description: "Field guides on learning Japanese: Anki alternatives, app selection, vocabulary, kanji, and study workflows. Written by the Origa team.",
    blog_index_h1: "Blog",
    blog_index_subtitle: "Field guides on learning Japanese — when to switch tools, what to look for, and where each app wins.",
    blog_index_empty: "No articles published yet. Check back soon.",
    blog_updated_label: "Updated:",
    breadcrumb_blog: "Blog",
    breadcrumb_docs: "Docs",
    docs_updated_label: "Updated:",

    footer_product: "Product",
    footer_resources: "Resources",
    footer_license: "License",

    home_meta_title: "Origa — App to Learn Japanese: Kanji, Grammar, JLPT, Offline",
    home_meta_description: "Learn Japanese free — vocabulary, kanji, grammar, 200K+ native phrases and JLPT analytics. Offline, in your own language.",
    home_schema_feature_list: "Vocabulary, Kanji, Grammar, Listening, JLPT Analytics, Offline Mode",
    home_hero_title: "The app to learn Japanese in your own language",
    home_hero_subtitle: "Vocabulary, kanji, grammar, listening and 200,000+ native phrases — all in one app. No English required.",
    home_cta_primary: "Download",
    home_cta_secondary: "Open web app",
    home_hero_tagline: "Cross-platform · Offline-first · Privacy-first",
    home_stats_line: "200,000+ native phrases with audio · N5–N1 grammar & kanji · Windows · Linux · macOS · Android · Web · Offline",
    home_faq_free_question: "Is Origa free?",
    home_faq_free_answer: "Yes. The apps for every platform and the web version are free.",
    home_problem_h2: "Five apps for one language",
    home_problem_text: "Vocabulary in Anki. Kanji in WaniKani. Grammar in Bunpro. Listening on YouTube. You stitch together five subscriptions and hope they somehow connect. Origa is one app — vocabulary, kanji, grammar, listening and 200,000+ phrases, built to work together from day one.",
    home_features_h2: "Everything you need. Nothing you don't.",
    home_feature_vocab_title: "Vocabulary",
    home_feature_vocab_text: "Built-in dictionaries in your language. Create cards in seconds — type a word, scan a photo, or record audio.",
    home_feature_kanji_title: "Kanji",
    home_feature_kanji_text: "Automatic furigana that hides as you learn. Writing practice with stroke order. JLPT-mapped kanji dictionaries.",
    home_feature_grammar_title: "Grammar",
    home_feature_grammar_text: "Structured grammar reference by JLPT level. Rules explained with words you already know. Practice tests.",
    home_feature_listening_title: "Phrases",
    home_feature_listening_text: "Phrases from anime, visual novels and native content with original voice acting. Auto-selected for your level.",
    home_principle_content_title: "Learn from your own content.",
    home_principle_content_text: "Study what you're actually reading, watching or listening to — not someone else's word list.",
    home_principle_fsrs_title: "Smart spaced repetition.",
    home_principle_fsrs_text: "FSRS algorithm adapts review intervals to your memory.",
    home_principle_local_title: "Everything runs locally.",
    home_principle_local_text: "All AI models and data processing happen on your device. Nothing is sent to the cloud.",
    home_principle_offline_title: "Works offline.",
    home_principle_offline_text: "Full functionality without internet. Study on the train, on a plane, anywhere.",
    home_cta_title: "Start learning Japanese your way",

    features_meta_title: "Vocabulary, Kanji, Grammar & Listening | Origa",
    features_meta_description: "Built-in dictionaries, smart flashcards, automatic furigana, JLPT grammar and 200K+ phrases with audio — offline, all in one app.",
    features_schema_how_to_name: "How to build Japanese vocabulary with Origa",
    features_h1: "Everything for Japanese learning",
    features_vocab_h2: "Vocabulary",
    features_vocab_label: "How it works",
    features_vocab_step1: "Type or paste a Japanese word or sentence",
    features_vocab_step2: "Scan a photo — OCR extracts words and adds translations automatically",
    features_vocab_step3: "A new word goes through presentation and training, then sticks with spaced repetition",
    features_vocab_dict: "Built-in bilingual dictionaries",
    features_vocab_cards: "Instant card creation",
    features_vocab_audio: "NHK audio pronunciation",
    features_vocab_import: "Import from textbooks",
    features_vocab_fsrs: "FSRS spaced repetition",
    features_vocab_jlpt: "JLPT level tracking",
    features_kanji_h2: "Kanji",
    features_kanji_subtitle: "Automatic furigana, writing practice, JLPT dictionaries and interactive reading tests.",
    features_kanji_furigana: "Automatic furigana",
    features_kanji_furigana_desc: "Furigana appears above kanji and gradually hides as you master each character.",
    features_kanji_writing: "Writing practice",
    features_kanji_writing_desc: "Practice writing with correct stroke order animations.",
    features_kanji_dict: "JLPT kanji dictionaries",
    features_kanji_dict_desc: "Kanji organized by JLPT level from N5 to N1.",
    features_kanji_tests: "Interactive reading tests",
    features_kanji_tests_desc: "Test your kanji reading in context with interactive exercises.",
    features_kanji_insight: "You don't need to learn all jōyō kanji at once. Learn the ones that appear in your content.",
    features_grammar_h2: "Grammar",
    features_grammar_subtitle: "Structured grammar reference by JLPT level with contextual examples.",
    features_grammar_jlpt: "JLPT-organized reference",
    features_grammar_jlpt_desc: "Grammar points organized by JLPT level from N5 to N1.",
    features_grammar_context: "Contextual examples",
    features_grammar_context_desc: "Grammar examples use vocabulary you already know.",
    features_grammar_tests: "Practice tests",
    features_grammar_tests_desc: "Test your understanding with grammar practice tests.",
    features_grammar_search: "Grammar search",
    features_grammar_search_desc: "Look up grammar patterns directly from text you're reading.",
    features_grammar_insight: "Grammar examples use vocabulary you already know — so you focus on the pattern itself.",
    features_listening_h2: "Phrases",
    features_listening_subtitle: "Phrases from real Japanese content, auto-selected for your level.",
    features_listening_n1: "N+1 auto-selection",
    features_listening_n1_desc: "Phrases automatically selected at your current level plus one.",
    features_listening_audio: "Original voice acting",
    features_listening_audio_desc: "Audio from native Japanese content — anime, visual novels and more.",
    features_listening_comp: "Listening comprehension",
    features_listening_comp_desc: "Practice understanding spoken Japanese in context.",
    features_listening_everyday: "Everyday Japanese",
    features_listening_everyday_desc: "Learn phrases used in daily life by native speakers.",
    features_listening_insight: "These phrases come from content made by Japanese speakers, for Japanese speakers.",
    features_cta: "Start learning",
    features_faq_h2: "Questions and answers",
    faq_q1: "How do I start learning Japanese with Origa?",
    faq_a1: "Start from the built-in bilingual dictionaries in your own language. Create a card from a word, a sentence, a photo, or an audio clip; Origa adds furigana, walks it through a short presentation-and-training intro, and schedules reviews with FSRS spaced repetition. If you are new to the language, begin at JLPT N5.",
    faq_q2: "Is Origa an alternative to Anki?",
    faq_a2: "Anki is a powerful tool for building custom decks. Origa adds automatic furigana, OCR card creation, built-in dictionaries, and grammar and listening in one app. You can also import your existing Anki decks.",
    faq_q3: "Can I learn Japanese offline?",
    faq_a3: "Yes. All features work offline: the OCR and speech-to-text models run locally on your device, so you can create and review cards without an internet connection.",
    faq_q4: "Does Origa help with JLPT?",
    faq_a4: "Yes. Origa tracks your JLPT level, organises the grammar reference by N5 to N1, maps kanji dictionaries to JLPT levels, and shows review analytics so you can see where you stand.",
    faq_q5: "Is Origa available in my language?",
    faq_a5: "The interface is available in English and Russian, with built-in bilingual dictionaries; Vietnamese and Korean are planned. You do not need English to start.",

    compare_meta_title: "Origa vs Anki, WaniKani, Bunpro & Duolingo",
    compare_meta_description: "Compare Origa with Anki, WaniKani, Bunpro and Duolingo. See which Japanese learning tools work best together.",
    compare_h1: "Choosing the right Japanese learning tool",
    compare_subtitle: "There's no single perfect app. Here's an honest comparison to help you decide.",
    compare_table_feature: "Feature",
    compare_table_origa: "Origa",
    compare_table_anki: "Anki",
    compare_table_wanikani: "WaniKani",
    compare_table_bunpro: "Bunpro",
    compare_table_duolingo: "Duolingo",
    compare_vocab: "Vocabulary",
    compare_kanji: "Kanji",
    compare_grammar: "Grammar",
    compare_listening: "Listening",
    compare_languages: "Your language",
    compare_offline: "Offline",
    compare_yes: "✅",
    compare_partial: "Partial",
    compare_no: "❌",
    compare_english_only: "English only",
    compare_limited: "Limited",
    compare_manual: "Manual",
    compare_legend_title: "Legend",
    compare_legend_origa: "Origa",
    compare_legend_supported: "Supported",
    compare_legend_partial: "Partial support",
    compare_legend_not_supported: "Not supported",
    compare_legend_value: "Specific value",
    compare_label_best_for: "Best for",
    compare_label_origa_wins: "Origa advantage",
    compare_label_together: "Using both —",
    compare_anki_title: "Anki",
    compare_anki_what: "Powerful general-purpose spaced repetition system.",
    compare_anki_when: "Multi-subject SRS or custom HTML/CSS cards.",
    compare_anki_better: "All-in-one Japanese learning in your native language, zero configuration.",
    compare_anki_together: "Import Anki decks into Origa for a unified experience.",
    compare_wanikani_title: "WaniKani",
    compare_wanikani_what: "Radical-based kanji learning platform.",
    compare_wanikani_when: "Structured kanji path from zero.",
    compare_wanikani_better: "Learn kanji you actually encounter in your content today.",
    compare_wanikani_together: "Build a foundation with WaniKani, then switch to Origa for real-world practice.",
    compare_bunpro_title: "Bunpro",
    compare_bunpro_what: "Grammar drilling platform.",
    compare_bunpro_when: "Focused grammar practice.",
    compare_bunpro_better: "Vocabulary and grammar unified — examples use words you already know.",
    compare_bunpro_together: "Use Bunpro for grammar drilling and Origa as your unified learning system.",
    compare_duolingo_title: "Duolingo",
    compare_duolingo_subtitle: "When Duolingo is right: you want a gentle introduction to Japanese.",
    compare_duolingo_start: "Gamified language learning app for beginners.",
    compare_duolingo_grow: "Real-world practice when you've outgrown beginner content.",
    compare_duolingo_together: "Start with Duolingo, then switch to Origa for growth.",
    compare_bridge_label: "Detailed comparison",
    compare_score: "Coverage",

    download_meta_title: "Download Origa — Offline Japanese Learning App",
    download_meta_description: "Download Origa free for Windows, Linux, macOS or Android. All platforms work offline.",
    download_h1: "Get Origa",
    download_windows: "Windows",
    download_windows_formats: ".exe, .msi",
    download_linux: "Linux",
    download_linux_formats: ".deb, .AppImage, .rpm",
    download_macos: "macOS",
    download_macos_formats: "Mac",
    download_android: "Android",
    download_android_formats: ".apk",
    download_web: "Web",
    download_ios: "iOS",
    download_ios_formats: "iPhone, iPad",
    download_ios_coming_soon: "Coming soon",
    download_button: "Download",
    download_subtitle: "Download Origa to any device — everything works offline.",
    download_try_web: "Open →",

    integrations_meta_title: "Vocabulary Sets & Word Lists | Origa",
    integrations_meta_description: "Import vocabulary from Duolingo, Anki, Minna no Nihongo, Irodori, JLPT and more. Pre-built word sets ready to study.",
    integrations_h1: "Learn from what you already use",
    integrations_subtitle: "Pre-built vocabulary sets from popular Japanese learning sources. Import and start reviewing instantly.",
    integrations_hero_stat: "7 sources · 1000+ word sets",

    integrations_section_exams: "Exam preparation",
    integrations_section_textbooks: "Textbooks",
    integrations_section_apps: "Apps & Content",

    integrations_tag_exam: "Exam",
    integrations_tag_app: "App",
    integrations_tag_textbook: "Textbook",
    integrations_tag_content: "Content",
    integrations_tag_import: "Import",

    integrations_jlpt_name: "JLPT",
    integrations_jlpt_desc: "Official vocabulary for the Japanese Language Proficiency Test, organized by level.",
    integrations_jlpt_detail: "N5 · N4 · N3 · N2 · N1",

    integrations_duolingo_name: "Duolingo",
    integrations_duolingo_desc: "Vocabulary extracted from Duolingo's Japanese course, available in English and Russian tracks.",
    integrations_duolingo_detail: "190+ word sets · English & Russian tracks",

    integrations_minna_name: "Minna no Nihongo",
    integrations_minna_desc: "Word lists from the most popular Japanese textbook series, organized lesson by lesson.",
    integrations_minna_detail: "みんなの日本語 · N5 · N4 · N3 · N2",

    integrations_irodori_name: "Irodori",
    integrations_irodori_desc: "Vocabulary from the Japan Foundation's free JLPT-oriented textbook series.",
    integrations_irodori_detail: "いろどり · 入門 · 初級1 · 初級2 · 54 lessons",

    integrations_migii_name: "Migii",
    integrations_migii_desc: "JLPT test preparation study sets from the Migii app, organized by level.",
    integrations_migii_detail: "JLPT prep sets by level",

    integrations_spy_name: "Spy × Family",
    integrations_spy_desc: "Vocabulary from the popular anime — learn Japanese through content you enjoy.",
    integrations_spy_detail: "Season 1 vocabulary",

    integrations_anki_name: "Anki Import",
    integrations_anki_desc: "Import any Anki deck directly into Origa. Supports .anki2, .anki21 and .anki21b formats.",
    integrations_anki_detail: ".anki2 · .anki21 · .anki21b",
    integrations_anki_note: "Import any Anki deck and continue learning where you left off — all your progress transfers.",

    banner_wip: "",

    footer_legal: "Legal",
    legal_privacy_link: "Privacy Policy",
    legal_terms_link: "Terms of Service",

    privacy_meta_title: "Privacy Policy | Origa",
    privacy_meta_description: "What data Origa collects (account, learning progress, diagnostics), why it is collected, where it is stored, and your rights.",
    privacy_h1: "Privacy Policy",
    privacy_last_updated_label: "Last updated",
    privacy_body: PR_BODY_EN,

    terms_meta_title: "Terms of Service | Origa",
    terms_meta_description: "Terms governing use of Origa, the free Japanese-learning application distributed under the BSL license.",
    terms_h1: "Terms of Service",
    terms_last_updated_label: "Last updated",
    terms_body: TO_BODY_EN,
};

pub const PR_BODY_EN: &str = r#"
<section>
  <h2>What data we collect</h2>
  <p>To operate your account and synchronise your learning progress across devices, Origa stores the following data on its server:</p>
  <table>
    <thead>
      <tr><th>Category</th><th>Details</th></tr>
    </thead>
    <tbody>
      <tr><td>Account</td><td>Email address, username, and a server-assigned identifier.</td></tr>
      <tr><td>Authentication</td><td>Email and password (passwords are hashed server-side and never stored in plain text), or sign-in via Google or Yandex OAuth.</td></tr>
      <tr><td>Learning progress</td><td>Your study cards (vocabulary, kanji, grammar, phrases), spaced-repetition state, review history, JLPT progress, known-vocabulary index, and current Japanese level.</td></tr>
      <tr><td>Settings</td><td>Interface language and daily study-load preference.</td></tr>
      <tr><td>Imported sets</td><td>Identifiers of content sets you imported (the set contents themselves are public catalog data).</td></tr>
      <tr><td>Optional profile</td><td>A Telegram user identifier, only if you choose to link a Telegram account.</td></tr>
      <tr><td>Diagnostics</td><td>Crash reports, performance traces, and technical logs, sent to Sentry (sentry.io), a third-party error-monitoring service. They include the app version, operating system version, and device model. These records are not linked to your account, email, or learning progress.</td></tr>
    </tbody>
  </table>
  <p>Origa does <strong>not</strong> collect advertising identifiers, precise location, contacts, or browsing and search history. The on-device machine-learning models (OCR for camera text recognition, Whisper for speech recognition) run entirely on your device; camera frames and audio are processed locally and are never uploaded.</p>
</section>
<section>
  <h2>How data is used</h2>
  <ul>
    <li>To authenticate you and keep your account secure.</li>
    <li>To synchronise your learning progress between your devices.</li>
    <li>To display your progress, level and review schedule inside the app.</li>
    <li>To diagnose crashes and performance problems, so they can be fixed.</li>
  </ul>
  <p>The email received from an OAuth provider is used solely to identify your account and to synchronise progress; it is not sold or transferred to third parties. Diagnostic records sent to Sentry serve no advertising purpose and are not combined with data from other services.</p>
</section>
<section>
  <h2>Where data is stored</h2>
  <ul>
    <li>Account and progress data: the TrailBase server at <code>app.origa.uwuwu.net</code>, hosted in Singapore.</li>
    <li>Crash reports, performance traces and technical logs: Sentry (sentry.io), hosted in the USA. These records are not linked to your account.</li>
    <li>Static assets (dictionaries, machine-learning models, phrase audio): the CDN at <code>s3.origa.uwuwu.net</code>. These are public catalog content, not user data.</li>
    <li>OAuth sign-in is handled by Google and Yandex according to their respective privacy policies.</li>
  </ul>
</section>
<section>
  <h2>Data retention</h2>
  <p>We retain your data until you request deletion. Origa does not erase accounts automatically after a period of inactivity. You can delete your account at any time from the in-app profile screen, or by contacting us.</p>
</section>
<section>
  <h2>Your rights</h2>
  <ul>
    <li><strong>Access</strong> — request a copy of the data we hold about you.</li>
    <li><strong>Rectification</strong> — correct inaccurate account or profile data.</li>
    <li><strong>Erasure</strong> — delete your account and associated data.</li>
    <li><strong>Portability</strong> — receive your progress data in a machine-readable form.</li>
  </ul>
  <p>To exercise any of these rights, use the in-app account-deletion function or contact us at the address below. If you are located in the European Union (GDPR) or the Russian Federation (Federal Law No. 152-FZ), these rights are reflected in the controls described above.</p>
</section>
<section>
  <h2>Children's privacy</h2>
  <p>Origa is a general-purpose language-learning tool and is not directed at children under 13. We do not knowingly collect data from children under 13. If you believe a child has provided us with personal data, contact us so we can delete it.</p>
</section>
<section>
  <h2>Third-party services</h2>
  <ul>
    <li>Google OAuth (accounts.google.com) — optional sign-in provider.</li>
    <li>Yandex OAuth (oauth.yandex.ru) — optional sign-in provider.</li>
    <li>Sentry (sentry.io) — crash reporting and error monitoring; receives crash reports, performance traces and technical logs, not linked to your account.</li>
    <li>Static CDN (s3.origa.uwuwu.net) — public catalog content.</li>
  </ul>
  <p>Origa does not include advertising SDKs or cross-device tracking SDKs, and the data listed above is not used for advertising and is not transferred to data brokers.</p>
</section>
<section>
  <h2>Changes to this policy</h2>
  <p>If this policy changes, the updated version will be published on this page.</p>
</section>
<section>
  <h2>Contact</h2>
  <p>Questions about this policy can be sent to <a href="mailto:yurvon@ya.ru">yurvon@ya.ru</a>.</p>
</section>
"#;

pub const TO_BODY_EN: &str = r#"
<section>
  <h2>1. Acceptance of terms</h2>
  <p>By installing, accessing or using Origa ("the Service"), you agree to be bound by these Terms of Service. If you do not agree, do not install or use the Service.</p>
</section>
<section>
  <h2>2. Description of the service</h2>
  <p>Origa is a free application for learning the Japanese language. It includes vocabulary, kanji, grammar, listening practice, spaced repetition, and optional on-device optical character recognition and speech recognition. The source code is publicly available on GitHub under the Business Source License (BSL).</p>
</section>
<section>
  <h2>3. License to use</h2>
  <p>The source code of Origa is distributed under the Business Source License. You may use, study and modify the code for your own non-commercial use. Additional restrictions on commercial use apply; see the BSL terms and the change-of-use date specified in the repository's LICENSE file for details.</p>
</section>
<section>
  <h2>4. User accounts</h2>
  <p>To synchronise progress across devices, Origa uses an account identified by your email address. You are responsible for maintaining the confidentiality of your sign-in credentials and for all activity that occurs under your account. Provide accurate registration information and keep it current.</p>
</section>
<section>
  <h2>5. Acceptable use</h2>
  <p>You agree not to misuse the Service, not to attempt to gain unauthorised access to its server or another user's data, and not to use the Service to violate any applicable law. Automated abuse of the Service or its server is prohibited.</p>
</section>
<section>
  <h2>6. User content</h2>
  <p>You are responsible for any content you import into the Service for your own study (for example, personal text notes or imported study decks). You retain ownership of your content. Origa processes your content solely to provide the learning functionality to you.</p>
</section>
<section>
  <h2>7. Disclaimers</h2>
  <p>The Service is provided "as is" and "as available", without warranties of any kind, whether express or implied. Machine-learning outputs (OCR, speech recognition, translation) may contain errors and should not be relied upon as authoritative.</p>
</section>
<section>
  <h2>8. Limitation of liability</h2>
  <p>To the maximum extent permitted by law, the authors and contributors of Origa shall not be liable for any indirect, incidental, special, consequential or punitive damages, or any loss of data, arising out of or related to your use of the Service.</p>
</section>
<section>
  <h2>9. Changes to the service and to these terms</h2>
  <p>The Service may evolve over time. We may update these Terms; the current version will always be available on this page. Continued use after a change constitutes acceptance of the updated Terms.</p>
</section>
<section>
  <h2>10. Governing law</h2>
  <p>These Terms are governed by the laws of the Republic of Kazakhstan.</p>
</section>
<section>
  <h2>11. Contact</h2>
  <p>Questions about these Terms can be sent to <a href="mailto:yurvon@ya.ru">yurvon@ya.ru</a>.</p>
</section>
"#;
