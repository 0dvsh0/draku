import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";

// Offline-first: fonts ship in the binary via @fontsource, no CDN.
import "@fontsource-variable/inter-tight";
import "@fontsource-variable/jetbrains-mono";
import "@fontsource/instrument-serif/400-italic.css";
import "./styles/tokens.css";
import "./styles/app.css";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
