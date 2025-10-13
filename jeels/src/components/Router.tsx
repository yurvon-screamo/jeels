import { useEffect, useState } from "react";
import { List } from "@react95/core";
import { Computer, FolderOpen, MediaVideo, User3 } from "@react95/icons";

export type Page = "feed" | "lessons" | "translate" | "profile";

export type RouterProps = {
    initial?: Page;
    views: Record<Page, { title: string; node: React.ReactNode }>;
    onPageChange?: (page: Page) => void;
};

export function useStartMenu(navigate: (p: Page) => void) {
    return (
        <List style={{ minWidth: 200 }}>
            <div style={{ padding: 4 }} onClick={() => navigate("feed")}>
                <Computer style={{ marginRight: 8 }} /> フィード
            </div>
            <div style={{ padding: 4 }} onClick={() => navigate("lessons")}>
                <FolderOpen style={{ marginRight: 8 }} /> レッスン
            </div>
            <div style={{ padding: 4 }} onClick={() => navigate("translate")}>
                <MediaVideo style={{ marginRight: 8 }} /> 翻訳
            </div>
            <div style={{ padding: 4 }} onClick={() => navigate("profile")}>
                <User3 style={{ marginRight: 8 }} /> プロフィール
            </div>
        </List>
    );
}

function getPageFromHash(): Page {
    const hash = window.location.hash || "";
    const clean = hash.replace(/^#\/?/, "");
    switch (clean) {
        case "feed":
        case "lessons":
        case "translate":
        case "profile":
            return clean as Page;
        default:
            return "feed";
    }
}

export function Router({ initial = "feed", views, onPageChange }: RouterProps) {
    const [page, setPage] = useState<Page>(getPageFromHash() || initial);

    useEffect(() => {
        const onHashChange = () => {
            const p = getPageFromHash();
            setPage(p);
            onPageChange?.(p);
        };
        window.addEventListener('hashchange', onHashChange);
        // ensure URL reflects current state on mount
        if (!window.location.hash) {
            window.history.replaceState(null, "", `#/${page}`);
        }
        return () => window.removeEventListener('hashchange', onHashChange);
    }, []);

    const navigate = (p: Page) => {
        setPage(p);
        onPageChange?.(p);
        window.history.pushState(null, "", `#/${p}`);
        // Manually dispatch hashchange for consistency in some environments
        window.dispatchEvent(new HashChangeEvent('hashchange'));
    };
    const menu = useStartMenu(navigate);
    const current = views[page];

    return { page, navigate, menu, current } as const;
}


