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
	setAuthFailureCallback,
	updateAttachment,
	updateNode,
	uploadAttachment,
} from "./api";
import { ContextMenuItem } from "./components/ContextMenuItem";
import { LoginDialog } from "./components/LoginDialog";
import { NodeSearch } from "./components/NodeSearch";
import { ProjectManagementDialog } from "./components/ProjectManagementDialog";
import { ProjectMismatchDialog } from "./components/ProjectMismatchDialog";
import { ProjectSelector } from "./components/ProjectSelector";
import { AuthProvider, useAuth } from "./contexts/AuthContext";
import type { Attachment, OSINTNode, Project } from "./types";
import { getNodeColor, hasSyncedValue, NodeTypeInfo } from "./types";
import "./osint-graph.css";

const initialNodes: Node[] = [];
const initialEdges: Edge[] = [];

const PROJECT_ID_KEY = "osint-graph-project-id";
const DEBOUNCE_DELAY = 100; // ms

function AppContent() {
	const [nodes, setNodes, onNodesChange] = useNodesState(initialNodes);
	const [edges, setEdges, onEdgesChange] = useEdgesState(initialEdges);
	const { screenToFlowPosition, setCenter, getZoom } = useReactFlow();
	const { requireLogin } = useAuth();
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

	// Set up authentication failure callback
	useEffect(() => {
		setAuthFailureCallback(requireLogin);
	}, [requireLogin]);

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

	const getNodeColorCallBack = useCallback(getNodeColor, []);

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
					className: "react-node",
					style: {
						background: getNodeColorCallBack(osintNode.node_type),
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
		[setNodes, setEdges, getNodeColorCallBack],
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
		// const position = project({ x: centerX, y: centerY });
		const position = screenToFlowPosition({ x: centerX, y: centerY });

		// Add small random offset to prevent exact stacking
		const offsetX = (Math.random() - 0.5) * 40; // ±20px
		const offsetY = (Math.random() - 0.5) * 40; // ±20px

		return {
			x: Math.round(position.x + offsetX),
			y: Math.round(position.y + offsetY),
		};
	}, [screenToFlowPosition]);

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
				display: "",
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
				className: "react-node",
				style: {
					background: getNodeColorCallBack(nodeType),
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
		[getViewportCenterPosition, setNodes, getNodeColorCallBack, saveHistory],
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

			// Show context menu for URL and domain nodes
			if (nodeType === "url" || nodeType === "domain") {
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

	const handleSearchUrlscan = useCallback(() => {
		if (!contextMenu) return;

		// eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
		const nodeType = contextMenu.node.data.nodeType as string;
		// eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
		const value = contextMenu.node.data.osintNode?.value as string;

		if (value) {
			const cleanValue = value.trim();
			let searchQuery = "";

			if (nodeType === "domain") {
				// For domains, use domain: search
				searchQuery = `domain:${cleanValue}`;
			} else if (nodeType === "url") {
				// For URLs, use page.url.keyword: search with backslash escaping
				const escapedUrl = cleanValue.replace(/[:/]/g, (match) => `\\${match}`);
				searchQuery = `page.url.keyword:${escapedUrl}`;
			}

			if (searchQuery) {
				const urlscanUrl = `https://urlscan.io/search/#${searchQuery}`;
				window.open(urlscanUrl, "_blank", "noopener,noreferrer");
			}
		}

		setContextMenu(null);
	}, [contextMenu]);

	const handleSearchUrlscanDomain = useCallback(() => {
		if (!contextMenu) return;

		// eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
		const value = contextMenu.node.data.osintNode?.value as string;

		if (value) {
			const cleanValue = value.trim();
			try {
				// Parse the URL to extract the domain
				// Add protocol if not present to make URL parsing work
				const urlToParse = cleanValue.match(/^https?:\/\//)
					? cleanValue
					: `https://${cleanValue}`;
				const url = new URL(urlToParse);
				const domain = url.hostname;

				if (domain) {
					const searchQuery = `domain:${domain}`;
					const urlscanUrl = `https://urlscan.io/search/#${searchQuery}`;
					window.open(urlscanUrl, "_blank", "noopener,noreferrer");
				}
			} catch (error) {
				console.error("Failed to parse URL for domain extraction:", error);
				toast.error("Failed to extract domain from URL");
			}
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

		// For person, organisation, image, and document nodes, sync value with display

		const finalValue = hasSyncedValue(editingNodeType ?? "")
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

	const handleNodeSelect = useCallback(
		(nodeId: string) => {
			const node = nodes.find((n) => n.id === nodeId);
			if (node) {
				// Center the view on the selected node with a smooth transition
				const x = node.position.x + 100; // Offset to center of node (approximate)
				const y = node.position.y + 50;
				const zoom = getZoom();
				setCenter(x, y, { zoom, duration: 800 });

				// Briefly highlight the node by selecting it
				setNodes((nds) =>
					nds.map((n) => ({
						...n,
						selected: n.id === nodeId,
					})),
				);

				// Deselect after a brief moment
				setTimeout(() => {
					setNodes((nds) =>
						nds.map((n) => ({
							...n,
							selected: false,
						})),
					);
				}, 2000);
			}
		},
		[nodes, setCenter, getZoom, setNodes],
	);

	const handleGlobalNodeSelect = useCallback(
		async (nodeId: string, projectId: string) => {
			try {
				// Switch to the target project
				await handleProjectChange(projectId);

				// Wait a bit for the project to load, then center on the node
				setTimeout(() => {
					handleNodeSelect(nodeId);
				}, 500);
			} catch (error) {
				console.error("Failed to switch to project:", error);
				toast.error("Failed to switch to project");
			}
		},
		[handleProjectChange, handleNodeSelect],
	);

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

				// Delete nodes from backend when removed
				changes.forEach((change) => {
					if (change.type === "remove") {
						console.debug("Deleting node from backend:", change.id);
						deleteNode(change.id).catch((error) => {
							console.error("Failed to delete node:", error);
							toast.error("Failed to delete node from backend");
						});
					}
				});
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

	// Create projects map for NodeSearch
	const projectsMap = new Map<string, string>(
		availableProjects.map((p) => [p.id, p.name]),
	);

	return (
		<div className="app-container">
			<Toaster position="top-right" />
			<LoginDialog />

			<ProjectSelector
				currentProject={currentProject}
				onProjectChange={handleProjectChange}
				onCreateNew={handleCreateNewProject}
				setShowProjectManagement={setShowProjectManagement}
			/>

			<NodeSearch
				nodes={nodes}
				onNodeSelect={handleNodeSelect}
				onGlobalResultSelect={handleGlobalNodeSelect}
				currentProjectId={currentProject?.id || null}
				projects={projectsMap}
			/>

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
				className={
					movingAttachment ? "react-flow-crosshair" : "react-flow-default"
				}
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
				<div className="node-panel-content">
					<div className="node-panel-title">Add Node</div>
					{nodeTypes.map((type) => (
						<button
							type="button"
							key={type}
							onClick={() => createOSINTNode(type)}
							className="node-type-button"
							style={{ background: getNodeColorCallBack(type) }}
						>
							{NodeTypeInfo[type]?.label ?? type}
						</button>
					))}
				</div>
			</div>

			{editingNode && (
				<div
					role="dialog"
					aria-modal="true"
					className="edit-node-backdrop"
					onClick={(e) => {
						// Close on backdrop click
						if (e.target === e.currentTarget) {
							cancelNodeEdit();
						}
					}}
					onKeyDown={(e) => {
						if (e.key === "Escape") {
							cancelNodeEdit();
						}
					}}
				>
					<div className="edit-node-modal">
						<h3>Edit Node</h3>
						<div className="modal-field">
							<label htmlFor={idDisplay} className="modal-label">
								Display Name
							</label>
							<input
								type="text"
								id={idDisplay}
								value={editDisplay}
								onChange={(e) => setEditDisplay(e.target.value)}
								className="modal-input"
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

						{/* Only show value field for nodes that need it (not person, organisation, image, document) */}
						{!hasSyncedValue(editingNodeType ?? "") && (
							<div className="modal-field">
								<label htmlFor={idValue} className="modal-label">
									Value
								</label>
								<input
									type="text"
									value={editValue}
									id={idValue}
									onChange={(e) => setEditValue(e.target.value)}
									className="modal-input"
									placeholder="Actual value (e.g., email, phone number)"
								/>
							</div>
						)}

						<div className="modal-field">
							<label htmlFor={idMoreInfo} className="modal-label">
								Notes
							</label>
							<textarea
								value={editNotes}
								onChange={(e) => setEditNotes(e.target.value)}
								id={idMoreInfo}
								className="modal-textarea"
								placeholder="Additional information..."
							/>
						</div>

						{/* Attachments section - only show for saved nodes */}
						{!pendingNodes.has(editingNode) && (
							<div className="modal-field">
								<div className="modal-section-label">Attachments</div>

								{/* List of existing attachments */}
								{nodeAttachments.length > 0 && (
									<div className="attachment-list-container">
										{nodeAttachments.map((attachment) => (
											<div key={attachment.id} className="attachment-item">
												<div className="attachment-info">
													<div className="attachment-filename">
														{attachment.filename}
													</div>
													<div className="attachment-size">
														{formatFileSize(attachment.size)}
													</div>
												</div>
												<div className="attachment-actions">
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
									className={`btn btn-success ${uploadingAttachment ? "upload-label-uploading" : "upload-label-ready"}`}
								>
									{uploadingAttachment ? "Uploading..." : "Upload File"}
									<input
										type="file"
										onChange={handleFileUpload}
										disabled={uploadingAttachment}
										className="upload-input-hidden"
									/>
								</label>
							</div>
						)}

						<div className="modal-buttons-container">
							<div className="modal-buttons-group">
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
				</div>
			)}

			{/* Context menu for URL and domain nodes */}
			{contextMenu && (
				<div
					role="menuitem"
					className="context-menu"
					style={{ top: contextMenu.y, left: contextMenu.x }}
					onClick={(e) => e.stopPropagation()}
					onKeyDown={() => {}}
					tabIndex={0}
				>
					<ContextMenuItem
						node={contextMenu.node}
						onClick={handleOpenUrl}
						applicableNodeTypes={["url"]}
						icon="🔗"
						title="Open in new tab"
					/>
					<ContextMenuItem
						node={contextMenu.node}
						onClick={handleSearchUrlscan}
						applicableNodeTypes={["url", "domain"]}
						icon="🔍"
						title="Search on urlscan.io"
					/>
					<ContextMenuItem
						node={contextMenu.node}
						onClick={handleSearchUrlscanDomain}
						applicableNodeTypes={["url"]}
						icon="🌐"
						title="Search this domain on urlscan.io"
					/>
				</div>
			)}

			{/* Move attachment mode indicator */}
			{movingAttachment && (
				<div className="move-attachment-indicator">
					<span>
						📦 Moving "{movingAttachment.filename}" - Double-click a node to
						move it there
					</span>
					<button
						type="button"
						onClick={handleCancelMoveAttachment}
						className="btn btn-secondary"
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
		<AuthProvider>
			<ReactFlowProvider>
				<AppContent />
			</ReactFlowProvider>
		</AuthProvider>
	);
}
