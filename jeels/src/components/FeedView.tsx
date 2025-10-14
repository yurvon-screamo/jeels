import { useEffect, useMemo, useRef, useState, useSyncExternalStore } from 'react';
import { Button, Fieldset, Frame, ProgressBar } from '@react95/core';
import { useClippy } from '@react95/clippy';
import { LessonStore, UserStore, lessonKey, LessonContent, LessonMeta } from '../store';

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

function splitIntoSentences(text: string): string[] {
    const normalized = (text ?? '').replace(/\r\n/g, '\n');
    const parts = normalized.includes('\n')
        ? normalized.split(/\n+/)
        : normalized.split(/(?<=[.!?。！？])\s+/);
    return parts.map((p) => p.trim()).filter((p) => p.length > 0);
}

function pickCurrentLessonMeta(): LessonMeta | null {
    const user = UserStore.getState();
    const lessons = LessonStore.getState();
    const key = user.learningLessonKeys[0] ?? (lessons.index[0] && lessonKey(lessons.index[0]));
    if (!key) return null;
    const [group, ...topicParts] = key.split('/');
    const topic = topicParts.join('/');
    return group && topic ? { group, topic } : null;
}

export function FeedView() {
    const lessons = useLessonState();
    const user = useUserState();
    const [meta, setMeta] = useState<LessonMeta | null>(null);
    const [content, setContent] = useState<LessonContent | null>(null);
    const [activeIdx, setActiveIdx] = useState(0);
    const [isHard, setIsHard] = useState(false);
    const [isPlaying, setIsPlaying] = useState(false);
    const [progress, setProgress] = useState(0);
    const [currentPart, setCurrentPart] = useState<'general' | 'practice'>('general');
    const [durGeneral, setDurGeneral] = useState(0);
    const [durPractice, setDurPractice] = useState(0);
    const audioGeneralRef = useRef<HTMLAudioElement | null>(null);
    const audioPracticeRef = useRef<HTMLAudioElement | null>(null);
    const linesRefs = useRef<Array<HTMLDivElement | null>>([]);

    useEffect(() => {
        LessonStore.ensureIndexLoaded().catch(() => { });
    }, []);

    // update meta when index or user changes
    useEffect(() => {
        const m = pickCurrentLessonMeta();
        setMeta(m);
        setActiveIdx(0);
    }, [user.learningLessonKeys, lessons.index.length]);

    // load lesson content
    useEffect(() => {
        if (!meta) return;
        LessonStore.ensureLoaded(meta).then((c) => {
            if (c) setContent(c);
        });
    }, [meta]);

    // sentence splitting (general + practice)
    const generalSentences = useMemo(() => splitIntoSentences(content?.general_md_content ?? ''), [content?.general_md_content]);
    const practiceSentences = useMemo(() => splitIntoSentences(content?.practic_md_content ?? ''), [content?.practic_md_content]);
    const sentences = useMemo(() => [...generalSentences, ...practiceSentences], [generalSentences, practiceSentences]);

    // sync highlighting with audio progress across both parts
    const onTimeUpdateGeneral = () => {
        const el = audioGeneralRef.current;
        if (!el || !content) return;
        const duration = durGeneral || el.duration || 0;
        const current = el.currentTime || 0;
        if (duration <= 0 || sentences.length === 0) return;
        const ratio = Math.max(0, Math.min(1, current / duration));
        const idx = Math.min(generalSentences.length - 1, Math.floor(ratio * generalSentences.length));
        const globalIdx = idx;
        setActiveIdx(globalIdx);
        const practiceDur = durPractice || (audioPracticeRef.current?.duration || 0);
        const totalDur = duration + practiceDur;
        const percent = totalDur > 0 ? Math.round((current / totalDur) * 100) : Math.round(ratio * 100);
        setProgress(Math.max(0, Math.min(100, percent)));
        setCurrentPart('general');
    };

    const onTimeUpdatePractice = () => {
        const el = audioPracticeRef.current;
        if (!el || !content) return;
        const duration = durPractice || el.duration || 0;
        const current = el.currentTime || 0;
        if (duration <= 0 || sentences.length === 0) return;
        const ratio = Math.max(0, Math.min(1, current / duration));
        const idx = Math.min(practiceSentences.length - 1, Math.floor(ratio * practiceSentences.length));
        const globalIdx = generalSentences.length + idx;
        setActiveIdx(globalIdx);
        const generalDur = durGeneral || (audioGeneralRef.current?.duration || 0);
        const totalDur = generalDur + duration;
        const percent = totalDur > 0 ? Math.round(((generalDur + current) / totalDur) * 100) : Math.round(ratio * 100);
        setProgress(Math.max(0, Math.min(100, percent)));
        setCurrentPart('practice');
    };

    // autoscroll to active sentence
    useEffect(() => {
        const node = linesRefs.current[activeIdx];
        if (node) node.scrollIntoView({ block: 'nearest' });
    }, [activeIdx]);

    // local hard toggle storage
    useEffect(() => {
        const k = meta ? lessonKey(meta) : '';
        if (!k) return;
        const raw = localStorage.getItem('jeels.hard.v1') || '[]';
        try {
            const arr = JSON.parse(raw) as string[];
            setIsHard(arr.includes(k));
        } catch {
            setIsHard(false);
        }
    }, [meta]);

    const toggleHard = () => {
        if (!meta) return;
        const k = lessonKey(meta);
        const raw = localStorage.getItem('jeels.hard.v1') || '[]';
        let arr: string[];
        try { arr = JSON.parse(raw) as string[]; } catch { arr = []; }
        if (arr.includes(k)) arr = arr.filter((x) => x !== k); else arr.push(k);
        localStorage.setItem('jeels.hard.v1', JSON.stringify(arr));
        setIsHard(arr.includes(k));
    };

    const markDone = () => {
        if (!meta) return;
        const k = lessonKey(meta);
        UserStore.removeLesson(k);
        // pick next automatically
        const next = pickCurrentLessonMeta();
        setMeta(next);
        setActiveIdx(0);
    };

    const skip = () => {
        // naive: move current key to end of the list
        if (!meta) return;
        const k = lessonKey(meta);
        const st = UserStore.getState();
        if (!st.learningLessonKeys.includes(k)) return;
        UserStore.removeLesson(k);
        UserStore.addLesson(k);
        const next = pickCurrentLessonMeta();
        setMeta(next);
        setActiveIdx(0);
    };

    const { clippy } = useClippy();
    const lastSpokenIdxRef = useRef<number>(-1);
    useEffect(() => {
        if (!clippy || sentences.length === 0) {
            return;
        }

        if (!isPlaying) {
            lastSpokenIdxRef.current = -1;
            return;
        }

        if (activeIdx === lastSpokenIdxRef.current) {
            return;
        }

        lastSpokenIdxRef.current = activeIdx;
        const text = sentences[activeIdx] ?? '';

        if (!text) {
            return;
        }
        try {
            clippy.stopCurrent?.();
            clippy.speak?.(text, false);
        } catch { }
    }, [activeIdx, sentences, clippy, isPlaying]);

    return (
        <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
            {!content && (
                <div style={{ color: '#555' }}>Выберите урок для просмотра.</div>
            )}

            {content && (
                <div style={{ display: 'grid', gridTemplateColumns: '1fr 2fr', gap: 16 }}>
                    {/* Left column: image only wrapped in Fieldset */}
                    <Fieldset legend="画像">
                        <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
                            <Frame variant="window" style={{ padding: 8 }}>
                                <img src={currentPart === 'general' ? content.general_img_path : content.practic_img_path} alt={content.topic} style={{ width: '100%', aspectRatio: '3/4', objectFit: 'cover' }} />
                            </Frame>
                            {/* hidden native audio element */}
                            <audio
                                ref={audioGeneralRef}
                                src={content.general_audio_path}
                                style={{ display: 'none' }}
                                onTimeUpdate={onTimeUpdateGeneral}
                                onLoadedMetadata={(e) => setDurGeneral((e.target as HTMLAudioElement).duration || 0)}
                                onPlay={() => setIsPlaying(true)}
                                onPause={() => setIsPlaying(false)}
                                onLoadedData={() => { audioGeneralRef.current?.play().catch(() => { }); setCurrentPart('general'); }}
                                onEnded={() => { setCurrentPart('practice'); audioPracticeRef.current?.play().catch(() => { }); }}
                            />
                            <audio
                                ref={audioPracticeRef}
                                src={content.practic_audio_path}
                                style={{ display: 'none' }}
                                onTimeUpdate={onTimeUpdatePractice}
                                onLoadedMetadata={(e) => setDurPractice((e.target as HTMLAudioElement).duration || 0)}
                                onPlay={() => setIsPlaying(true)}
                                onPause={() => setIsPlaying(false)}
                            />
                        </div>
                    </Fieldset>

                    {/* Right column: subtitles and actions wrapped in Fieldset */}
                    <Fieldset legend="テキスト">
                        <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
                            {/* Controls above text, wider */}
                            <Frame variant="window" style={{ padding: 8 }}>
                                <div style={{ display: 'flex', gap: 12, alignItems: 'center' }}>
                                    {!isPlaying ? (
                                        <Button onClick={() => { (currentPart === 'general' ? audioGeneralRef.current : audioPracticeRef.current)?.play(); }} style={{ width: '25%' }}>
                                            ▶️ 再生
                                        </Button>
                                    ) : (
                                        <Button onClick={() => { (currentPart === 'general' ? audioGeneralRef.current : audioPracticeRef.current)?.pause(); }} style={{ width: '25%' }}>
                                            ⏸️ 一時停止
                                        </Button>
                                    )}
                                    <ProgressBar percent={progress} />
                                </div>
                            </Frame>
                            <div style={{ maxHeight: 360, overflow: 'auto', background: 'var(--message-bg)', border: '1px solid var(--message-border)', padding: 12 }}>
                                {sentences.map((line, i) => (
                                    <div
                                        key={i}
                                        ref={(el) => { linesRefs.current[i] = el; }}
                                        style={{
                                            marginTop: i === 0 ? 0 : 8,
                                            background: i === activeIdx ? '#fff8c5' : 'transparent'
                                        }}
                                    >
                                        {line}
                                    </div>
                                ))}
                            </div>

                            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                                <div style={{ display: 'flex', gap: 8 }}>
                                    <Button onClick={() => setMeta(pickCurrentLessonMeta())}>🔙 戻る</Button>
                                    <Button onClick={toggleHard}>{isHard ? '🤯 難しい' : '🤯 難しいとしてマーク'}</Button>
                                </div>
                                <div style={{ display: 'flex', gap: 8 }}>
                                    <Button onClick={markDone}>✅ 視聴済み</Button>
                                    <Button onClick={skip}>⏭️ スキップ</Button>
                                </div>
                            </div>
                        </div>
                    </Fieldset>
                </div>
            )}

        </div>
    );
}


