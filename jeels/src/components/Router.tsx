import { useState } from "react";
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
                <Computer style={{ marginRight: 8 }} /> Лента
            </div>
            <div style={{ padding: 4 }} onClick={() => navigate("lessons")}>
                <FolderOpen style={{ marginRight: 8 }} /> Уроки
            </div>
            <div style={{ padding: 4 }} onClick={() => navigate("translate")}>
                <MediaVideo style={{ marginRight: 8 }} /> Переводчик
            </div>
            <div style={{ padding: 4 }} onClick={() => navigate("profile")}>
                <User3 style={{ marginRight: 8 }} /> Профиль
            </div>
        </List>
    );
}

export function Router({ initial = "feed", views, onPageChange }: RouterProps) {
    const [page, setPage] = useState<Page>(initial);
    const navigate = (p: Page) => { setPage(p); onPageChange?.(p); };
    const menu = useStartMenu(navigate);
    const current = views[page];

    return { page, navigate, menu, current } as const;
}


