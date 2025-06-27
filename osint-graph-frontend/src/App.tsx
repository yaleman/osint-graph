import {
	useCallback,
	useEffect,
	useState,
} from "react";

import type { MouseEvent as ReactMouseEvent } from "react";

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
import { NodeTypeMenu } from "./components/NodeTypeMenu";
import { v4 as uuidv4 } from "uuid";
import {
	forceCenter,
	forceCollide,
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

const RADIUS = 8;
const FORCE_RADIUS_FACTOR = 1.5;
// const LINK_WIDTH = 3;
// const LINK_DISTANCE = 30;
const NODE_STRENGTH = -50;
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
	const [showNodeMenu, setShowNodeMenu] = useState(false);
	const [menuPosition, setMenuPosition] = useState({ x: 0, y: 0 });

	const [nodes, setNodes, onNodesChange] = useNodesState(initialNodes);
	const [edges, setEdges, onEdgesChange] = useEdgesState(
		initialEdges.map((customEdge) => customEdge.edge),
	);

	const [_windowSize, setWindowSize] = useState({
		width: window.innerWidth,
		height: window.innerHeight,
	});


	// we use the useEffect hook to listen to the window resize event
	useEffect(() => {
		window.onresize = () => {
			setWindowSize({
				width: window.innerWidth,
				height: window.innerHeight,
			});
		};
	}, []);

	useEffect(() => {
		// Create the D3 simulation
		const simulation = forceSimulation(nodes as CustomNode[])
			.force(
				"link",
				forceLink<CustomNode, CustomEdge>()
					.id((node: CustomNode) => node.id)
					.distance((edge: CustomEdge) => {
						const source = nodes.find((n) => n.id === edge.source);
						const target = nodes.find((n) => n.id === edge.target);
						const distance = Math.sqrt(
							((source?.position?.x || 0) - (target?.position?.x || 0)) ^ 2 +
							((source?.position?.y || 0) - (target?.position?.y || 0)) ^ 2);
						return distance;
						// const horizDistance = (source?.position?.x || 0) - (target?.position?.x || 0);
						// const vertDistance = (source?.position?.y || 0) - (target?.position?.y || 0);
					})
					.strength(-0.01),
			)
			.force("charge", forceManyBody().strength(NODE_STRENGTH / 10))
			.force("collide", forceCollide(RADIUS + FORCE_RADIUS_FACTOR))
			.force("center", forceCenter(0, 0).strength(1))
			;

		// Update the node positions after each tick
		simulation.on("tick", () => {
			setNodes((theseNodes) =>
				theseNodes.map((thisNode) => {
					const node = simulation.nodes().find((n) => n.id === thisNode.id);
					// const node = simulation.find(el.position.x, el.position.y);
					if (node !== undefined) {
						if (node.x !== undefined && node.y !== undefined) {
							thisNode.position = { x: node.x, y: node.y };
						}
					}
					return thisNode;
				}),
			);
		});
		return () => {
			simulation.stop();
		};
	}, [nodes, setNodes]);

	// const onLoad = (reactFlowInstance: OnLoadParams) =>
	// 		setReactFlowInstance(reactFlowInstance);

	// handler for when a connection is made
	const onConnect: OnConnect = useCallback(
		(connection) => {
			setEdges((edges) => addEdge(connection, edges));
		},
		[setEdges],
	);

	// on startup, pull the project list
	useEffect(() => {
		updateProjects(setProjectNodes);
	}, []); // Empty array means this effect runs once on component mount

	// Handle background click to show node menu
	const onPaneClick = useCallback((event: ReactMouseEvent) => {
		console.debug('Pane clicked at:', event.clientX, event.clientY);
		setMenuPosition({
			x: event.clientX,
			y: event.clientY,
		});
		setShowNodeMenu(true);
	}, []);

	// Create new node of specified type at menu position
	const createNode = useCallback((nodeType: string) => {
		// Convert screen coordinates to ReactFlow coordinates
		const reactFlowBounds = document.querySelector('.react-flow')?.getBoundingClientRect();
		const x = reactFlowBounds ? menuPosition.x - reactFlowBounds.left - 60 : menuPosition.x - 60;
		const y = reactFlowBounds ? menuPosition.y - reactFlowBounds.top - 30 : menuPosition.y - 30;
		
		const newNode: CustomNode = {
			id: uuidv4(),
			type: nodeType,
			position: { x, y },
			data: {
				label: `New ${nodeType}`,
				content: "",
			},
		};
		setNodes((nodes) => [...nodes, newNode]);
		setShowNodeMenu(false);
	}, [menuPosition, setNodes]);

	// Close node menu
	const closeNodeMenu = useCallback(() => {
		setShowNodeMenu(false);
	}, []);

	// Close menu when clicking outside
	useEffect(() => {
		const handleClickOutside = (event: MouseEvent) => {
			if (showNodeMenu && !(event.target as HTMLElement).closest('.node-type-menu')) {
				setShowNodeMenu(false);
			}
		};

		document.addEventListener('mousedown', handleClickOutside);
		return () => {
			document.removeEventListener('mousedown', handleClickOutside);
		};
	}, [showNodeMenu]);

	return (
		<div style={{ position: 'relative', width: '100vw', height: '100vh' }}>
			<ReactFlow
				nodes={nodes}
				nodeTypes={nodeTypes}
				onNodeDragStop={(event: ReactMouseEvent, eventNode: CustomNode, eventNodes: CustomNode[]) => {
					console.debug("onNodeDragStop", event, eventNode, eventNodes);
					setNodes(nodes);
				}}
				onNodesChange={onNodesChange}
				edges={edges}
				edgeTypes={edgeTypes}
				onEdgesChange={onEdgesChange}
				onConnect={onConnect}
				onPaneClick={onPaneClick}
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
				{projectLis(projectNodes)}
			</Panel>
			</ReactFlow>
			{showNodeMenu && (
				<NodeTypeMenu
					x={menuPosition.x}
					y={menuPosition.y}
					onSelect={createNode}
					onClose={closeNodeMenu}
				/>
			)}
		</div>
	);
}
