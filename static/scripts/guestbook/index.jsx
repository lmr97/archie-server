import { StrictMode } from 'react';
import { createRoot } from "react-dom/client";
import GuestbookApp from "./guestbook-react";

const guestbookRoot = createRoot(document.getElementById("react-root"));
guestbookRoot.render(<StrictMode><GuestbookApp /></StrictMode>);