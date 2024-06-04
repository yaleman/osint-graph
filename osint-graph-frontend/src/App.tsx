import { useCallback, useEffect, useState } from "react";
import {
	Background,
	Controls,
	MiniMap,
	ReactFlow,
	useNodesState,
	useEdgesState,
	Panel,
	addEdge,
	type OnConnect,
} from "reactflow";

import "reactflow/dist/style.css";

import { initialNodes, nodeTypes } from "./nodes";
import { initialEdges, edgeTypes } from "./edges";
import { fetchProjects, projectLis, newProject } from "./api";
import type { Project } from "./types";
import { newNode } from "./ui";

function updateProjects(
	setProjectNodes: React.Dispatch<React.SetStateAction<Project[]>>,
) {
	fetchProjects().then((projects) => {
		console.log(projects);
		setProjectNodes(projects);
	});
}

// const RADIUS = 8;
// const LINK_WIDTH = 3;
// const LINK_DISTANCE = 30;
// const FORCE_RADIUS_FACTOR = 1.5;
// const NODE_STRENGTH = -50;
// const width = 800;
// const height = 600;

export default function App() {
	const [projectNodes, setProjectNodes] = useState<Project[]>([]);

	// // below should be in your component
	// const newLinks = useMemo(() => {
	// 	const sources = initLinks.map((link) => link.source);
	// 	const targets = initLinks.map((link) => link.target);
	// 	const nodesMap = d3Map(initNodes, getId, (d) => d);

	// 	const newLinks = initLinks.map((_, i) => ({
	// 		source: nodesMap.get(sources[i]),
	// 		target: nodesMap.get(targets[i]),
	// 		strength: _.strength,
	// 	}));
	// 	return newLinks;
	// }, [initNodes]);

	const [nodes, setNodes, onNodesChange] = useNodesState(initialNodes);
	const [edges, setEdges, onEdgesChange] = useEdgesState(initialEdges);

	// const { _formattedEdges, _formattedNodes } = simulateGraph({
	// 	edges,
	// 	nodes,
	// });

	// useEffect(() => {
	// 	const simulation = forceSimulation<CustomNode, CustomLink>(initNodes)
	// 		.force(
	// 			"link",
	// 			forceLink<CustomNode, CustomLink>(newLinks)
	// 				.id((d) => d.id)
	// 				.distance(LINK_DISTANCE),
	// 		)
	// 		.force("center", forceCenter(width / 2, height / 2).strength(0.05))
	// 		.force("charge", forceManyBody().strength(NODE_STRENGTH))
	// 		.force("collision", forceCollide(RADIUS * FORCE_RADIUS_FACTOR));

	// 	// update state on every frame
	// 	simulation.on("tick", () => {
	// 		setNodes([...simulation.nodes()]);
	// 		setLinks([...initLinks]);
	// 	});

	// 	return () => {
	// 		simulation.stop();
	// 	};
	// }, [width, height, newLinks, initNodes]);

	// handler for when a connection is made
	const onConnect: OnConnect = useCallback(
		(connection) => setEdges((edges) => addEdge(connection, edges)),
		[setEdges],
	);

	// on startup, pull the project list
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
						newNode(nodes, setNodes);
					}}
				>
					New Node
				</button>
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
				{projectLis(projectNodes)}
			</Panel>
		</ReactFlow>
	);
}
