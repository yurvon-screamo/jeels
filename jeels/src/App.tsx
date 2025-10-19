import { Fieldset, ProgressBar, TitleBar, Modal } from "@react95/core";
import { useModal } from "@react95/core";
import { useEffect, useState } from "react";
import "./App.css";
import { Layout } from "./components/Layout";
import { useStartMenu, Page } from "./components/Router";
import { FeedView } from "./components/FeedView";
import { LessonsView } from "./components/LessonsView";
import { TranslateView } from "./components/TranslateView";
import { LessonStore } from "./store";

const views: Record<Page, { title: string; node: React.ReactNode }> = {
  feed: { title: "フィード", node: <FeedView /> },
  lessons: { title: "レッスン", node: <LessonsView /> },
  translate: {
    title: "翻訳", node: (
      <TranslateView />
    )
  },
  profile: {
    title: "プロフィール", node: (
      <Fieldset legend="プロフィール">
        <p>ユーザープロフィール（開発中）。</p>
      </Fieldset>
    )
  }
};

function App() {
  const [isIndexReady, setIsIndexReady] = useState(false);
  const [nextId, setNextId] = useState(1);
  const [openWindows, setOpenWindows] = useState<{ id: string; type: Page }[]>([]);
  const { remove } = useModal();
  const closeWindow = (id: string) => {
    remove(id);
    setOpenWindows((arr) => arr.filter((w) => w.id !== id));
  };
  const openWindow = (type: Page) => {
    const id = `${type}-${nextId}`;
    setNextId((n) => n + 1);
    setOpenWindows((arr) => [...arr, { id, type }]);
  };
  // Maximize is disabled per UX decision; only minimize/close are available.
  const menu = useStartMenu(openWindow);

  useEffect(() => {
    LessonStore.ensureIndexLoaded().finally(() => setIsIndexReady(true));
  }, []);
  useEffect(() => {
    if (!isIndexReady) return;
    // Открываем одно окно Ленты по умолчанию
    if (openWindows.length === 0) openWindow('feed');
  }, [isIndexReady]);

  return (
    <Layout title="Jeels" startMenu={menu}>
      <div style={{ display: 'flex', flexDirection: 'column', gap: 20 }}>
        {!isIndexReady ? (
          <Fieldset legend="読み込み中">
            <div style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
              <div style={{ flex: 1, minWidth: 0 }}>
                <ProgressBar percent={66} />
              </div>
              <span>コンテンツを読み込んでいます…</span>
            </div>
          </Fieldset>
        ) : (
          <>
            {openWindows.map(({ id, type }) => {
              const ModalAny: any = Modal;
              const index = parseInt(id.split('-')[1]);
              const baseTop = 15 + (isNaN(index) ? 0 : (index - 1) * 20);
              const baseLeft = 15 + (isNaN(index) ? 0 : (index - 1) * 20);
              const modalStyle = { position: 'absolute', top: baseTop, left: baseLeft, width: 1375, height: 770, resize: 'both' as const, overflow: 'hidden', maxWidth: '100%', maxHeight: '100%' };
              const title = type === 'feed' ? (() => {
                // Try to extract current lesson topic from FeedView state via LessonStore
                const st = LessonStore.getState();
                // Use first selected lesson or first index
                const key = st.index[0] ? `${st.index[0].group}/${st.index[0].topic}` : '';
                const [, ...topicParts] = key.split('/');
                const topic = topicParts.join('/') || views[type].title;
                return topic;
              })() : views[type].title;
              return (
                <ModalAny
                  key={id}
                  id={id}
                  title={title}
                  hasWindowButton
                  style={modalStyle}
                  titleBarOptions={
                    <TitleBar.OptionsBox>
                      <ModalAny.Minimize />
                      <TitleBar.Close onClick={() => closeWindow(id)} />
                    </TitleBar.OptionsBox>
                  }
                >
                  <div style={{ padding: 16, height: '100%', overflow: 'auto' }}>
                    {views[type].node}
                  </div>
                </ModalAny>
              );
            })}
          </>
        )}
      </div>
    </Layout>
  );
}

export default App;
