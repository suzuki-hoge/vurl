import React from "react"
import ReactDOM from "react-dom/client"
import { Toaster } from "react-hot-toast"

import { App } from "./app/App"

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <Toaster
      position="bottom-right"
      toastOptions={{
        duration: 4000,
        style: {
          borderRadius: "0.95rem",
          background: "#ffffff",
          color: "#111722",
          border: "1px solid rgba(125, 145, 175, 0.28)",
          boxShadow: "0 18px 48px rgba(0, 0, 0, 0.35)",
          backdropFilter: "blur(16px)"
        },
        success: {
          iconTheme: {
            primary: "#32c26a",
            secondary: "#ffffff"
          }
        },
        error: {
          iconTheme: {
            primary: "#ff6b6b",
            secondary: "#ffffff"
          }
        }
      }}
    />
    <App />
  </React.StrictMode>
)
