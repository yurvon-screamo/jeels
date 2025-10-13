import { useState } from "react";
import { Button, Fieldset, Input } from "@react95/core";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";
import { Layout } from "./components/Layout";
import { Router } from "./components/Router";

type Page = "feed" | "lessons" | "translate" | "profile";

const views: Record<Page, React.ReactNode> = {
  feed: (
    <Fieldset legend="Feed">
      <p>Главная лента с рекомендациями уроков.</p>
    </Fieldset>
  ),
  lessons: (
    <Fieldset legend="Lessons">
      <p>Дерево уроков, выбор и поиск.</p>
    </Fieldset>
  ),
  translate: (
    <Fieldset legend="Translate">
      <p>Двусторонний переводчик (rus ⇄ jap).</p>
    </Fieldset>
  ),
  profile: (
    <Fieldset legend="Profile">
      <p>Профиль пользователя (в разработке).</p>
    </Fieldset>
  )
};

function App() {
  const { current, menu } = Router({
    views: {
      feed: { title: "Лента", node: views.feed },
      lessons: { title: "Уроки", node: views.lessons },
      translate: { title: "Переводчик", node: views.translate },
      profile: { title: "Профиль", node: views.profile }
    }
  });
  const [greetMsg, setGreetMsg] = useState("");
  const [name, setName] = useState("");

  async function greet() {
    setGreetMsg(await invoke("greet", { name }));
  }

  return (
    <Layout title={current.title} startMenu={menu}>
      <div style={{ display: 'flex', flexDirection: 'column', gap: 20 }}>
        {current.node}

        <Fieldset legend="Greet Function">
          <div style={{ display: 'flex', flexDirection: 'column', gap: 10 }}>
            <Input
              value={name}
              onChange={(e: React.ChangeEvent<HTMLInputElement>) => setName(e.target.value)}
              placeholder="Enter a name..."
            />
            <Button onClick={greet} disabled={!name.trim()}>Greet</Button>
            {greetMsg && (
              <div style={{
                padding: 10,
                background: 'var(--message-bg)',
                border: '1px solid var(--message-border)'
              }}>
                {greetMsg}
              </div>
            )}
          </div>
        </Fieldset>
      </div>
    </Layout>
  );
}

export default App;
