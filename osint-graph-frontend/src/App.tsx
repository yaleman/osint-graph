import {
	useCallback,
	useEffect,
	// useMemo,
	useState,
} from "react";

import {
	Background,
	Controls,
	MiniMap,
	ReactFlow,
	Panel,
	addEdge,
	type OnConnect,
	useNodesState,
	useEdgesState,
} from "reactflow";

import "reactflow/dist/style.css";

import { type CustomNode, initialNodes, nodeTypes } from "./nodes";
import { type CustomEdge, initialEdges, edgeTypes } from "./edges";
import { fetchProjects, projectLis, newProject } from "./api";
import type { Project } from "./types";
import { newNode } from "./ui";
import {
	forceCenter,
	forceLink,
	forceManyBody,
	forceSimulation,
} from "d3-force";

function updateProjects(
	setProjectNodes: React.Dispatch<React.SetStateAction<Project[]>>,
) {
	fetchProjects().then((projects) => {
		console.debug("projects loaded", projects);
		setProjectNodes(projects);
	});
}

// const RADIUS = 8;
// const FORCE_RADIUS_FACTOR = 1.5;
// const LINK_WIDTH = 3;
// const LINK_DISTANCE = 30;
// const NODE_STRENGTH = -50;
// const width = 800;
// const height = 600;

// const getId = (node: CustomNode) => node.id;

// function d3Map<T, U>(
// 	data: T[],
// 	keyAccessor: (datum: T) => string,
// 	valueAccessor: (datum: T) => U,
// ): Map<string, U> {
// 	return new Map(
// 		Array.from(data, (datum) => [keyAccessor(datum), valueAccessor(datum)]),
// 	);
// }

// export declare const useNodesState: <NodeData = any>(
// 	initialItems: CustomNode[],
// ) => [
// 	CustomNode<NodeData, string | undefined>[],
// 	Dispatch<SetStateAction<Node<NodeData, string | undefined>[]>>,
// 	OnChange<NodeChange>,
// ];
// export declare const useEdgesState: <EdgeData = any>(
// 	initialItems: CustomEdge[],
// ) => [
// 	CustomEdge<EdgeData>[],
// 	Dispatch<SetStateAction<Edge<EdgeData>[]>>,
// 	OnChange<EdgeChange>,
// ];

export default function App() {
	const [projectNodes, setProjectNodes] = useState<Project[]>([]);

	// below should be in your component
	// const newLinks = useMemo(() => {
	// 	const sources = initialEdges.map((link) => link.edge.source);
	// 	const targets = initialEdges.map((link) => link.edge.target);
	// 	const nodesMap = d3Map(initialNodes, getId, (d) => d.id);

	// 	const newLinks = initialEdges.map((_, i) => ({
	// 		source: nodesMap.get(sources[i]),
	// 		target: nodesMap.get(targets[i]),
	// 		strength: _.strength,
	// 	}));
	// 	return newLinks;
	// }, []);

	const [nodes, setNodes, onNodesChange] = useNodesState(initialNodes);
	const [edges, setEdges, onEdgesChange] = useEdgesState(
		initialEdges.map((customEdge) => customEdge.edge),
	);

	// const [reactFlowInstance, _setReactFlowInstance] = useState();

	// const { _formattedEdges, _formattedNodes } = simulateGraph({
	// 	edges,
	// 	nodes,
	// });

	useEffect(() => {
		// if (!reactFlowInstance) return;
		// Create the D3 simulation
		const simulation = forceSimulation(nodes as CustomNode[])
			.force(
				"link",
				forceLink<CustomNode, CustomEdge>()
					.id((node: CustomNode) => node.id)
					.distance(0.1)
					.strength(0.1),
			)
			.force("charge", forceManyBody())
			.force("center", forceCenter(0, 0));

		// Update the node positions after each tick
		simulation.on("tick", () => {
			setNodes((els) =>
				els.map((el) => {
					const node = simulation.find(el.position.x, el.position.y);
					if (node) {
						el.position = { x: node.x || 0, y: node.y || 0.0 };
					}
					return el;
				}),
			);
		});
	}, [nodes, setNodes]);

	// const onLoad = (reactFlowInstance: OnLoadParams) =>
	// 		setReactFlowInstance(reactFlowInstance);

	// useEffect(() => {
	// 	const simulation = forceSimulation<CustomNode, CustomEdge>(nodes)
	// 		// .force(
	// 		// 	"link",
	// 		// 	forceLink<CustomNode, CustomEdge>(newLinks)
	// 		// 		.strength(0.1)
	// 		// 		.id((d) => d.id)
	// 		// 		.distance(LINK_DISTANCE),
	// 		// )
	// 		.force("center", forceCenter(width / 2, height / 2).strength(0.05))
	// 		.force("charge", forceManyBody().strength(NODE_STRENGTH))
	// 		.force("collision", forceCollide(RADIUS * FORCE_RADIUS_FACTOR));

	// 	// update state on every frame
	// 	simulation.on("tick", () => {
	// 		setNodes([...simulation.nodes()]);
	// 		setEdges([...initialEdges.map((customEdge) => customEdge.edge)]);
	// 	});

	// 	return () => {
	// 		simulation.stop();
	// 	};
	// }, [nodes, setNodes, setEdges]);

	// handler for when a connection is made
	const onConnect: OnConnect = useCallback(
		(connection) => {
			setEdges((edges) => addEdge(connection, edges));
		},
		[setEdges],
	);

	// on startup, pull the project list
	useEffect(() => {
		// const reactFlow = useReactFlow();
		// console.info("viewport", reactFlow.getViewport());
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
