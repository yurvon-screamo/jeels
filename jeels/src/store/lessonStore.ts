import { parse } from 'yaml';
import { GroupIndexYaml, LessonContent, LessonMeta, LessonYaml, LessonsState, TopIndexYaml } from './types';

// Minimal reactive store without external deps
type Listener = () => void;

class LessonStoreImpl {
    private state: LessonsState = {
        index: [],
        lessonsByKey: {},
        errorsByKey: {},
        loadingKeys: new Set<string>(),
    };

    private listeners: Set<Listener> = new Set();
    private indexLoaded = false;
    private indexLoading: Promise<LessonMeta[]> | null = null;

    subscribe(listener: Listener): () => void {
        this.listeners.add(listener);
        return () => this.listeners.delete(listener);
    }

    getState(): LessonsState {
        return this.state;
    }

    private notify() {
        this.listeners.forEach((l) => l());
    }

    // key format: `${group}/${topic}` to match folder structure under public/content
    private static toKey(meta: LessonMeta): string {
        return `${meta.group}/${meta.topic}`;
    }

    // Build public asset paths. Vite serves /public at the root.
    private static toPaths(meta: LessonMeta) {
        const base = `/content/${meta.group}/${meta.topic}`;
        return {
            general_audio_path: `${base}/general.mp3`,
            general_img_path: `${base}/general.png`,
            practic_audio_path: `${base}/practic.mp3`,
            practic_img_path: `${base}/practic.png`,
            yaml_path: `${base}/content.yaml`,
        };
    }

    // Build index from YAML files at runtime: /content/index.yaml and /content/<group>/index.yaml
    async loadIndexFromYaml(): Promise<LessonMeta[]> {
        const fetchText = async (url: string) => {
            const res = await fetch(url);
            if (!res.ok) throw new Error(`Failed to fetch ${url}: ${res.status}`);
            return await res.text();
        };

        const metas: LessonMeta[] = [];
        // 1) Top-level
        const topUrl = `/content/index.yaml`;
        let top: TopIndexYaml = {};
        try {
            const topText = await fetchText(topUrl);
            top = parse(topText) as TopIndexYaml;
        } catch (e) {
            top = {} as TopIndexYaml;
        }

        if (Array.isArray(top.lessons) && top.lessons.length > 0) {
            for (const l of top.lessons) {
                metas.push({ group: String(l.group), topic: String(l.topic) });
            }
        }

        if (Array.isArray(top.groups) && top.groups.length > 0) {
            for (const group of top.groups) {
                const groupUrl = `/content/${encodeURIComponent(group)}/index.yaml`;
                try {
                    const gText = await fetchText(groupUrl);
                    const g = parse(gText) as GroupIndexYaml;
                    if (Array.isArray(g.lessons) && g.lessons.length > 0) {
                        for (const t of g.lessons) metas.push({ group, topic: String(t.topic) });
                    }
                    if (Array.isArray(g.topics) && g.topics.length > 0) {
                        for (const topic of g.topics) metas.push({ group, topic: String(topic) });
                    }
                } catch {
                    // skip missing group index
                }
            }
        }

        const seen = new Set<string>();
        const list = metas
            .filter((m) => {
                const k = `${m.group}/${m.topic}`;
                if (seen.has(k)) return false;
                seen.add(k);
                return true;
            })
            .sort((a, b) => (a.group === b.group ? a.topic.localeCompare(b.topic) : a.group.localeCompare(b.group)));

        this.state.index = list;
        this.notify();
        this.indexLoaded = true;
        return list;
    }

    async ensureIndexLoaded(): Promise<LessonMeta[]> {
        if (this.indexLoaded) return this.state.index;
        if (this.indexLoading) return this.indexLoading;
        this.indexLoading = this.loadIndexFromYaml()
            .catch(() => [])
            .finally(() => {
                this.indexLoading = null;
            });
        return this.indexLoading;
    }

    async ensureLoaded(meta: LessonMeta): Promise<LessonContent | undefined> {
        const key = LessonStoreImpl.toKey(meta);
        if (this.state.lessonsByKey[key]) return this.state.lessonsByKey[key];
        if (this.state.loadingKeys.has(key)) return undefined;

        this.state.loadingKeys.add(key);
        this.notify();

        try {
            const paths = LessonStoreImpl.toPaths(meta);
            const res = await fetch(paths.yaml_path);
            if (!res.ok) throw new Error(`Failed to fetch ${paths.yaml_path}: ${res.status}`);
            const text = await res.text();
            const parsed = parse(text) as LessonYaml;

            const lesson: LessonContent = {
                group: meta.group,
                topic: parsed.topic ?? meta.topic,
                general_audio_path: paths.general_audio_path,
                general_img_path: paths.general_img_path,
                general_md_content: parsed.general_md_content,
                practic_audio_path: paths.practic_audio_path,
                practic_img_path: paths.practic_img_path,
                practic_md_content: parsed.practic_md_content,
                general_img_promt: parsed.general_img_promt,
                practic_img_promt: parsed.practic_img_promt,
            };

            this.state.lessonsByKey[key] = lesson;
            this.state.errorsByKey[key] = undefined;
        } catch (e) {
            const message = e instanceof Error ? e.message : String(e);
            this.state.errorsByKey[key] = message;
        } finally {
            this.state.loadingKeys.delete(key);
            this.notify();
        }

        return this.state.lessonsByKey[key];
    }
}

export const LessonStore = new LessonStoreImpl();

// Convenience helpers
export function lessonKey(meta: LessonMeta): string {
    return `${meta.group}/${meta.topic}`;
}


