import React from "react";
import ReactDOM from "react-dom/client";

import App from "./App";

import "./index.css";

const rootElement = document.getElementById("root");
if (!rootElement) {
	alert("Critical Error: Failed to find the root element. The application cannot start.");
	throw new Error("Failed to find the root element");
}

ReactDOM.createRoot(rootElement,
).render(
	<React.StrictMode>
		<App />
	</React.StrictMode>,
);
