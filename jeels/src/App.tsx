import { useState } from "react";
import { Button, Fieldset, Input } from "@react95/core";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";
import { Layout } from "./components/Layout";
import { Router } from "./components/Router";
import { LessonsView } from "./components/LessonsView";

type Page = "feed" | "lessons" | "translate" | "profile";

const views: Record<Page, React.ReactNode> = {
  feed: (
    <Fieldset legend="Feed">
      <p>Главная лента с рекомендациями уроков.</p>
    </Fieldset>
  ),
  lessons: (
    <LessonsView />
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
      </div>
    </Layout>
  );
}

export default App;
