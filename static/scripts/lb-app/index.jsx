import { StrictMode } from 'react';
import { createRoot } from "react-dom/client";
import { LetterboxdApp } from "./lb-app-react";

const guestbookRoot = createRoot(document.getElementById("react-root"));
guestbookRoot.render(<StrictMode><LetterboxdApp /></StrictMode>);