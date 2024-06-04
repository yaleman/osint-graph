import type { Node } from "reactflow";

import { v4 as uuidv4 } from "uuid";

export function newNode(
	nodes: Node[],
	setNodes: React.Dispatch<React.SetStateAction<Node[]>>,
) {
	console.log("New node button pressed");

	const newNode = {
		id: uuidv4(),
		data: { label: "Hello" },
		position: { x: 0, y: 0 },
		type: "default",
	};
	setNodes([...nodes, newNode]);
}
