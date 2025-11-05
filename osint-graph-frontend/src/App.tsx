import type React from "react";
import { useCallback, useEffect, useId, useRef, useState } from "react";
import ReactFlow, {
	addEdge,
	Background,
	Controls,
	type Edge,
	MiniMap,
	type Node,
	type OnConnect,
	type OnEdgesChange,
	type OnNodesChange,
	ReactFlowProvider,
	useEdgesState,
	useNodesState,
	useReactFlow,
} from "reactflow";
import "reactflow/dist/style.css";
import toast, { Toaster } from "react-hot-toast";
import { v4 as uuidv4 } from "uuid";
import {
	createNode,
	createNodeLink,
	createProject,
	deleteAttachment,
	deleteNode,
	deleteNodeLink,
	downloadAttachment,
	exportProject,
	fetchProjects,
	listAttachments,
	updateAttachment,
	updateNode,
	uploadAttachment,
} from "./api";
import { ProjectManagementDialog } from "./components/ProjectManagementDialog";
import { ProjectMismatchDialog } from "./components/ProjectMismatchDialog";
import { ProjectSelector } from "./components/ProjectSelector";
import type { Attachment, OSINTNode, Project } from "./types";
import { NodeTypeInfo } from "./types";
import "./osint-graph.css";

const initialNodes: Node[] = [];
const initialEdges: Edge[] = [];

const PROJECT_ID_KEY = "osint-graph-project-id";
const DEBOUNCE_DELAY = 100; // ms

