import type { Node, NodeTypes } from "reactflow";
import { PositionLoggerNode } from "./PositionLoggerNode";

export interface CustomNode extends Node {
	id: string;
}

export const initialNodes: CustomNode[] = [];

export const nodeTypes = {
	"position-logger": PositionLoggerNode,
} satisfies NodeTypes;
