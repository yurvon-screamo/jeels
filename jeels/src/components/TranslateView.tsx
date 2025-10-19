import { useEffect, useRef, useState } from 'react';
import { Button, Fieldset, Frame, TextArea } from '@react95/core';

type LanguageCode = 'rus_Cyrl' | 'jpn_Jpan';

type TranslateTurn = {
    id: string;
    createdAt: number;
    sourceText: string;
    sourceLang: LanguageCode;
    targetText: string;
    targetLang: LanguageCode;
};

const STORAGE_KEY = 'jeels.translate.history.v1';

function loadHistory(): TranslateTurn[] {
    try {
        const raw = localStorage.getItem(STORAGE_KEY);
        if (!raw) return [];
        const parsed = JSON.parse(raw) as TranslateTurn[];
        if (!Array.isArray(parsed)) return [];
        return parsed.filter(Boolean);
    } catch {
        return [];
    }
}

function saveHistory(turns: TranslateTurn[]) {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(turns));
}

function detectLang(text: string): LanguageCode {
    const t = text || '';
    const hasJapanese = /[\u3040-\u30ff\u3400-\u4dbf\u4e00-\u9fff]/.test(t);
    const hasCyrillic = /[\u0400-\u04FF]/.test(t);
    if (hasJapanese && !hasCyrillic) return 'jpn_Jpan';
    if (hasCyrillic && !hasJapanese) return 'rus_Cyrl';
    // fallback: prefer Japanese if any kana/kanji found, else Russian
    return hasJapanese ? 'jpn_Jpan' : 'rus_Cyrl';
}

// Lazy singleton translator instance
let translatorPromise: Promise<any> | null = null;
async function getTranslator() {
    if (!translatorPromise) {
        translatorPromise = (async () => {
            const { pipeline } = await import('@huggingface/transformers');
            const trans = await pipeline('translation', 'Xenova/nllb-200-distilled-600M');
            return trans;
        })();
    }
    return translatorPromise;
}

