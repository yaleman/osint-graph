import React from "react";
import ReactDOM from "react-dom/client";

import App from "./App";

import "./index.css";

ReactDOM.createRoot(
	// biome-ignore lint/style/noNonNullAssertion: if this doesn't work we're in trouble
	document.getElementById("root")!,
).render(
	<React.StrictMode>
		<App />
	</React.StrictMode>,
);
