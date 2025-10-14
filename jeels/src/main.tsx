import ReactDOM from "react-dom/client";
import App from "./App";
import '@react95/core/GlobalStyle';
import '@react95/core/themes/win95.css';
import '@react95/icons/icons.css';
import { ClippyProvider } from '@react95/clippy';

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <ClippyProvider>
    <App />
  </ClippyProvider>,
);
