import type { OnConnect } from "reactflow";

import { useCallback, useEffect, useState } from "react";
import {
	Background,
	Controls,
	MiniMap,
	ReactFlow,
	addEdge,
	useNodesState,
	useEdgesState,
	Panel,
} from "reactflow";

import "reactflow/dist/style.css";

import { initialNodes, nodeTypes } from "./nodes";
import { initialEdges, edgeTypes } from "./edges";
import { fetchProjects, newProject } from "./api";
import type { Project } from "./types";

function updateProjects(
	setProjectNodes: React.Dispatch<React.SetStateAction<Project[]>>,
) {
	fetchProjects().then((projects) => {
		console.log(projects);
		setProjectNodes(projects);
	});
}

/** Takes the project list and sends back a list */
function listProjects(projects: Project[]) {
	return (
		<ul>
			{projects.map((project) => {
				return <li key={project.id}>{project.name}</li>;
			})}
		</ul>
	);
}

export default function App() {
	const [projectNodes, setProjectNodes] = useState<Project[]>([]);
	const [nodes, , onNodesChange] = useNodesState(initialNodes);
	const [edges, setEdges, onEdgesChange] = useEdgesState(initialEdges);
	const onConnect: OnConnect = useCallback(
		(connection) => setEdges((edges) => addEdge(connection, edges)),
		[setEdges],
	);
	useEffect(() => {
		updateProjects(setProjectNodes);
	}, []); // Empty array means this effect runs once on component mount

	return (
		<ReactFlow
			nodes={nodes}
			nodeTypes={nodeTypes}
			onNodesChange={onNodesChange}
			edges={edges}
			edgeTypes={edgeTypes}
			onEdgesChange={onEdgesChange}
			onConnect={onConnect}
			fitView
		>
			<Background />
			<MiniMap />
			<Controls />
			<Panel position="top-right">
				<button
					type="button"
					onClick={() => {
						updateProjects(setProjectNodes);
					}}
				>
					Get Projects
				</button>
				<button
					type="button"
					onClick={() => {
						newProject().then(() => {
							updateProjects(setProjectNodes);
						});
					}}
				>
					New Project
				</button>
				{listProjects(projectNodes)}
			</Panel>
		</ReactFlow>
	);
}
