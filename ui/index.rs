
    html { lang: "ru",
        head {
            meta { charset: "UTF-8" }
            meta {
                content: "width=device-width, initial-scale=1.0",
                name: "viewport",
            }
            title { "Rainbow Uwuwu UI Kit" }
            script { src: "https://cdn.tailwindcss.com" }
            style {
                "@font-face {\n            font-family: 'Maple Mono';\n            src: url('https://cdn.jsdelivr.net/gh/subframe7536/maple-font@v6.4/woff2/MapleMono-Regular.woff2') format('woff2');\n            font-weight: 400;\n        }\n\n        @font-face {\n            font-family: 'Maple Mono';\n            src: url('https://cdn.jsdelivr.net/gh/subframe7536/maple-font@v6.4/woff2/MapleMono-Bold.woff2') format('woff2');\n            font-weight: 700;\n        }\n\n        /* Кастомные стили для скроллбара */\n        ::-webkit-scrollbar {\n            width: 6px;\n        }\n\n        ::-webkit-scrollbar-track {\n            background: transparent;\n        }\n\n        ::-webkit-scrollbar-thumb {\n            background: #cbd5e1;\n            border-radius: 10px;\n        }\n\n        /* Анимации входа */\n        @keyframes slide-up {\n            from {\n                opacity: 0;\n                transform: translateY(30px);\n            }\n\n            to {\n                opacity: 1;\n                transform: translateY(0);\n            }\n        }\n\n        .animate-enter {\n            animation: slide-up 0.8s cubic-bezier(0.16, 1, 0.3, 1) forwards;\n            opacity: 0;\n        }\n\n        /* Задержки для каскада */\n        .delay-100 {\n            animation-delay: 100ms;\n        }\n\n        .delay-200 {\n            animation-delay: 200ms;\n        }\n\n        .delay-300 {\n            animation-delay: 300ms;\n        }\n\n        .delay-400 {\n            animation-delay: 400ms;\n        }\n\n        .delay-500 {\n            animation-delay: 500ms;\n        }\n\n        /* Кастомный чекбокс и радио */\n        .custom-check:checked {\n            background-color: #F472B6;\n            border-color: #F472B6;\n            background-image: url(\"data:image/svg+xml,%3csvg viewBox='0 0 16 16' fill='white' xmlns='http://www.w3.org/2000/svg'%3e%3cpath d='M12.207 4.793a1 1 0 010 1.414l-5 5a1 1 0 01-1.414 0l-2-2a1 1 0 011.414-1.414L6.5 9.086l4.293-4.293a1 1 0 011.414 0z'/%3e%3c/svg%3e\");\n        }\n\n        /* ОБНОВЛЕННЫЙ СВИТЧ (Яркий градиент при включении) */\n        .bg-rainbow-switch:checked {\n            background-image: linear-gradient(to right, #F472B6, #A78BFA, #22D3EE);\n            /* Pink -> Purple -> Cyan */\n            opacity: 1;\n        }\n\n        /* Убираем стандартный серый фон при чеке, чтобы градиент был виден */\n        .peer:checked~.peer-checked-bg {\n            background-color: transparent;\n        }"
            }
            script {
                "tailwind.config = {\n            theme: {\n                extend: {\n                    fontFamily: { sans: ['Maple Mono', 'monospace'] },\n                    colors: {\n                        bg: '#F8F9FD',\n                        surface: '#FFFFFF',\n                        accent: {\n                            pink: '#F472B6',\n                            purple: '#A78BFA',\n                            cyan: '#22D3EE',\n                        },\n                        text: {\n                            main: '#334155',\n                            muted: '#94A3B8'\n                        }\n                    },\n                    backgroundImage: {\n                        'rainbow-soft': 'linear-gradient(to right top, #ffc3a0, #ffafbd, #c9afff, #a0e9ff)',\n                        'rainbow-vibrant': 'linear-gradient(to right, #F472B6, #A78BFA, #60A5FA)',\n                    },\n                    boxShadow: {\n                        'soft': '0 20px 40px -15px rgba(200, 100, 200, 0.05)',\n                        'soft-hover': '0 30px 60px -12px rgba(200, 100, 255, 0.1)',\n                        'glow': '0 0 25px rgba(244, 114, 182, 0.4), 0 0 10px rgba(34, 211, 238, 0.3)',\n                    },\n                    transitionTimingFunction: {\n                        'elastic': 'cubic-bezier(0.68, -0.55, 0.265, 1.55)',\n                        'smooth': 'cubic-bezier(0.16, 1, 0.3, 1)',\n                    }\n                }\n            }\n        }"
            }
        }
        body { class: "bg-bg text-text-main p-8 min-h-screen selection:bg-accent-pink/30 selection:text-text-main",
            div { class: "fixed top-[-20%] left-[-10%] w-[700px] h-[700px] bg-purple-300/30 rounded-full mix-blend-multiply filter blur-[150px] animate-pulse duration-[8000ms] pointer-events-none" }
            div { class: "fixed top-[10%] right-[-20%] w-[600px] h-[600px] bg-accent-pink/30 rounded-full mix-blend-multiply filter blur-[150px] animate-pulse duration-[10000ms] pointer-events-none" }
            div { class: "fixed bottom-[-10%] right-[10%] w-[600px] h-[600px] bg-accent-cyan/30 rounded-full mix-blend-multiply filter blur-[150px] animate-pulse duration-[12000ms] pointer-events-none" }
            div { class: "max-w-7xl mx-auto relative z-10",
                header { class: "mb-12 animate-enter",
                    h1 { class: "text-4xl md:text-5xl font-bold mb-2 tracking-tight",
                        "\n                Rainbow\n                "
                        span { class: "bg-clip-text text-transparent bg-rainbow-vibrant",
                            "Uwuwu UI Kit"
                        }
                    }
                    p { class: "text-text-muted text-lg",
                        "Полная библиотека компонентов Uwuwu UI"
                    }
                }
                div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8",
                    div { class: "bg-surface rounded-[2rem] p-8 shadow-soft hover:shadow-soft-hover transition-shadow duration-500 animate-enter delay-100 flex flex-col justify-between relative overflow-hidden",
                        div {
                            span { class: "text-xs font-bold text-text-muted uppercase tracking-widest mb-4 block",
                                "01. Typography\n                        & Card"
                            }
                            h1 { class: "text-3xl font-bold mb-2 text-slate-800", "Headline H1" }
                            h2 { class: "text-2xl font-bold mb-2 text-slate-700", "Headline H2" }
                            h3 { class: "text-xl font-bold mb-4 text-slate-600", "Headline H3" }
                            hr { class: "border-slate-100 my-4" }
                            p { class: "text-slate-500 leading-relaxed text-sm",
                                "\n                        Шрифты и цвета адаптированы под мягкую, \"зефирную\" палитру. Черный цвет заменен на глубокие\n                        оттенки серого для снижения контраста.\n                    "
                            }
                        }
                        div { class: "mt-6 pt-4 border-t border-slate-50",
                            span { class: "inline-block px-3 py-1 rounded-lg bg-gradient-to-r from-pink-50 to-cyan-50 text-accent-purple text-xs font-bold",
                                "Rainbow\n                        Tag"
                            }
                        }
                    }
                    div { class: "bg-surface rounded-[2rem] p-8 shadow-soft hover:shadow-soft-hover transition-shadow duration-500 animate-enter delay-200 relative overflow-hidden",
                        span { class: "text-xs font-bold text-text-muted uppercase tracking-widest mb-6 block",
                            "02. Buttons"
                        }
                        div { class: "flex flex-col gap-4",
                            button { class: "w-full py-3.5 rounded-xl bg-rainbow-vibrant text-white font-bold shadow-lg shadow-accent-purple/30 hover:scale-[1.02] hover:shadow-accent-pink/40 active:scale-95 transition-all duration-300 ease-elastic relative overflow-hidden group",
                                div { class: "absolute inset-0 flex items-center justify-center pointer-events-none",
                                    div { class: "w-[200%] h-[200%] bg-white/20 rounded-full scale-0 group-hover:scale-100 transition-transform duration-500 ease-out" }
                                }
                                span { class: "relative z-10", "Rainbow Action" }
                            }
                            button { class: "w-full py-3.5 rounded-xl bg-pink-50 text-accent-pink font-bold hover:bg-cyan-50 hover:text-accent-cyan transition-all duration-300",
                                "\n                        Pearlescent Soft\n                    "
                            }
                            button { class: "w-full py-3.5 rounded-xl border-2 border-slate-100 text-slate-500 font-bold hover:border-accent-pink hover:text-accent-pink group transition-all duration-300 relative overflow-hidden",
                                span { class: "relative z-10", "Outline Button" }
                            }
                            div { class: "flex gap-4 justify-center mt-2",
                                button { class: "w-12 h-12 rounded-xl bg-white shadow-md shadow-accent-pink/15 text-slate-400 flex items-center justify-center hover:text-accent-pink hover:shadow-glow transition-all duration-300",
                                    svg {
                                        class: "w-6 h-6",
                                        fill: "none",
                                        stroke: "currentColor",
                                        view_box: "0 0 24 24",
                                        path {
                                            d: "M4.318 6.318a4.5 4.5 0 000 6.364L12 20.364l7.682-7.682a4.5 4.5 0 00-6.364-6.364L12 7.636l-1.318-1.318a4.5 4.5 0 00-6.364 0z",
                                            stroke_linecap: "round",
                                            stroke_linejoin: "round",
                                            stroke_width: "2",
                                        }
                                    }
                                }
                                button { class: "w-12 h-12 rounded-full bg-rainbow-vibrant text-white flex items-center justify-center shadow-md shadow-accent-pink/15 hover:scale-110 hover:shadow-glow transition-all duration-300 ease-elastic",
                                    svg {
                                        class: "w-5 h-5",
                                        fill: "none",
                                        stroke: "currentColor",
                                        view_box: "0 0 24 24",
                                        path {
                                            d: "M14 5l7 7m0 0l-7 7m7-7H3",
                                            stroke_linecap: "round",
                                            stroke_linejoin: "round",
                                            stroke_width: "2",
                                        }
                                    }
                                }
                            }
                        }
                    }
                    div { class: "bg-surface rounded-[2rem] p-8 shadow-soft hover:shadow-soft-hover transition-shadow duration-500 animate-enter delay-300 relative overflow-hidden",
                        span { class: "text-xs font-bold text-text-muted uppercase tracking-widest mb-6 block",
                            "03. Inputs &\n                    Textarea"
                        }
                        div { class: "space-y-5",
                            div { class: "group",
                                label { class: "block text-xs font-bold text-slate-400 mb-2 ml-1 group-focus-within:text-accent-pink transition-colors",
                                    "EMAIL"
                                }
                                input {
                                    class: "w-full px-5 py-3 rounded-xl bg-slate-50 border border-transparent text-slate-700 font-medium placeholder-slate-300 transition-all duration-300 focus:bg-white focus:border-pink-200 focus:ring-4 focus:ring-pink-50 focus:outline-none focus:shadow-sm",
                                    placeholder: "hello@maple.com",
                                    r#type: "text",
                                }
                            }
                            div { class: "group relative",
                                label { class: "block text-xs font-bold text-slate-400 mb-2 ml-1 group-focus-within:text-accent-cyan transition-colors",
                                    "SEARCH"
                                }
                                input {
                                    class: "w-full pl-11 pr-5 py-3 rounded-xl bg-slate-50 border border-transparent text-slate-700 font-medium placeholder-slate-300 transition-all duration-300 focus:bg-white focus:border-cyan-200 focus:ring-4 focus:ring-cyan-50 focus:outline-none",
                                    placeholder: "Find component...",
                                    r#type: "text",
                                }
                                svg {
                                    class: "w-5 h-5 absolute left-4 top-[38px] text-slate-400 group-focus-within:text-accent-cyan transition-colors",
                                    fill: "none",
                                    stroke: "currentColor",
                                    view_box: "0 0 24 24",
                                    path {
                                        d: "M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z",
                                        stroke_linecap: "round",
                                        stroke_linejoin: "round",
                                        stroke_width: "2",
                                    }
                                }
                            }
                            div { class: "group",
                                label { class: "block text-xs font-bold text-slate-400 mb-2 ml-1 group-focus-within:text-accent-purple transition-colors",
                                    "MESSAGE"
                                }
                                textarea {
                                    class: "w-full px-5 py-3 rounded-xl bg-slate-50 border border-transparent text-slate-700 font-medium placeholder-slate-300 transition-all duration-300 focus:bg-white focus:border-purple-200 focus:ring-4 focus:ring-purple-50 focus:outline-none resize-none",
                                    placeholder: "Type something nice...",
                                    rows: "3",
                                }
                            }
                        }
                    }
                    div { class: "bg-surface rounded-[2rem] p-8 shadow-soft hover:shadow-soft-hover transition-shadow duration-500 animate-enter delay-400 relative overflow-hidden",
                        span { class: "text-xs font-bold text-text-muted uppercase tracking-widest mb-6 block",
                            "04. Toggles &\n                    Controls"
                        }
                        div { class: "space-y-6",
                            div { class: "flex items-center justify-between p-3 rounded-xl bg-slate-50",
                                span { class: "text-sm font-bold text-slate-600", "Rainbow Mode" }
                                label { class: "relative inline-flex items-center cursor-pointer",
                                    input {
                                        checked: "false",
                                        class: "sr-only peer bg-rainbow-switch",
                                        r#type: "checkbox",
                                    }
                                    div { class: "w-12 h-7 bg-slate-200 peer-focus:outline-none rounded-full peer peer-checked:bg-transparent transition-all duration-300 ease-smooth relative overflow-hidden peer-checked-bg",
                                        div { class: "absolute inset-0 bg-gradient-to-r from-pink-400 via-purple-400 to-cyan-400 opacity-0 peer-checked:opacity-100 transition-opacity duration-300" }
                                    }
                                    div { class: "absolute left-[4px] top-[4px] bg-white w-5 h-5 rounded-full shadow-md transition-transform duration-300 ease-elastic peer-checked:translate-x-5 z-10" }
                                }
                            }
                            hr { class: "border-slate-100" }
                            div { class: "space-y-3",
                                label { class: "flex items-center gap-3 cursor-pointer group",
                                    input {
                                        checked: "false",
                                        class: "custom-check appearance-none w-5 h-5 rounded-lg border-2 border-slate-300 bg-white transition-all duration-200 ease-elastic select-none",
                                        r#type: "checkbox",
                                    }
                                    span { class: "text-sm font-medium text-slate-600 group-hover:text-accent-pink transition-colors",
                                        "Soft\n                                notifications"
                                    }
                                }
                                label { class: "flex items-center gap-3 cursor-pointer group",
                                    input {
                                        class: "custom-check appearance-none w-5 h-5 rounded-lg border-2 border-slate-300 bg-white transition-all duration-200 ease-elastic select-none",
                                        r#type: "checkbox",
                                    }
                                    span { class: "text-sm font-medium text-slate-600 group-hover:text-accent-pink transition-colors",
                                        "Holographic\n                                newsletter"
                                    }
                                }
                            }
                            div { class: "space-y-3",
                                label { class: "flex items-center gap-3 cursor-pointer group",
                                    div { class: "relative flex items-center",
                                        input {
                                            checked: "false",
                                            class: "peer appearance-none w-5 h-5 rounded-full border-2 border-slate-300 checked:border-accent-pink transition-colors",
                                            name: "plan",
                                            r#type: "radio",
                                        }
                                        div { class: "absolute inset-0 m-auto w-2.5 h-2.5 rounded-full bg-accent-pink scale-0 peer-checked:scale-100 transition-transform duration-300 ease-elastic" }
                                    }
                                    span { class: "text-sm font-medium text-slate-600",
                                        "Free Plan"
                                    }
                                }
                                label { class: "flex items-center gap-3 cursor-pointer group",
                                    div { class: "relative flex items-center",
                                        input {
                                            class: "peer appearance-none w-5 h-5 rounded-full border-2 border-slate-300 checked:border-accent-pink transition-colors",
                                            name: "plan",
                                            r#type: "radio",
                                        }
                                        div { class: "absolute inset-0 m-auto w-2.5 h-2.5 rounded-full bg-accent-pink scale-0 peer-checked:scale-100 transition-transform duration-300 ease-elastic" }
                                    }
                                    span { class: "text-sm font-medium text-slate-600",
                                        "Pro Plan"
                                    }
                                }
                            }
                        }
                    }
                    div { class: "bg-surface rounded-[2rem] p-8 shadow-soft hover:shadow-soft-hover transition-shadow duration-500 animate-enter delay-500 overflow-visible relative",
                        span { class: "text-xs font-bold text-text-muted uppercase tracking-widest mb-6 block",
                            "05. Selects &\n                    Dropdowns"
                        }
                        div { class: "space-y-6",
                            div {
                                class: "relative w-full z-30",
                                id: "customSelect",
                                label { class: "block text-xs font-bold text-slate-400 mb-2 ml-1",
                                    "CATEGORY"
                                }
                                button {
                                    class: "relative w-full px-5 py-4 rounded-xl bg-slate-50 text-left cursor-pointer outline-none focus:bg-white focus:ring-4 focus:ring-purple-50 focus:shadow-lg transition-all duration-300 group",
                                    id: "dropdownBtn",
                                    "onclick": "toggleDropdown()",
                                    span {
                                        class: "font-medium text-slate-700",
                                        id: "selectedValue",
                                        "Design System"
                                    }
                                    div {
                                        class: "absolute right-5 top-1/2 -translate-y-1/2 pointer-events-none transition-transform duration-300",
                                        id: "dropdownArrow",
                                        svg {
                                            class: "w-5 h-5 text-slate-400",
                                            fill: "none",
                                            stroke: "currentColor",
                                            view_box: "0 0 24 24",
                                            path {
                                                d: "M19 9l-7 7-7-7",
                                                stroke_linecap: "round",
                                                stroke_linejoin: "round",
                                                stroke_width: "2",
                                            }
                                        }
                                    }
                                }
                                div {
                                    class: "absolute top-full left-0 w-full mt-2 bg-white rounded-2xl shadow-soft-hover border border-slate-100 opacity-0 invisible scale-95 transform transition-all duration-300 origin-top overflow-hidden z-40",
                                    id: "dropdownMenu",
                                    ul { class: "flex flex-col p-2",
                                        li {
                                            class: "px-4 py-3 rounded-xl text-slate-600 font-medium hover:bg-purple-50 hover:text-accent-purple cursor-pointer transition-colors duration-200",
                                            "onclick": "selectOption('Design System')",
                                            "\n                                    Design System"
                                        }
                                        li {
                                            class: "px-4 py-3 rounded-xl text-slate-600 font-medium hover:bg-pink-50 hover:text-accent-pink cursor-pointer transition-colors duration-200",
                                            "onclick": "selectOption('Development')",
                                            "\n                                    Development"
                                        }
                                        li {
                                            class: "px-4 py-3 rounded-xl text-slate-600 font-medium hover:bg-cyan-50 hover:text-accent-cyan cursor-pointer transition-colors duration-200",
                                            "onclick": "selectOption('Marketing Strategy')",
                                            "\n                                    Marketing Strategy"
                                        }
                                    }
                                }
                            }
                            div { class: "relative group z-10",
                                button { class: "w-full flex items-center justify-between px-5 py-3 rounded-xl bg-white border border-slate-200 text-slate-700 font-medium shadow-sm hover:border-accent-pink/50 transition-all",
                                    span { "More Options" }
                                    svg {
                                        class: "w-4 h-4 transition-transform group-focus-within:rotate-180",
                                        fill: "none",
                                        stroke: "currentColor",
                                        view_box: "0 0 24 24",
                                        path {
                                            d: "M19 9l-7 7-7-7",
                                            stroke_linecap: "round",
                                            stroke_linejoin: "round",
                                            stroke_width: "2",
                                        }
                                    }
                                }
                                div { class: "absolute top-full left-0 w-full mt-2 bg-white rounded-xl shadow-xl border border-slate-100 opacity-0 invisible group-hover:opacity-100 group-hover:visible transition-all duration-300 transform translate-y-2 group-hover:translate-y-0 z-20",
                                    a {
                                        class: "block px-4 py-3 text-sm text-slate-600 hover:bg-pink-50 hover:text-accent-pink first:rounded-t-xl transition-colors",
                                        href: "#",
                                        "Option\n                                One"
                                    }
                                    a {
                                        class: "block px-4 py-3 text-sm text-slate-600 hover:bg-cyan-50 hover:text-accent-cyan last:rounded-b-xl transition-colors",
                                        href: "#",
                                        "Option\n                                Three"
                                    }
                                }
                            }
                        }
                    }
                    div { class: "bg-rainbow-soft rounded-[2rem] p-8 shadow-xl shadow-accent-purple/20 text-white animate-enter delay-500 relative overflow-hidden group",
                        div { class: "absolute top-0 right-0 w-40 h-40 bg-white/20 opacity-50 rounded-full blur-3xl -translate-y-1/2 translate-x-1/2 group-hover:scale-125 transition-transform duration-700 mix-blend-overlay" }
                        div { class: "absolute bottom-0 left-0 w-32 h-32 bg-accent-cyan/30 opacity-50 rounded-full blur-2xl translate-y-1/2 -translate-x-1/2 group-hover:scale-125 transition-transform duration-700 mix-blend-overlay" }
                        span { class: "text-xs font-bold text-white/70 uppercase tracking-widest mb-6 block relative z-10",
                            "06.\n                    Rainbow Composition"
                        }
                        h3 { class: "text-2xl font-bold mb-6 relative z-10 text-white",
                            "Join the Holographic Waitlist"
                        }
                        form { class: "space-y-4 relative z-10",
                            div { class: "grid grid-cols-2 gap-4",
                                input {
                                    class: "w-full px-4 py-3 rounded-xl bg-white/20 border border-white/30 text-white placeholder-white/70 focus:bg-white/30 focus:outline-none focus:ring-2 focus:ring-white/50 backdrop-blur-md transition-all",
                                    placeholder: "Name",
                                    r#type: "text",
                                }
                                input {
                                    class: "w-full px-4 py-3 rounded-xl bg-white/20 border border-white/30 text-white placeholder-white/70 focus:bg-white/30 focus:outline-none focus:ring-2 focus:ring-white/50 backdrop-blur-md transition-all",
                                    placeholder: "Surname",
                                    r#type: "text",
                                }
                            }
                            input {
                                class: "w-full px-4 py-3 rounded-xl bg-white/20 border border-white/30 text-white placeholder-white/70 focus:bg-white/30 focus:outline-none focus:ring-2 focus:ring-white/50 backdrop-blur-md transition-all",
                                placeholder: "email@address.com",
                                r#type: "email",
                            }
                            button {
                                class: "w-full py-3.5 rounded-xl bg-white text-accent-purple font-bold shadow-lg hover:text-accent-pink hover:scale-[1.02] active:scale-95 transition-all duration-300 ease-elastic mt-2",
                                r#type: "button",
                                "\n                        Get Early Access\n                    "
                            }
                        }
                    }
                }
            }
            script {
                "function toggleDropdown() {\n            const menu = document.getElementById('dropdownMenu');\n            const arrow = document.getElementById('dropdownArrow');\n\n            if (menu.classList.contains('invisible')) {\n                menu.classList.remove('invisible', 'opacity-0', 'scale-95');\n                menu.classList.add('visible', 'opacity-100', 'scale-100');\n                arrow.classList.add('rotate-180');\n            } else {\n                menu.classList.add('invisible', 'opacity-0', 'scale-95');\n                menu.classList.remove('visible', 'opacity-100', 'scale-100');\n                arrow.classList.remove('rotate-180');\n            }\n        }\n\n        function selectOption(value) {\n            document.getElementById('selectedValue').innerText = value;\n            toggleDropdown();\n        }\n\n        document.addEventListener('click', function (event) {\n            const dropdown = document.getElementById('customSelect');\n            if (!dropdown.contains(event.target)) {\n                const menu = document.getElementById('dropdownMenu');\n                const arrow = document.getElementById('dropdownArrow');\n                menu.classList.add('invisible', 'opacity-0', 'scale-95');\n                menu.classList.remove('visible', 'opacity-100', 'scale-100');\n                arrow.classList.remove('rotate-180');\n            }\n        });"
            }
        }
    }