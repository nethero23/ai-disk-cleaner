import React from "react";
import ReactDOM from "react-dom/client";
import { App } from "./App";
import "./ui";
import "./styles.css";
import { applyTheme, loadTheme } from "./theme";

// Before the first paint, so a saved dark theme does not flash light.
applyTheme(loadTheme());

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