function AppContent() {
	const [nodes, setNodes, onNodesChange] = useNodesState(initialNodes);
	const [edges, setEdges, onEdgesChange] = useEdgesState(initialEdges);
	const { project } = useReactFlow();
	const [isPanelCollapsed, setIsPanelCollapsed] = useState(false);
	const [editingNode, setEditingNode] = useState<string | null>(null);
	const [editingNodeType, setEditingNodeType] = useState<string | null>(null);
	const [editDisplay, setEditDisplay] = useState("");
	const [editValue, setEditValue] = useState("");
	const [editNotes, setEditNotes] = useState("");
	const [currentProject, setCurrentProject] = useState<Project | null>(null);
	const [isLoading, setIsLoading] = useState(true);
	const [showMismatchDialog, setShowMismatchDialog] = useState(false);
	const [availableProjects, setAvailableProjects] = useState<Project[]>([]);
	const [showProjectManagement, setShowProjectManagement] = useState(false);
	const [pendingNodes, setPendingNodes] = useState<Set<string>>(new Set());
	const [contextMenu, setContextMenu] = useState<{
		x: number;
		y: number;
		node: Node;
	} | null>(null);
	const [nodeAttachments, setNodeAttachments] = useState<Attachment[]>([]);
	const [uploadingAttachment, setUploadingAttachment] = useState(false);
	const [allAttachments, setAllAttachments] = useState<Attachment[]>([]);
	const [movingAttachment, setMovingAttachment] = useState<{
		attachmentId: string;
		filename: string;
		sourceNodeId: string;
	} | null>(null);

	// Refs for debouncing node updates
	const pendingUpdatesRef = useRef<Map<string, number>>(new Map());
	const latestNodeDataRef = useRef<Map<string, OSINTNode>>(new Map());

	// History state for undo/redo (max 10 levels)
	const [history, setHistory] = useState<{
		past: Array<{ nodes: Node[]; edges: Edge[] }>;
		future: Array<{ nodes: Node[]; edges: Edge[] }>;
	}>({
		past: [],
		future: [],
	});
	const isUndoingRef = useRef(false);

	const idMoreInfo = useId();
	const idDisplay = useId();
	const idValue = useId();

	// Flush all pending updates immediately
	const flushPendingUpdates = useCallback(() => {
		const updates = Array.from(latestNodeDataRef.current.entries());

		updates.forEach(([nodeId, nodeData]) => {
			// Clear the timeout
			const timeout = pendingUpdatesRef.current.get(nodeId);
			if (timeout) {
				clearTimeout(timeout);
				pendingUpdatesRef.current.delete(nodeId);
			}

			// Execute the update
			updateNode(nodeData).catch((error) => {
				console.error("Failed to update node:", error);
				toast.error("Failed to update node");
			});
		});

		// Clear the latest data map
		latestNodeDataRef.current.clear();
	}, []);

	// Debounced update function
	const debouncedUpdateNode = useCallback((nodeData: OSINTNode) => {
		const nodeId = nodeData.id;

		// Store the latest data for this node
		latestNodeDataRef.current.set(nodeId, nodeData);

		// Clear existing timeout for this node
		const existingTimeout = pendingUpdatesRef.current.get(nodeId);
		if (existingTimeout) {
			clearTimeout(existingTimeout);
		}

		// Set new timeout
		const timeout = setTimeout(() => {
			const latestData = latestNodeDataRef.current.get(nodeId);
			if (latestData) {
				updateNode(latestData).catch((error) => {
					console.error("Failed to update node:", error);
					toast.error("Failed to update node");
				});
				latestNodeDataRef.current.delete(nodeId);
			}
			pendingUpdatesRef.current.delete(nodeId);
		}, DEBOUNCE_DELAY);

		pendingUpdatesRef.current.set(nodeId, timeout);
	}, []);

	// Save current state to history (max 10 levels)
	const saveHistory = useCallback(() => {
		if (isUndoingRef.current) return;

		setHistory((prev) => {
			const newPast = [
				...prev.past,
				{
					nodes: JSON.parse(JSON.stringify(nodes)),
					edges: JSON.parse(JSON.stringify(edges)),
				},
			];
			// Keep only last 10 states
			if (newPast.length > 10) {
				newPast.shift();
			}
			return {
				past: newPast,
				future: [], // Clear future when new action is performed
			};
		});
	}, [nodes, edges]);

	// Undo function
	const undo = useCallback(() => {
		if (history.past.length === 0) {
			toast.error("Nothing to undo");
			return;
		}

		flushPendingUpdates();
		isUndoingRef.current = true;

		setHistory((prev) => {
			const newPast = [...prev.past];
			const previousState = newPast.pop();

			if (!previousState) return prev;

			const currentState = {
				nodes: JSON.parse(JSON.stringify(nodes)),
				edges: JSON.parse(JSON.stringify(edges)),
			};

			return {
				past: newPast,
				future: [currentState, ...prev.future],
			};
		});

		const previousState = history.past[history.past.length - 1];
		if (previousState) {
			setNodes(JSON.parse(JSON.stringify(previousState.nodes)));
			setEdges(JSON.parse(JSON.stringify(previousState.edges)));
			toast.success("Undo");
		}

		setTimeout(() => {
			isUndoingRef.current = false;
		}, 100);
	}, [history, nodes, edges, setNodes, setEdges, flushPendingUpdates]);

	// Redo function
	const redo = useCallback(() => {
		if (history.future.length === 0) {
			toast.error("Nothing to redo");
			return;
		}

		flushPendingUpdates();
		isUndoingRef.current = true;

		setHistory((prev) => {
			const newFuture = [...prev.future];
			const nextState = newFuture.shift();

			if (!nextState) return prev;

			const currentState = {
				nodes: JSON.parse(JSON.stringify(nodes)),
				edges: JSON.parse(JSON.stringify(edges)),
			};

			return {
				past: [...prev.past, currentState],
				future: newFuture,
			};
		});

		const nextState = history.future[0];
		if (nextState) {
			setNodes(JSON.parse(JSON.stringify(nextState.nodes)));
			setEdges(JSON.parse(JSON.stringify(nextState.edges)));
			toast.success("Redo");
		}

		setTimeout(() => {
			isUndoingRef.current = false;
		}, 100);
	}, [history, nodes, edges, setNodes, setEdges, flushPendingUpdates]);

	// Keyboard shortcuts for undo/redo and delete
	useEffect(() => {
		const handleKeyDown = (e: KeyboardEvent) => {
			const isMac = navigator.platform.toUpperCase().indexOf("MAC") >= 0;
			const modifier = isMac ? e.metaKey : e.ctrlKey;

			// Undo/Redo shortcuts
			if (modifier && e.key === "z" && !e.shiftKey) {
				e.preventDefault();
				undo();
			} else if (modifier && (e.key === "y" || (e.key === "z" && e.shiftKey))) {
				e.preventDefault();
				redo();
			}
			// Delete/Backspace for selected nodes and edges
			else if (e.key === "Delete" || e.key === "Backspace") {
				// Don't delete if user is typing in an input field
				const target = e.target as HTMLElement;
				if (target.tagName === "INPUT" || target.tagName === "TEXTAREA") {
					return;
				}

				e.preventDefault();

				const selectedNodes = nodes.filter((n) => n.selected);
				const selectedEdges = edges.filter((e) => e.selected);

				if (selectedNodes.length === 0 && selectedEdges.length === 0) {
					return;
				}

				// Save history before deletion
				saveHistory();

				// Delete selected nodes
				if (selectedNodes.length > 0) {
					selectedNodes.forEach((node) => {
						deleteNode(node.id).catch((error) => {
							console.error("Failed to delete node:", error);
							toast.error(`Failed to delete node from backend`);
						});
					});
					setNodes(nodes.filter((n) => !n.selected));
					toast.success(
						`Deleted ${selectedNodes.length} node${selectedNodes.length > 1 ? "s" : ""}`,
					);
				}

				// Delete selected edges
				if (selectedEdges.length > 0) {
					selectedEdges.forEach((edge) => {
						if (edge.id.startsWith("reactflow")) {
							console.debug("Skipping deletion of temporary edge:", edge.id);
							return;
						}
						deleteNodeLink(edge.id).catch((error) => {
							console.error("Failed to delete edge:", error);
							// toast.error(`Failed to delete connection from backend`);
						});
					});
					setEdges(edges.filter((e) => !e.selected));
					toast.success(
						`Deleted ${selectedEdges.length} connection${selectedEdges.length > 1 ? "s" : ""}`,
					);
				}
			}
		};

		window.addEventListener("keydown", handleKeyDown);
		return () => window.removeEventListener("keydown", handleKeyDown);
	}, [undo, redo, nodes, edges, setNodes, setEdges, saveHistory]);

	// Cleanup: flush pending updates on unmount
	useEffect(() => {
		return () => {
			flushPendingUpdates();
		};
	}, [flushPendingUpdates]);

	const onConnect: OnConnect = useCallback(
		async (params) => {
			// Save history before making changes
			saveHistory();

			// Add edge locally
			setEdges((eds) => addEdge(params, eds));

			// Validate source and target
			if (!params.source || !params.target) {
				console.error("Invalid connection: missing source or target");
				return;
			}

			// Save to backend
			const projectId = localStorage.getItem(PROJECT_ID_KEY);
			if (!projectId || projectId === "undefined" || projectId.trim() === "") {
				console.error("No valid project ID found when creating node link");
				toast.error("Cannot create link: no project selected");
				return;
			}

			try {
				const nodeLink = {
					id: uuidv4(),
					left: params.source,
					right: params.target,
					project_id: projectId,
					linktype: "Directional" as const,
				};

				await createNodeLink(nodeLink);
				toast.success("Connection saved");
			} catch (error) {
				console.error("Failed to save connection:", error);
				toast.error("Failed to save connection to backend");
			}
		},
		[setEdges, saveHistory],
	);

	const getNodeColor = useCallback((nodeType: string): string => {
		const colors: Record<string, string> = {
			person: "#3b82f6",
			domain: "#f59e0b",
			ip: "#ef4444",
			phone: "#8b5cf6",
			email: "#ec4899",
			url: "#06b6d4",
			image: "#10b981",
			location: "#84cc16",
			organization: "#f97316",
			document: "#6b7280",
		};
		return colors[nodeType] ?? "#6b7280";
	}, []);

	/** Helper function to load data for a project */
	const loadProjectData = useCallback(
		async (projectId: string) => {
			try {
				// Load project data using export endpoint (single request)
				const exportData = await exportProject(projectId, false);

				// Store all attachments for later use
				setAllAttachments(exportData.attachments || []);

				// Convert nodes to ReactFlow format
				const reactFlowNodes: Node[] = exportData.nodes.map((osintNode) => ({
					id: osintNode.id,
					type: "default",
					position: { x: osintNode.pos_x || 100, y: osintNode.pos_y || 100 },
					data: {
						label: osintNode.display,
						nodeType: osintNode.node_type,
						osintNode: osintNode,
					},
					style: {
						background: getNodeColor(osintNode.node_type),
						color: "white",
						border: "1px solid #222",
						width: 180,
						cursor: "pointer",
					},
				}));
				setNodes(reactFlowNodes);

				// Convert links to ReactFlow edges
				const reactFlowEdges: Edge[] = exportData.nodelinks.map((nodeLink) => ({
					id: nodeLink.id,
					source: nodeLink.left,
					target: nodeLink.right,
					type: nodeLink.linktype === "Directional" ? "default" : "straight",
				}));
				setEdges(reactFlowEdges);

				return exportData.project;
			} catch (error) {
				console.error("Failed to load project data:", error);
				throw error;
			}
		},
		[setNodes, setEdges, getNodeColor],
	);

	// Initialize project on component mount or when currentProject is cleared
	useEffect(() => {
		// Don't run if we already have a project
		if (currentProject !== null) return;

		// Don't run if mismatch dialog is showing (user is selecting a project)
		if (showMismatchDialog) return;

		const initializeProject = async () => {
			try {
				setIsLoading(true);
				const projectId = localStorage.getItem(PROJECT_ID_KEY);

				// Check for valid project ID (not null, not "undefined", not empty)
				if (projectId && projectId !== "undefined" && projectId.trim() !== "") {
					// Load project and data in single request
					try {
						const project = await loadProjectData(projectId);
						if (project) {
							setCurrentProject(project);
							setIsLoading(false);
						} else {
							// Project doesn't exist in backend, show mismatch dialog
							const projects = await fetchProjects();
							setAvailableProjects(projects);
							setShowMismatchDialog(true);
							setIsLoading(false);
						}
					} catch (error) {
						// If export fails (404), project doesn't exist
						console.error("Failed to load project data:", error);
						const projects = await fetchProjects();
						setAvailableProjects(projects);
						setShowMismatchDialog(true);
						setIsLoading(false);
					}
				} else {
					// No project ID in localStorage, create new project
					const project = await createProject();
					localStorage.setItem(PROJECT_ID_KEY, project.id);
					setCurrentProject(project);
					setIsLoading(false);
				}
			} catch (error) {
				console.error("Failed to initialize project:", error);
				toast.error("Failed to initialize project");
				setIsLoading(false);
			}
		};

		initializeProject();
	}, [currentProject, showMismatchDialog, loadProjectData]);

	const nodeTypes = Object.keys(NodeTypeInfo);

	const handleCreateNewProject = useCallback(async () => {
		try {
			const project = await createProject();
			localStorage.setItem(PROJECT_ID_KEY, project.id);
			setCurrentProject(project);
			setShowMismatchDialog(false);
			setNodes([]);
			setEdges([]);
			toast.success(`Created new project: ${project.name}`);
		} catch (error) {
			console.error("Failed to create project:", error);
			toast.error("Failed to create new project");
		}
	}, [setNodes, setEdges]);

	const handleProjectUpdate = useCallback((updatedProject: Project) => {
		setCurrentProject(updatedProject);
	}, []);

	const handleProjectDelete = useCallback(async () => {
		// Clear everything
		localStorage.removeItem(PROJECT_ID_KEY);
		setNodes([]);
		setEdges([]);
		setShowProjectManagement(false);

		// Fetch projects AFTER deletion is confirmed complete
		try {
			const projects = await fetchProjects();

			if (projects.length > 0) {
				// Show project selector if other projects exist
				setAvailableProjects(projects);
				setShowMismatchDialog(true);
				setCurrentProject(null);
				toast.success("Project deleted - please select a project");
			} else {
				// No other projects, create a new one
				setTimeout(() => {
					setCurrentProject(null);
					toast.success("Project deleted - creating new project");
				}, 100);
			}
		} catch (error) {
			console.error("Failed to fetch projects:", error);
			// Fallback to creating new project
			setTimeout(() => {
				setCurrentProject(null);
			}, 100);
		}
	}, [setNodes, setEdges]);

	const handleProjectChange = useCallback(
		async (projectId: string) => {
			try {
				// Load project and data in single request
				const project = await loadProjectData(projectId);
				if (project) {
					localStorage.setItem(PROJECT_ID_KEY, projectId);
					setCurrentProject(project);
					toast.success(`Switched to project: ${project.name}`);
				}
			} catch (error) {
				console.error("Failed to switch project:", error);
				toast.error("Failed to switch project");
			}
		},
		[loadProjectData],
	);

	const handleProjectSelect = useCallback(
		async (projectId: string) => {
			// Load project first, then close dialog to prevent race condition
			await handleProjectChange(projectId);
			setShowMismatchDialog(false);
		},
		[handleProjectChange],
	);

	// Get viewport center position for new nodes
	const getViewportCenterPosition = useCallback((): {
		x: number;
		y: number;
	} => {
		const reactFlowBounds = document
			.querySelector(".react-flow")
			?.getBoundingClientRect();
		if (!reactFlowBounds) {
			return { x: 400, y: 300 }; // Fallback position
		}

		// Get the center of the visible viewport
		const centerX = reactFlowBounds.width / 2;
		const centerY = reactFlowBounds.height / 2;

		// Convert screen coordinates to flow coordinates
		const position = project({ x: centerX, y: centerY });

		// Add small random offset to prevent exact stacking
		const offsetX = (Math.random() - 0.5) * 40; // ±20px
		const offsetY = (Math.random() - 0.5) * 40; // ±20px

		return {
			x: Math.round(position.x + offsetX),
			y: Math.round(position.y + offsetY),
		};
	}, [project]);

	const createOSINTNode = useCallback(
		async (nodeType: string) => {
			let projectId = localStorage.getItem(PROJECT_ID_KEY);
			if (!projectId || projectId === "undefined" || projectId.trim() === "") {
				console.log("No valid project ID found, creating new project...");
				try {
					const project = await createProject();
					projectId = project.id;
					localStorage.setItem(PROJECT_ID_KEY, projectId);
					setCurrentProject(project);
				} catch (error) {
					console.error("Failed to create new project:", error);
					return;
				}
			}

			// Place new node at viewport center with small random offset
			const position = getViewportCenterPosition();
			const x = position.x;
			const y = position.y;

			const nodeId = uuidv4();

			const osintNode: OSINTNode = {
				id: nodeId,
				project_id: projectId,
				node_type: nodeType,
				display: `New ${NodeTypeInfo[nodeType]?.label ?? nodeType}`,
				value: "",
				updated: new Date().toISOString(),
				pos_x: Math.round(x),
				pos_y: Math.round(y),
				attachments: [],
			};

			const newReactFlowNode: Node = {
				id: nodeId,
				type: "default",
				position: { x, y },
				data: {
					label: osintNode.display,
					nodeType: nodeType,
					osintNode: osintNode,
				},
				style: {
					background: getNodeColor(nodeType),
					color: "white",
					border: "1px solid #222",
					width: 180,
					cursor: "pointer",
				},
			};

			// Save history before making changes
			saveHistory();

			// Update local state immediately
			setNodes((nds) => nds.concat(newReactFlowNode));

			// Mark node as pending (not yet saved to backend)
			setPendingNodes((prev) => new Set(prev).add(nodeId));

			// Automatically open edit UI for the new node
			setEditingNode(nodeId);
			setEditDisplay(osintNode.display);
			setEditValue("");

			// Don't save to backend yet - wait for user to click Save
		},
		[getViewportCenterPosition, setNodes, getNodeColor, saveHistory],
	);

	const handleNodeDoubleClick = useCallback(
		(event: React.MouseEvent, node: Node) => {
			event.stopPropagation();

			// If we're in move mode, complete the move
			if (movingAttachment) {
				const targetNodeId = node.id;
				const { attachmentId, filename, sourceNodeId } = movingAttachment;

				// Don't move to the same node
				if (targetNodeId === sourceNodeId) {
					toast.error("Cannot move attachment to the same node");
					return;
				}

				// Perform the move
				updateAttachment(attachmentId, targetNodeId)
					.then((updatedAttachment) => {
						// Update the cache
						setAllAttachments((prev) =>
							prev.map((a) => (a.id === attachmentId ? updatedAttachment : a)),
						);

						// If we're still viewing the source node, remove from its list
						if (editingNode === sourceNodeId) {
							setNodeAttachments((prev) =>
								prev.filter((a) => a.id !== attachmentId),
							);
						}

						toast.success(
							`Moved "${filename}" to ${node.data.label as string}`,
						);
						setMovingAttachment(null);
					})
					.catch((error) => {
						console.error("Failed to move attachment:", error);
						toast.error("Failed to move attachment");
					});

				return;
			}

			// Normal edit mode
			setEditingNode(node.id);
			// eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
			const nodeType = node.data.nodeType as string;
			setEditingNodeType(nodeType);
			// eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
			setEditDisplay(node.data.osintNode?.display ?? node.data.label ?? "");
			// eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
			setEditValue(node.data.osintNode?.value ?? "");
			// eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
			setEditNotes(node.data.osintNode?.notes ?? "");
		},
		[movingAttachment, editingNode],
	);

	const handleNodeContextMenu = useCallback(
		(event: React.MouseEvent, node: Node) => {
			event.preventDefault();
			// eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
			const nodeType = node.data.nodeType as string;

			// Only show context menu for URL nodes
			if (nodeType === "url") {
				setContextMenu({
					x: event.clientX,
					y: event.clientY,
					node,
				});
			}
		},
		[],
	);

	const handleOpenUrl = useCallback(() => {
		if (!contextMenu) return;

		// eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
		const url = contextMenu.node.data.osintNode?.value as string;
		if (url) {
			// Trim and clean the URL to remove any invisible characters
			const cleanUrl = url.trim().replace(/[\u200B-\u200D\uFEFF\u2069]/g, "");
			// Add https:// if no protocol specified
			const fullUrl = cleanUrl.match(/^https?:\/\//)
				? cleanUrl
				: `https://${cleanUrl}`;
			window.open(fullUrl, "_blank", "noopener,noreferrer");
		}

		setContextMenu(null);
	}, [contextMenu]);

	// Close context menu when clicking elsewhere
	useEffect(() => {
		const handleClick = () => setContextMenu(null);
		if (contextMenu) {
			document.addEventListener("click", handleClick);
			return () => document.removeEventListener("click", handleClick);
		}
		return undefined;
	}, [contextMenu]);

	const cancelNodeEdit = useCallback(() => {
		if (!editingNode) return;

		const isPending = pendingNodes.has(editingNode);

		if (isPending) {
			// Remove the pending node from the UI since it was never saved
			setNodes((nds) => nds.filter((n) => n.id !== editingNode));
			setPendingNodes((prev) => {
				const newSet = new Set(prev);
				newSet.delete(editingNode);
				return newSet;
			});
		}

		setEditingNode(null);
		setEditingNodeType(null);
		setEditDisplay("");
		setEditValue("");
		setEditNotes("");
	}, [editingNode, pendingNodes, setNodes]);

	const handleDeleteNodeFromDialog = useCallback(() => {
		if (!editingNode) return;

		const node = nodes.find((n) => n.id === editingNode);
		// eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
		const nodeName = node?.data?.osintNode?.display ?? "this node";

		if (!window.confirm(`Delete "${nodeName}"?`)) return;

		const isPending = pendingNodes.has(editingNode);

		// Save history before deletion
		saveHistory();

		// Remove from UI
		setNodes((nds) => nds.filter((n) => n.id !== editingNode));

		// If it's not a pending node, delete from backend
		if (!isPending) {
			deleteNode(editingNode)
				.then(() => {
					toast.success("Node deleted");
				})
				.catch((error) => {
					console.error("Failed to delete node:", error);
					toast.error("Failed to delete node from backend");
				});
		} else {
			// Just remove from pending set for unsaved nodes
			setPendingNodes((prev) => {
				const newSet = new Set(prev);
				newSet.delete(editingNode);
				return newSet;
			});
		}

		// Close the edit dialog
		setEditingNode(null);
		setEditingNodeType(null);
		setEditDisplay("");
		setEditValue("");
		setEditNotes("");
		setNodeAttachments([]);
	}, [editingNode, nodes, pendingNodes, setNodes, saveHistory]);

	const saveNodeEdit = useCallback(async () => {
		if (!editingNode) return;

		const isPending = pendingNodes.has(editingNode);

		// Save history before making changes
		saveHistory();

		// For person, organization, image, and document nodes, sync value with display
		const nodeTypesWithSyncedValue = [
			"person",
			"organization",
			"image",
			"document",
		];
		const finalValue = nodeTypesWithSyncedValue.includes(editingNodeType ?? "")
			? editDisplay
			: editValue;

		setNodes((nds) =>
			nds.map((node) => {
				if (node.id === editingNode) {
					const updatedOSINTNode: OSINTNode = {
						// eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
						...node.data.osintNode,
						display: editDisplay,
						value: finalValue,
						notes: editNotes || undefined,
						updated: new Date().toISOString(),
					};

					if (isPending) {
						// This is a new node - save to backend immediately
						createNode(updatedOSINTNode)
							.then(() => {
								toast.success("Node created successfully");
								// Remove from pending set
								setPendingNodes((prev) => {
									const newSet = new Set(prev);
									newSet.delete(editingNode);
									return newSet;
								});
							})
							.catch((error) => {
								console.error("Failed to create node:", error);
								toast.error("Failed to create node");
							});
					} else {
						// Existing node - use debounced update
						debouncedUpdateNode(updatedOSINTNode);
					}

					return {
						...node,
						data: {
							...node.data,
							label: editDisplay,
							osintNode: updatedOSINTNode,
						},
					};
				}
				return node;
			}),
		);

		setEditingNode(null);
		setEditingNodeType(null);
		setEditDisplay("");
		setEditValue("");
		setEditNotes("");
		setNodeAttachments([]);
	}, [
		editingNode,
		editingNodeType,
		editDisplay,
		editValue,
		editNotes,
		setNodes,
		debouncedUpdateNode,
		saveHistory,
		pendingNodes,
	]);

	// Load attachments when editing a node
	useEffect(() => {
		if (editingNode && !pendingNodes.has(editingNode)) {
			// First, try to use cached attachments from export data
			const cachedAttachments = allAttachments.filter(
				(att) => att.node_id === editingNode,
			);

			if (cachedAttachments.length > 0) {
				// Use cached attachments
				setNodeAttachments(cachedAttachments);
			} else {
				// Fall back to API call if no cached attachments
				listAttachments(editingNode)
					.then(setNodeAttachments)
					.catch((error) => {
						console.error("Failed to load attachments:", error);
						toast.error("Failed to load attachments");
					});
			}
		} else {
			setNodeAttachments([]);
		}
	}, [editingNode, pendingNodes, allAttachments]);

	const handleFileUpload = useCallback(
		async (event: React.ChangeEvent<HTMLInputElement>) => {
			if (!editingNode || !event.target.files?.length) return;

			const file = event.target.files[0];
			if (!file) {
				toast.error("No file selected for upload");
				return;
			}
			setUploadingAttachment(true);

			try {
				const attachment = await uploadAttachment(editingNode, file);
				setNodeAttachments((prev) => [...prev, attachment]);
				// Also update the global attachments cache
				setAllAttachments((prev) => [...prev, attachment]);
				toast.success(`Uploaded ${file.name}`);
			} catch (error) {
				console.error("Failed to upload attachment:", error);
				toast.error("Failed to upload file");
			} finally {
				setUploadingAttachment(false);
				// Reset input so the same file can be uploaded again
				event.target.value = "";
			}
		},
		[editingNode],
	);

	const handleDownloadAttachment = useCallback(
		async (attachment: Attachment) => {
			if (!editingNode) return;

			try {
				const blob = await downloadAttachment(attachment.id);
				const url = window.URL.createObjectURL(blob);
				const a = document.createElement("a");
				a.href = url;
				a.download = attachment.filename;
				document.body.appendChild(a);
				a.click();
				window.URL.revokeObjectURL(url);
				document.body.removeChild(a);
				toast.success(`Downloaded ${attachment.filename}`);
			} catch (error) {
				console.error("Failed to download attachment:", error);
				toast.error("Failed to download file");
			}
		},
		[editingNode],
	);

	const handleDeleteAttachment = useCallback(
		async (attachmentId: string, filename: string) => {
			if (!editingNode) return;

			if (!window.confirm(`Delete ${filename}?`)) return;

			try {
				await deleteAttachment(attachmentId);
				setNodeAttachments((prev) => prev.filter((a) => a.id !== attachmentId));
				// Also update the global attachments cache
				setAllAttachments((prev) => prev.filter((a) => a.id !== attachmentId));
				toast.success(`Deleted ${filename}`);
			} catch (error) {
				console.error("Failed to delete attachment:", error);
				toast.error("Failed to delete file");
			}
		},
		[editingNode],
	);

	const handleMoveAttachment = useCallback(
		(attachmentId: string, filename: string) => {
			if (!editingNode) return;

			setMovingAttachment({
				attachmentId,
				filename,
				sourceNodeId: editingNode,
			});
			toast.success(`Click on a node to move "${filename}" to it`, {
				duration: 5000,
			});
		},
		[editingNode],
	);

	const handleCancelMoveAttachment = useCallback(() => {
		setMovingAttachment(null);
		toast.success("Cancelled moving attachment");
	}, []);

	const formatFileSize = (bytes: number): string => {
		if (bytes < 1024) return `${bytes} B`;
		if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
		return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
	};

	const isViewableFile = (contentType: string, filename: string): boolean => {
		// Images
		if (contentType.startsWith("image/")) return true;
		// PDFs
		if (contentType === "application/pdf") return true;
		// Text files
		if (contentType.startsWith("text/")) return true;
		// JSON, XML
		if (contentType === "application/json" || contentType === "application/xml")
			return true;

		// Check by extension if content type is generic
		const ext = filename.toLowerCase().split(".").pop();
		const viewableExts = [
			"jpg",
			"jpeg",
			"png",
			"gif",
			"bmp",
			"webp",
			"svg",
			"pdf",
			"txt",
			"md",
			"json",
			"xml",
			"html",
			"css",
			"js",
			"ts",
			"tsx",
			"jsx",
		];
		return ext ? viewableExts.includes(ext) : false;
	};

	const handleViewAttachment = useCallback(
		(attachment: Attachment) => {
			if (!editingNode) return;

			const url = `/api/v1/attachment/${attachment.id}/view`;
			window.open(url, "_blank");
		},
		[editingNode],
	);

	const handleNodesChange: OnNodesChange = useCallback(
		(changes) => {
			// Check if any changes are removals, save history before applying
			const hasRemove = changes.some((change) => change.type === "remove");

			if (hasRemove) {
				saveHistory();
			}

			onNodesChange(changes);

			// Update position changes in backend
			changes.forEach((change) => {
				if (change.type === "position" && change.position) {
					const node = nodes.find((n) => n.id === change.id);
					// eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
					if (node?.data.osintNode) {
						const projectId = localStorage.getItem(PROJECT_ID_KEY);
						if (
							!projectId ||
							projectId === "undefined" ||
							projectId.trim() === ""
						) {
							console.error(
								"No valid project ID found when updating node position",
							);
							return;
						}

						const updatedNode: OSINTNode = {
							// eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
							...node.data.osintNode,
							project_id: projectId,
							pos_x: Math.round(change.position.x),
							pos_y: Math.round(change.position.y),
							updated: new Date().toISOString(),
						};
						// Use debounced update for position changes (happens frequently during drag)
						debouncedUpdateNode(updatedNode);
					}
				}
			});
		},
		[onNodesChange, nodes, debouncedUpdateNode, saveHistory],
	);

	// Handle edge changes (including deletions)
	const handleEdgesChange: OnEdgesChange = useCallback(
		(changes) => {
			const hasRemove = changes.some((change) => change.type === "remove");

			if (hasRemove) {
				saveHistory();

				// Delete nodelinks from backend
				changes.forEach((change) => {
					if (change.type === "remove") {
						console.debug("Deleting nodelink:", change);
						if (!change.id.startsWith("reactflow")) {
							deleteNodeLink(change.id).catch((error) => {
								console.error("Failed to delete nodelink:", error);
								// toast.error('Failed to delete connection from backend');
							});
						}
					}
				});
			}

			onEdgesChange(changes);
		},
		[onEdgesChange, saveHistory],
	);

	// Save history when node drag starts (for position undo)
	const onNodeDragStart = useCallback(() => {
		saveHistory();
	}, [saveHistory]);

	if (isLoading) {
		return <div className="loadingScreen">Initializing OSINT Graph...</div>;
	}

	return (
		<div className="app-container">
			<Toaster position="top-right" />

			<ProjectSelector
				currentProject={currentProject}
				onProjectChange={handleProjectChange}
				onCreateNew={handleCreateNewProject}
				setShowProjectManagement={setShowProjectManagement}
			/>

			{/* <button
        onClick={() => setShowProjectManagement(true)}
        className="btn btn-primary project-settings-button"
        title="Project Settings"
      >
        ⚙️ Settings
      </button> */}

			{showMismatchDialog && (
				<ProjectMismatchDialog
					onCreateNew={handleCreateNewProject}
					onProjectSelect={handleProjectSelect}
					projects={availableProjects}
				/>
			)}

			{showProjectManagement && (
				<ProjectManagementDialog
					isOpen={showProjectManagement}
					onClose={() => setShowProjectManagement(false)}
					currentProject={currentProject}
					onProjectUpdate={handleProjectUpdate}
					onProjectDelete={handleProjectDelete}
				/>
			)}

			<ReactFlow
				nodes={nodes}
				edges={edges}
				onNodesChange={handleNodesChange}
				onEdgesChange={handleEdgesChange}
				onConnect={onConnect}
				onNodeDragStart={onNodeDragStart}
				onNodeDoubleClick={handleNodeDoubleClick}
				onNodeContextMenu={handleNodeContextMenu}
				fitView
				style={{
					cursor: movingAttachment ? "crosshair" : "default",
				}}
			>
				<Controls />
				<MiniMap />
				<Background />
			</ReactFlow>

			{/* Right-side collapsible panel for adding nodes */}
			<div
				className={`node-panel ${isPanelCollapsed ? "node-panel-collapsed" : "node-panel-expanded"}`}
			>
				{/* Collapse/Expand button */}
				<button
					type="button"
					onClick={() => setIsPanelCollapsed(!isPanelCollapsed)}
					className="node-panel-collapse-button"
					title={isPanelCollapsed ? "Expand panel" : "Collapse panel"}
				>
					{isPanelCollapsed ? "◀" : "▶"}
				</button>

				{/* Panel content */}
				<div style={{ padding: "16px", overflowY: "auto", flex: 1 }}>
					<div
						style={{
							marginBottom: "12px",
							fontWeight: "bold",
							fontSize: "16px",
						}}
					>
						Add Node
					</div>
					{nodeTypes.map((type) => (
						<button
							type="button"
							key={type}
							onClick={() => createOSINTNode(type)}
							style={{
								display: "block",
								width: "100%",
								padding: "10px 12px",
								margin: "4px 0",
								border: "none",
								borderRadius: "6px",
								background: getNodeColor(type),
								color: "white",
								cursor: "pointer",
								textAlign: "left",
								fontSize: "14px",
								fontWeight: "500",
								transition: "transform 0.1s ease, box-shadow 0.1s ease",
							}}
							onMouseEnter={(e) => {
								e.currentTarget.style.transform = "translateY(-2px)";
								e.currentTarget.style.boxShadow =
									"0 4px 8px rgba(0, 0, 0, 0.2)";
							}}
							onMouseLeave={(e) => {
								e.currentTarget.style.transform = "translateY(0)";
								e.currentTarget.style.boxShadow = "none";
							}}
						>
							{NodeTypeInfo[type]?.label ?? type}
						</button>
					))}
				</div>
			</div>

			{editingNode && (
				<div
					style={{
						position: "fixed",
						top: "50%",
						left: "50%",
						transform: "translate(-50%, -50%)",
						background: "white",
						border: "1px solid #ccc",
						borderRadius: "8px",
						padding: "20px",
						boxShadow: "0 4px 12px rgba(0, 0, 0, 0.15)",
						zIndex: 1001,
						minWidth: "300px",
					}}
				>
					<h3>Edit Node</h3>
					<div className="modal-field">
						<label
							htmlFor={idDisplay}
							style={{
								display: "block",
								marginBottom: "4px",
								fontWeight: "500",
								fontSize: "14px",
							}}
						>
							Display Name
						</label>
						<input
							type="text"
							id={idDisplay}
							value={editDisplay}
							onChange={(e) => setEditDisplay(e.target.value)}
							style={{
								width: "100%",
								padding: "8px",
								border: "1px solid #ccc",
								borderRadius: "4px",
								boxSizing: "border-box",
							}}
							placeholder="Name shown on graph"
							// biome-ignore lint/a11y/noAutofocus: "it's a dialogue for input"
							autoFocus
							onKeyDown={(e) => {
								if (e.key === "Escape") {
									cancelNodeEdit();
								}
							}}
						/>
					</div>

					{/* Only show value field for nodes that need it (not person, organization, image, document) */}
					{editingNodeType !== "person" &&
						editingNodeType !== "organization" &&
						editingNodeType !== "image" &&
						editingNodeType !== "document" && (
							<div className="modal-field">
								<label
									htmlFor={idValue}
									style={{
										display: "block",
										marginBottom: "4px",
										fontWeight: "500",
										fontSize: "14px",
									}}
								>
									Value
								</label>
								<input
									type="text"
									value={editValue}
									id={idValue}
									onChange={(e) => setEditValue(e.target.value)}
									style={{
										width: "100%",
										padding: "8px",
										border: "1px solid #ccc",
										borderRadius: "4px",
										boxSizing: "border-box",
									}}
									placeholder="Actual value (e.g., email, phone number)"
								/>
							</div>
						)}

					<div className="modal-field">
						<label
							htmlFor={idMoreInfo}
							style={{
								display: "block",
								marginBottom: "4px",
								fontWeight: "500",
								fontSize: "14px",
							}}
						>
							Notes
						</label>
						<textarea
							value={editNotes}
							onChange={(e) => setEditNotes(e.target.value)}
							id={idMoreInfo}
							style={{
								width: "100%",
								padding: "8px",
								border: "1px solid #ccc",
								borderRadius: "4px",
								boxSizing: "border-box",
								minHeight: "80px",
								resize: "vertical",
								fontFamily: "inherit",
							}}
							placeholder="Additional information..."
						/>
					</div>

					{/* Attachments section - only show for saved nodes */}
					{!pendingNodes.has(editingNode) && (
						<div className="modal-field">
							<div
								style={{
									display: "block",
									marginBottom: "8px",
									fontWeight: "500",
									fontSize: "14px",
								}}
							>
								Attachments
							</div>

							{/* List of existing attachments */}
							{nodeAttachments.length > 0 && (
								<div
									style={{
										marginBottom: "12px",
										border: "1px solid #e5e7eb",
										borderRadius: "4px",
										overflow: "hidden",
									}}
								>
									{nodeAttachments.map((attachment) => (
										<div key={attachment.id} className="attachment-item">
											<div style={{ flex: 1, minWidth: 0 }}>
												<div className="attachment-filename">
													{attachment.filename}
												</div>
												<div
													style={{
														fontSize: "12px",
														color: "#6b7280",
													}}
												>
													{formatFileSize(attachment.size)}
												</div>
											</div>
											<div
												style={{
													display: "flex",
													gap: "8px",
													marginLeft: "12px",
												}}
											>
												{isViewableFile(
													attachment.content_type,
													attachment.filename,
												) && (
													<button
														type="button"
														onClick={() => handleViewAttachment(attachment)}
														className="btn btn-success"
														title="View in new tab"
													>
														👁
													</button>
												)}
												<button
													type="button"
													onClick={() => handleDownloadAttachment(attachment)}
													className="btn btn-primary"
													title="Download"
												>
													↓
												</button>
												<button
													type="button"
													onClick={() =>
														handleMoveAttachment(
															attachment.id,
															attachment.filename,
														)
													}
													className="btn btn-warning"
													title="Move to another node"
												>
													📦
												</button>
												<button
													type="button"
													onClick={() =>
														handleDeleteAttachment(
															attachment.id,
															attachment.filename,
														)
													}
													className="btn btn-danger"
													title="Delete"
												>
													×
												</button>
											</div>
										</div>
									))}
								</div>
							)}

							{/* Upload button */}
							<label
								style={{
									display: "inline-block",
									padding: "8px 12px",
									background: "#10b981",
									color: "white",
									border: "none",
									borderRadius: "4px",
									cursor: uploadingAttachment ? "wait" : "pointer",
									fontSize: "13px",
									opacity: uploadingAttachment ? 0.6 : 1,
								}}
							>
								{uploadingAttachment ? "Uploading..." : "Upload File"}
								<input
									type="file"
									onChange={handleFileUpload}
									disabled={uploadingAttachment}
									style={{ display: "none" }}
								/>
							</label>
						</div>
					)}

					<div
						style={{
							display: "flex",
							gap: "10px",
							justifyContent: "space-between",
						}}
					>
						<div style={{ display: "flex", gap: "10px" }}>
							<button
								type="button"
								onClick={saveNodeEdit}
								className="btn btn-primary"
							>
								Save
							</button>
							<button
								type="button"
								onClick={cancelNodeEdit}
								className="btn btn-secondary"
							>
								Cancel
							</button>
						</div>
						<button
							type="button"
							onClick={handleDeleteNodeFromDialog}
							className="btn btn-danger"
							title="Delete this node"
						>
							Delete
						</button>
					</div>
				</div>
			)}

			{/* Context menu for URL nodes */}
			{contextMenu && (
				<div
					role="menuitem"
					className="context-menu"
					style={{ top: contextMenu.y, left: contextMenu.x }}
					onClick={(e) => e.stopPropagation()}
					onKeyDown={() => {}}
					tabIndex={0}
				>
					<button
						type="button"
						onClick={handleOpenUrl}
						className="context-menu-item"
					>
						<span>🔗</span>
						<span>Open in new tab</span>
					</button>
				</div>
			)}

			{/* Move attachment mode indicator */}
			{movingAttachment && (
				<div
					style={{
						position: "fixed",
						top: "20px",
						left: "50%",
						transform: "translateX(-50%)",
						background: "#3b82f6",
						color: "white",
						padding: "12px 20px",
						borderRadius: "8px",
						boxShadow: "0 4px 12px rgba(0, 0, 0, 0.15)",
						zIndex: 1002,
						display: "flex",
						alignItems: "center",
						gap: "12px",
						fontSize: "14px",
						fontWeight: "500",
					}}
				>
					<span>
						📦 Moving "{movingAttachment.filename}" - Double-click a node to
						move it there
					</span>
					<button
						type="button"
						onClick={handleCancelMoveAttachment}
						style={{
							background: "white",
							color: "#3b82f6",
							border: "none",
							borderRadius: "4px",
							padding: "6px 12px",
							cursor: "pointer",
							fontSize: "13px",
							fontWeight: "600",
						}}
					>
						Cancel
					</button>
				</div>
			)}
		</div>
	);
}

export default function App() {
	return (
		<ReactFlowProvider>
			<AppContent />
		</ReactFlowProvider>
	);
}
