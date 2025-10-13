import { useEffect, useMemo, useSyncExternalStore, useState } from 'react';
import { Button, Fieldset, Frame, Input, Tabs, Tab, Tree } from '@react95/core';
import { LessonStore, UserStore, lessonKey } from '../store';

function useLessonState() {
    const subscribe = (cb: () => void) => LessonStore.subscribe(cb);
    const get = () => LessonStore.getState();
    return useSyncExternalStore(subscribe, get, get);
}

function useUserState() {
    const subscribe = (cb: () => void) => UserStore.subscribe(cb);
    const get = () => UserStore.getState();
    return useSyncExternalStore(subscribe, get, get);
}

export function LessonsView() {
    const lessons = useLessonState();
    const user = useUserState();
    const [query, setQuery] = useState('');
    const [selected, setSelected] = useState<{ group: string; topic: string } | null>(null);

    useEffect(() => {
        // Build index at runtime from YAML indexes
        LessonStore.loadIndexFromYaml();
    }, []);

    // Build grouped view with filtering
    const groups = useMemo(() => {
        const q = query.trim().toLowerCase();
        const map = new Map<string, string[]>();
        for (const m of lessons.index) {
            if (q && !m.topic.toLowerCase().includes(q) && !m.group.toLowerCase().includes(q)) continue;
            if (!map.has(m.group)) map.set(m.group, []);
            map.get(m.group)!.push(m.topic);
        }
        // sort topics per group
        for (const [, arr] of map) arr.sort((a, b) => a.localeCompare(b));
        return Array.from(map.entries()).sort((a, b) => a[0].localeCompare(b[0]));
    }, [query, lessons.index]);

    const treeData = useMemo(() => {
        return groups.map(([group, topics]) => ({
            id: group,
            label: group,
            children: topics.map((topic) => ({
                id: `${group}/${topic}`,
                label: topic,
                onClick: () => setSelected({ group, topic })
            }))
        }));
    }, [groups]);

    const selectedKey = selected ? lessonKey(selected) : '';
    const selectedInUser = selected ? user.learningLessonKeys.includes(selectedKey) : false;
    const loadingSelected = selected ? lessons.loadingKeys.has(selectedKey) : false;
    const errorSelected = selected ? lessons.errorsByKey[selectedKey] : undefined;
    const contentSelected = selected ? lessons.lessonsByKey[selectedKey] : undefined;

    useEffect(() => {
        if (selected) LessonStore.ensureLoaded(selected);
    }, [selected]);

    const renderLines = (text: string, keyPrefix: string) => {
        const normalized = (text ?? '').replace(/\r\n/g, '\n');
        const parts = normalized.includes('\n')
            ? normalized.split(/\n+/)
            : normalized.split(/(?<=[.!?。！？])\s+/);
        return parts.filter((p) => p.trim().length > 0).map((line, i) => (
            <div key={`${keyPrefix}-${i}`} style={{ marginTop: i === 0 ? 0 : 8 }}>{line}</div>
        ));
    };

    return (
        <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
            <Fieldset legend="Поиск">
                <Input
                    value={query}
                    onChange={(e: React.ChangeEvent<HTMLInputElement>) => setQuery(e.target.value)}
                    placeholder="Найти урок..."
                />
            </Fieldset>

            <div style={{ display: 'flex', gap: 12 }}>
                {/* Left: Tree */}
                <Fieldset legend="Дерево" style={{ minWidth: 260, width: 320 }}>
                    <Tree data={treeData as any} />
                </Fieldset>

                {/* Right: Preview */}
                <Fieldset legend="Просмотр" style={{ flex: 1 }}>
                    {!selected && (
                        <div style={{ color: '#555' }}>Выберите тему слева.</div>
                    )}

                    {selected && (
                        <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
                            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                                <div>
                                    <div><b>{selected.group}</b></div>
                                    <div>{selected.topic}</div>
                                </div>
                                <div style={{ display: 'flex', gap: 8 }}>
                                    {!selectedInUser ? (
                                        <Button onClick={() => UserStore.addLesson(selectedKey)}>Добавить в обучение</Button>
                                    ) : (
                                        <Button onClick={() => UserStore.removeLesson(selectedKey)}>Убрать из обучения</Button>
                                    )}
                                </div>
                            </div>

                            {loadingSelected && <div>Загрузка...</div>}
                            {errorSelected && <div style={{ color: 'crimson' }}>Ошибка: {errorSelected}</div>}

                            {contentSelected && (
                                <Tabs style={{ marginTop: 4 }}>
                                    <Tab title="Тема">
                                        <Frame variant="window" style={{ padding: 8, marginTop: 8 }}>
                                            <div style={{ display: 'grid', gridTemplateColumns: '1fr 2fr', gap: 16, alignItems: 'start' }}>
                                                <div>
                                                    <img src={contentSelected.general_img_path} alt={contentSelected.topic} style={{ width: '100%', aspectRatio: '3/4', objectFit: 'cover' }} />
                                                </div>
                                                <div style={{ maxHeight: 360, overflow: 'auto' }}>
                                                    {renderLines(contentSelected.general_md_content, 'g')}
                                                </div>
                                            </div>
                                        </Frame>
                                    </Tab>
                                    <Tab title="Практика">
                                        <Frame variant="window" style={{ padding: 8, marginTop: 8 }}>
                                            <div style={{ display: 'grid', gridTemplateColumns: '1fr 2fr', gap: 16, alignItems: 'start' }}>
                                                <div>
                                                    <img src={contentSelected.practic_img_path} alt={contentSelected.topic} style={{ width: '100%', aspectRatio: '3/4', objectFit: 'cover' }} />
                                                </div>
                                                <div style={{ maxHeight: 360, overflow: 'auto' }}>
                                                    {renderLines(contentSelected.practic_md_content, 'p')}
                                                </div>
                                            </div>
                                        </Frame>
                                    </Tab>
                                </Tabs>
                            )}
                        </div>
                    )}
                </Fieldset>
            </div>
        </div>
    );
}