export function TranslateView() {
    const [inputText, setInputText] = useState('');
    const [turns, setTurns] = useState<TranslateTurn[]>(() => loadHistory());
    const [isLoadingModel, setIsLoadingModel] = useState(false);
    const [isTranslating, setIsTranslating] = useState(false);
    const [editingTurnId, setEditingTurnId] = useState<string | null>(null);
    const [editingText, setEditingText] = useState('');
    const listRef = useRef<HTMLDivElement | null>(null);
    const [spinnerTick, setSpinnerTick] = useState(0);

    useEffect(() => {
        saveHistory(turns);
    }, [turns]);

    useEffect(() => {
        const el = listRef.current;
        if (el) el.scrollTop = el.scrollHeight;
    }, [turns.length]);

    const isBusy = isLoadingModel || isTranslating;

    // simple text-based spinner (avoids custom CSS)
    useEffect(() => {
        if (!isBusy) return;
        const id = setInterval(() => setSpinnerTick((n) => (n + 1) % 2), 400);
        return () => clearInterval(id);
    }, [isBusy]);
    const spinnerGlyph = spinnerTick === 0 ? '⏳' : '⌛';

    async function translate(text: string): Promise<{ outText: string; src: LanguageCode; tgt: LanguageCode }> {
        const sourceLang = detectLang(text);
        const targetLang: LanguageCode = sourceLang === 'jpn_Jpan' ? 'rus_Cyrl' : 'jpn_Jpan';

        setIsLoadingModel(true);
        try {
            const translator = await getTranslator();
            setIsLoadingModel(false);
            setIsTranslating(true);
            const res = await translator(text, { src_lang: sourceLang, tgt_lang: targetLang });
            const out = Array.isArray(res) && res[0] && res[0].translation_text ? String(res[0].translation_text) : '';
            return { outText: out, src: sourceLang, tgt: targetLang };
        } finally {
            setIsLoadingModel(false);
            setIsTranslating(false);
        }
    }

    const onSend = async () => {
        const raw = inputText.trim();
        if (!raw || isBusy) return;
        setInputText('');
        const { outText, src, tgt } = await translate(raw);
        const newTurn: TranslateTurn = {
            id: `t-${Date.now()}-${Math.random().toString(36).slice(2)}`,
            createdAt: Date.now(),
            sourceText: raw,
            sourceLang: src,
            targetText: outText,
            targetLang: tgt,
        };
        setTurns((arr) => [...arr, newTurn]);
    };

    const onEditStart = (turn: TranslateTurn) => {
        setEditingTurnId(turn.id);
        setEditingText(turn.sourceText);
    };

    const onEditCancel = () => {
        setEditingTurnId(null);
        setEditingText('');
    };

    const onEditConfirm = async () => {
        const id = editingTurnId;
        const newSource = editingText.trim();
        if (!id || !newSource) return onEditCancel();
        const { outText, src, tgt } = await translate(newSource);
        setTurns((arr) => arr.map((t) => (t.id === id ? { ...t, sourceText: newSource, sourceLang: src, targetText: outText, targetLang: tgt } : t)));
        onEditCancel();
    };

    const onDelete = (id: string) => {
        setTurns((arr) => arr.filter((t) => t.id !== id));
    };

    const onBurn = () => {
        if (isBusy) return;
        setTurns([]);
    };

    const legend = '翻訳';

    const headerBar = (
        <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
            <div style={{ flex: 1, minWidth: 0 }}>
                {isLoadingModel ? (
                    <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                        <span aria-busy="true" aria-label="loading">{spinnerGlyph}</span>
                        <span>モデルを読み込み中…</span>
                    </div>
                ) : isTranslating ? (
                    <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                        <span aria-busy="true" aria-label="translating">{spinnerGlyph}</span>
                        <span>翻訳中…</span>
                    </div>
                ) : (
                    <span style={{ color: '#555' }}>露⇄日 自動判定</span>
                )}
            </div>
            <Button onClick={onBurn} disabled={isBusy || turns.length === 0}>🔥</Button>
        </div>
    );

    return (
        <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
            <Fieldset legend={legend}>
                <div style={{ display: 'grid', gridTemplateRows: 'auto 1fr auto', gap: 12, height: 580 }}>
                    <Frame variant="window" style={{ padding: 8 }}>
                        {headerBar}
                    </Frame>

                    <div ref={listRef} style={{ maxHeight: 420, overflow: 'auto', background: 'var(--message-bg)', border: '1px solid var(--message-border)', padding: 12 }}>
                        {turns.length === 0 && (
                            <div style={{ color: '#555' }}>ここに翻訳履歴が表示されます。</div>
                        )}
                        {turns.map((t) => (
                            <div key={t.id} style={{ display: 'flex', flexDirection: 'column', gap: 6, marginBottom: 12 }}>
                                <Frame variant="window" style={{ padding: 8, background: '#fff' }}>
                                    <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 6 }}>
                                        <div>
                                            <b>入力</b> <span style={{ color: '#555' }}>({t.sourceLang === 'jpn_Jpan' ? '日本語' : 'ロシア語'})</span>
                                        </div>
                                        <div style={{ display: 'flex', gap: 8 }}>
                                            <Button size="sm" onClick={() => onEditStart(t)} disabled={isBusy}>編集</Button>
                                            <Button size="sm" onClick={() => onDelete(t.id)} disabled={isBusy}>削除</Button>
                                        </div>
                                    </div>
                                    {editingTurnId === t.id ? (
                                        <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
                                            <TextArea value={editingText} onChange={(e: any) => setEditingText(e.target.value)} rows={4} />
                                            <div style={{ display: 'flex', gap: 8 }}>
                                                <Button onClick={onEditConfirm} disabled={isBusy || !editingText.trim()}>保存</Button>
                                                <Button onClick={onEditCancel} disabled={isBusy}>キャンセル</Button>
                                            </div>
                                        </div>
                                    ) : (
                                        <div style={{ whiteSpace: 'pre-wrap' }}>{t.sourceText}</div>
                                    )}
                                </Frame>
                                <Frame variant="window" style={{ padding: 8, background: '#fff' }}>
                                    <div style={{ marginBottom: 6 }}>
                                        <b>翻訳</b> <span style={{ color: '#555' }}>({t.targetLang === 'jpn_Jpan' ? '日本語' : 'ロシア語'})</span>
                                    </div>
                                    <div style={{ whiteSpace: 'pre-wrap' }}>{t.targetText}</div>
                                </Frame>
                            </div>
                        ))}
                    </div>

                    <Frame variant="window" style={{ padding: 8 }}>
                        <div style={{ display: 'flex', gap: 8, alignItems: 'stretch' }}>
                            <TextArea
                                value={inputText}
                                onChange={(e: any) => setInputText(e.target.value)}
                                placeholder="テキストを入力して翻訳…（日本語/ロシア語 自動）"
                                rows={3}
                                style={{ flex: 1 }}
                            />
                            <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
                                <Button onClick={onSend} disabled={isBusy || inputText.trim().length === 0}>送信</Button>
                            </div>
                        </div>
                    </Frame>
                </div>
            </Fieldset>
        </div>
    );
}

export default TranslateView;
