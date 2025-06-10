import { StrictMode } from 'react';
import { createRoot } from "react-dom/client";
import HitCounter from "./hit-counter.tsx";
import ReactLogoMessage from './react-logo-msg.tsx';

const hitCounterRoot = createRoot(document.getElementById("react-root-hit-counter"));
const reactLogoRoot  = createRoot(document.getElementById("react-logo-container"))
hitCounterRoot.render(<StrictMode><HitCounter /></StrictMode>);
reactLogoRoot.render(<StrictMode><ReactLogoMessage /></StrictMode>);