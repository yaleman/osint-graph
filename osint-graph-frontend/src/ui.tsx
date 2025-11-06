import type React from "react";
import { v4 as uuidv4 } from "uuid";
import type { CustomNode } from "./nodes";

export function newNode(
	nodes: CustomNode[],
	setNodes: React.Dispatch<React.SetStateAction<CustomNode[]>>,
) {
	console.debug("New node button pressed");

	const newNode: CustomNode = {
		id: uuidv4(),
		data: { label: "Hello" },
		position: { x: 0, y: 0 },
		type: "default",
	};
	setNodes([...nodes, newNode]);
}
