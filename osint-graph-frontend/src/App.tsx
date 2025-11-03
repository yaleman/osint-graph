import React, { useCallback, useState, useEffect, useRef } from 'react';
import ReactFlow, {
  Node,
  Edge,
  addEdge,
  Background,
  Controls,
  MiniMap,
  useNodesState,
  useEdgesState,
  OnConnect,
  OnNodesChange,
} from 'reactflow';
import 'reactflow/dist/style.css';
import { v4 as uuidv4 } from 'uuid';
import toast, { Toaster } from 'react-hot-toast';
import { createNode, updateNode, createProject, fetchNodesByProject, getProject, fetchProjects, createNodeLink, fetchNodeLinksByProject } from './api';
import type { OSINTNode, Project } from './types';
import { NodeTypeInfo } from './types';
import { ProjectMismatchDialog } from './components/ProjectMismatchDialog';
import { ProjectSelector } from './components/ProjectSelector';

const initialNodes: Node[] = [];
const initialEdges: Edge[] = [];

const PROJECT_ID_KEY = 'osint-graph-project-id';
const DEBOUNCE_DELAY = 100; // ms

export default function App() {
  const [nodes, setNodes, onNodesChange] = useNodesState(initialNodes);
  const [edges, setEdges, onEdgesChange] = useEdgesState(initialEdges);
  const [isPanelCollapsed, setIsPanelCollapsed] = useState(false);
  const [editingNode, setEditingNode] = useState<string | null>(null);
  const [editDisplay, setEditDisplay] = useState('');
  const [editValue, setEditValue] = useState('');
  const [editNotes, setEditNotes] = useState('');
  const [currentProject, setCurrentProject] = useState<Project | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [showMismatchDialog, setShowMismatchDialog] = useState(false);
  const [availableProjects, setAvailableProjects] = useState<Project[]>([]);

  // Refs for debouncing node updates
  const pendingUpdatesRef = useRef<Map<string, number>>(new Map());
  const latestNodeDataRef = useRef<Map<string, OSINTNode>>(new Map());

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
      updateNode(nodeData).catch(error => {
        console.error('Failed to update node:', error);
        toast.error('Failed to update node');
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
        updateNode(latestData).catch(error => {
          console.error('Failed to update node:', error);
          toast.error('Failed to update node');
        });
        latestNodeDataRef.current.delete(nodeId);
      }
      pendingUpdatesRef.current.delete(nodeId);
    }, DEBOUNCE_DELAY);

    pendingUpdatesRef.current.set(nodeId, timeout);
  }, []);

  // Cleanup: flush pending updates on unmount
  useEffect(() => {
    return () => {
      flushPendingUpdates();
    };
  }, [flushPendingUpdates]);

  const onConnect: OnConnect = useCallback(
    async (params) => {
      // Add edge locally
      setEdges((eds) => addEdge(params, eds));

      // Validate source and target
      if (!params.source || !params.target) {
        console.error('Invalid connection: missing source or target');
        return;
      }

      // Save to backend
      const projectId = localStorage.getItem(PROJECT_ID_KEY);
      if (!projectId || projectId === "undefined" || projectId.trim() === "") {
        console.error('No valid project ID found when creating node link');
        toast.error('Cannot create link: no project selected');
        return;
      }

      try {
        const nodeLink = {
          id: uuidv4(),
          left: params.source,
          right: params.target,
          project_id: projectId,
          linktype: 'Directional' as const,
        };

        await createNodeLink(nodeLink);
        toast.success('Connection saved');
      } catch (error) {
        console.error('Failed to save connection:', error);
        toast.error('Failed to save connection to backend');
      }
    },
    [setEdges]
  );

  const getNodeColor = useCallback((nodeType: string): string => {
    const colors: Record<string, string> = {
      person: '#3b82f6',
      domain: '#f59e0b',
      ip: '#ef4444',
      phone: '#8b5cf6',
      email: '#ec4899',
      url: '#06b6d4',
      image: '#10b981',
      location: '#84cc16',
      organization: '#f97316',
      document: '#6b7280'
    };
    return colors[nodeType] || '#6b7280';
  }, []);

  // Helper function to load nodes and edges for a project
  const loadProjectData = useCallback(async (projectId: string) => {
    try {
      // Load nodes
      const existingNodes = await fetchNodesByProject(projectId);
      const reactFlowNodes: Node[] = existingNodes.map(osintNode => ({
        id: osintNode.id,
        type: 'default',
        position: { x: osintNode.pos_x || 100, y: osintNode.pos_y || 100 },
        data: {
          label: osintNode.display,
          nodeType: osintNode.node_type,
          osintNode: osintNode
        },
        style: {
          background: getNodeColor(osintNode.node_type),
          color: 'white',
          border: '1px solid #222',
          width: 180,
          cursor: 'pointer',
        },
      }));
      setNodes(reactFlowNodes);

      // Load node links
      const existingLinks = await fetchNodeLinksByProject(projectId);
      const reactFlowEdges: Edge[] = existingLinks.map(nodeLink => ({
        id: nodeLink.id,
        source: nodeLink.left,
        target: nodeLink.right,
        type: nodeLink.linktype === 'Directional' ? 'default' : 'straight',
      }));
      setEdges(reactFlowEdges);
    } catch (error) {
      console.error('Failed to load project data:', error);
      throw error;
    }
  }, [setNodes, setEdges, getNodeColor]);

  // Initialize project on component mount
  useEffect(() => {
    const initializeProject = async () => {
      try {
        setIsLoading(true);
        const projectId = localStorage.getItem(PROJECT_ID_KEY);

        // Check for valid project ID (not null, not "undefined", not empty)
        if (projectId && projectId !== "undefined" && projectId.trim() !== "") {
          // Validate project exists in backend
          const project = await getProject(projectId);

          if (project) {
            // Project exists, load its data
            setCurrentProject(project);
            try {
              await loadProjectData(projectId);
              setIsLoading(false);
            } catch (error) {
              console.error('Failed to load project data:', error);
              toast.error('Failed to load project data');
              setIsLoading(false);
            }
          } else {
            // Project doesn't exist in backend, show mismatch dialog
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
        console.error('Failed to initialize project:', error);
        toast.error('Failed to initialize project');
        setIsLoading(false);
      }
    };

    initializeProject();
  }, [loadProjectData]);

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
      console.error('Failed to create project:', error);
      toast.error('Failed to create new project');
    }
  }, [setNodes, setEdges]);

  const handleSelectExisting = useCallback(() => {
    setShowMismatchDialog(false);
    // The user can use the project selector to choose a project
    toast('Please select a project from the dropdown');
  }, []);

  const handleProjectChange = useCallback(async (projectId: string) => {
    try {
      const project = await getProject(projectId);
      if (project) {
        localStorage.setItem(PROJECT_ID_KEY, projectId);
        setCurrentProject(project);

        // Load nodes and links for this project
        await loadProjectData(projectId);
        toast.success(`Switched to project: ${project.name}`);
      }
    } catch (error) {
      console.error('Failed to switch project:', error);
      toast.error('Failed to switch project');
    }
  }, [loadProjectData]);

  // Helper function to check if a position overlaps with existing nodes
  const findNonOverlappingPosition = useCallback((bounds: DOMRect, existingNodes: Node[]): { x: number; y: number } => {
    const PANEL_WIDTH = 230; // 220px panel + 10px margin
    const NODE_WIDTH = 180;
    const NODE_HEIGHT = 80; // Approximate node height
    const MIN_DISTANCE = 200; // Minimum distance from other nodes
    const MAX_ATTEMPTS = 50;

    // Calculate safe area (avoiding right panel)
    const safeWidth = bounds.width - PANEL_WIDTH - NODE_WIDTH;
    const safeHeight = bounds.height - NODE_HEIGHT;

    // Center of safe area
    const centerX = safeWidth / 2;
    const centerY = safeHeight / 2;

    const checkOverlap = (x: number, y: number): boolean => {
      return existingNodes.some(node => {
        const dx = Math.abs(node.position.x - x);
        const dy = Math.abs(node.position.y - y);
        const distance = Math.sqrt(dx * dx + dy * dy);
        return distance < MIN_DISTANCE;
      });
    };

    // Try to find a non-overlapping position
    for (let attempt = 0; attempt < MAX_ATTEMPTS; attempt++) {
      // Start near center and spiral outward
      const angle = (attempt * 137.5) * (Math.PI / 180); // Golden angle
      const radius = 50 + (attempt * 30); // Spiral outward
      const x = centerX + Math.cos(angle) * radius;
      const y = centerY + Math.sin(angle) * radius;

      // Ensure position is within safe bounds
      if (x >= 0 && x <= safeWidth && y >= 0 && y <= safeHeight) {
        if (!checkOverlap(x, y)) {
          return { x: Math.round(x), y: Math.round(y) };
        }
      }
    }

    // Fallback: return center with random offset
    const fallbackX = centerX + (Math.random() * 100 - 50);
    const fallbackY = centerY + (Math.random() * 100 - 50);
    return { x: Math.round(fallbackX), y: Math.round(fallbackY) };
  }, []);

  const createOSINTNode = useCallback(async (nodeType: string) => {
    let projectId = localStorage.getItem(PROJECT_ID_KEY);
    if (!projectId || projectId === "undefined" || projectId.trim() === "") {
      console.log('No valid project ID found, creating new project...');
      try {
        const project = await createProject();
        projectId = project.id;
        localStorage.setItem(PROJECT_ID_KEY, projectId);
        setCurrentProject(project);
      } catch (error) {
        console.error('Failed to create new project:', error);
        return;
      }
    }

    // Find a good position that avoids the right panel and existing nodes
    const reactFlowBounds = document.querySelector('.react-flow')?.getBoundingClientRect();
    const bounds = reactFlowBounds || { width: 1200, height: 800 } as DOMRect;
    const position = findNonOverlappingPosition(bounds, nodes);
    const x = position.x;
    const y = position.y;

    const nodeId = uuidv4();
    
    const osintNode: OSINTNode = {
      id: nodeId,
      project_id: projectId,
      node_type: nodeType,
      display: `New ${NodeTypeInfo[nodeType]?.label || nodeType}`,
      value: '',
      updated: new Date().toISOString(),
      pos_x: Math.round(x),
      pos_y: Math.round(y),
    };

    const newReactFlowNode: Node = {
      id: nodeId,
      type: 'default',
      position: { x, y },
      data: { 
        label: osintNode.display,
        nodeType: nodeType,
        osintNode: osintNode
      },
      style: {
        background: getNodeColor(nodeType),
        color: 'white',
        border: '1px solid #222',
        width: 180,
        cursor: 'pointer',
      },
    };

    // Update local state immediately
    setNodes((nds) => nds.concat(newReactFlowNode));

    // Automatically open edit UI for the new node
    setEditingNode(nodeId);
    setEditDisplay(osintNode.display);
    setEditValue('');

    // Save to backend
    try {
      await createNode(osintNode);
      toast.success('Node created successfully');
    } catch (error) {
      console.error('Failed to save node to backend:', error);
      toast.error('Failed to save node to backend');
    }
  }, [nodes, findNonOverlappingPosition, setNodes, getNodeColor]);

  const handleNodeDoubleClick = useCallback((event: React.MouseEvent, node: Node) => {
    event.stopPropagation();
    setEditingNode(node.id);
    setEditDisplay(node.data.osintNode?.display || node.data.label || '');
    setEditValue(node.data.osintNode?.value || '');
    setEditNotes(node.data.osintNode?.notes || '');
  }, []);

  const saveNodeEdit = useCallback(async () => {
    if (!editingNode) return;

    setNodes((nds) =>
      nds.map((node) => {
        if (node.id === editingNode) {
          const updatedOSINTNode: OSINTNode = {
            ...node.data.osintNode,
            display: editDisplay,
            value: editValue,
            notes: editNotes || undefined,
            updated: new Date().toISOString(),
          };

          // Use debounced update for node edits
          debouncedUpdateNode(updatedOSINTNode);

          return {
            ...node,
            data: {
              ...node.data,
              label: editDisplay,
              osintNode: updatedOSINTNode
            }
          };
        }
        return node;
      })
    );

    setEditingNode(null);
    setEditDisplay('');
    setEditValue('');
    setEditNotes('');
  }, [editingNode, editDisplay, editValue, editNotes, setNodes, debouncedUpdateNode]);

  const handleNodesChange: OnNodesChange = useCallback((changes) => {
    onNodesChange(changes);

    // Update position changes in backend
    changes.forEach(change => {
      if (change.type === 'position' && change.position) {
        const node = nodes.find(n => n.id === change.id);
        if (node?.data.osintNode) {
          const projectId = localStorage.getItem(PROJECT_ID_KEY);
          if (!projectId || projectId === "undefined" || projectId.trim() === "") {
            console.error('No valid project ID found when updating node position');
            return;
          }

          const updatedNode: OSINTNode = {
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
  }, [onNodesChange, nodes, debouncedUpdateNode]);

  if (isLoading) {
    return (
      <div style={{ 
        width: '100vw', 
        height: '100vh', 
        display: 'flex', 
        alignItems: 'center', 
        justifyContent: 'center',
        fontSize: '18px'
      }}>
        Initializing OSINT Graph...
      </div>
    );
  }

  return (
    <div style={{ width: '100vw', height: '100vh', position: 'relative' }}>
      <Toaster position="top-right" />

      <ProjectSelector
        currentProject={currentProject}
        onProjectChange={handleProjectChange}
        onCreateNew={handleCreateNewProject}
      />

      {showMismatchDialog && (
        <ProjectMismatchDialog
          onCreateNew={handleCreateNewProject}
          onSelectExisting={handleSelectExisting}
          projects={availableProjects}
        />
      )}

      <ReactFlow
        nodes={nodes}
        edges={edges}
        onNodesChange={handleNodesChange}
        onEdgesChange={onEdgesChange}
        onConnect={onConnect}
        onNodeDoubleClick={handleNodeDoubleClick}
        fitView
      >
        <Controls />
        <MiniMap />
        <Background />
      </ReactFlow>

      {/* Right-side collapsible panel for adding nodes */}
      <div
        style={{
          position: 'fixed',
          top: '60px',
          right: isPanelCollapsed ? '-220px' : '10px',
          width: '220px',
          background: 'white',
          border: '1px solid #ccc',
          borderRadius: '8px',
          boxShadow: '0 4px 12px rgba(0, 0, 0, 0.15)',
          zIndex: 1000,
          transition: 'right 0.3s ease',
          maxHeight: 'calc(100vh - 80px)',
          display: 'flex',
          flexDirection: 'column',
        }}
      >
        {/* Collapse/Expand button */}
        <button
          onClick={() => setIsPanelCollapsed(!isPanelCollapsed)}
          style={{
            position: 'absolute',
            left: '-32px',
            top: '10px',
            width: '32px',
            height: '40px',
            background: 'white',
            border: '1px solid #ccc',
            borderRight: 'none',
            borderRadius: '8px 0 0 8px',
            cursor: 'pointer',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            fontSize: '18px',
            boxShadow: '-2px 2px 8px rgba(0, 0, 0, 0.1)',
          }}
          title={isPanelCollapsed ? 'Expand panel' : 'Collapse panel'}
        >
          {isPanelCollapsed ? '◀' : '▶'}
        </button>

        {/* Panel content */}
        <div style={{ padding: '16px', overflowY: 'auto', flex: 1 }}>
          <div style={{ marginBottom: '12px', fontWeight: 'bold', fontSize: '16px' }}>
            Add Node
          </div>
          {nodeTypes.map((type) => (
            <button
              key={type}
              onClick={() => createOSINTNode(type)}
              style={{
                display: 'block',
                width: '100%',
                padding: '10px 12px',
                margin: '4px 0',
                border: 'none',
                borderRadius: '6px',
                background: getNodeColor(type),
                color: 'white',
                cursor: 'pointer',
                textAlign: 'left',
                fontSize: '14px',
                fontWeight: '500',
                transition: 'transform 0.1s ease, box-shadow 0.1s ease',
              }}
              onMouseEnter={(e) => {
                e.currentTarget.style.transform = 'translateY(-2px)';
                e.currentTarget.style.boxShadow = '0 4px 8px rgba(0, 0, 0, 0.2)';
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.transform = 'translateY(0)';
                e.currentTarget.style.boxShadow = 'none';
              }}
            >
              {NodeTypeInfo[type]?.label || type}
            </button>
          ))}
        </div>
      </div>

      {editingNode && (
        <div
          style={{
            position: 'fixed',
            top: '50%',
            left: '50%',
            transform: 'translate(-50%, -50%)',
            background: 'white',
            border: '1px solid #ccc',
            borderRadius: '8px',
            padding: '20px',
            boxShadow: '0 4px 12px rgba(0, 0, 0, 0.15)',
            zIndex: 1001,
            minWidth: '300px',
          }}
        >
          <h3>Edit Node</h3>
          <div style={{ marginBottom: '10px' }}>
            <label style={{ display: 'block', marginBottom: '4px', fontWeight: '500', fontSize: '14px' }}>
              Display Name
            </label>
            <input
              type="text"
              value={editDisplay}
              onChange={(e) => setEditDisplay(e.target.value)}
              style={{
                width: '100%',
                padding: '8px',
                border: '1px solid #ccc',
                borderRadius: '4px',
                boxSizing: 'border-box',
              }}
              placeholder="Name shown on graph"
              autoFocus
              onKeyDown={(e) => {
                if (e.key === 'Escape') {
                  setEditingNode(null);
                  setEditDisplay('');
                  setEditValue('');
                  setEditNotes('');
                }
              }}
            />
          </div>

          <div style={{ marginBottom: '10px' }}>
            <label style={{ display: 'block', marginBottom: '4px', fontWeight: '500', fontSize: '14px' }}>
              Value
            </label>
            <input
              type="text"
              value={editValue}
              onChange={(e) => setEditValue(e.target.value)}
              style={{
                width: '100%',
                padding: '8px',
                border: '1px solid #ccc',
                borderRadius: '4px',
                boxSizing: 'border-box',
              }}
              placeholder="Actual value (e.g., email, phone number)"
            />
          </div>

          <div style={{ marginBottom: '16px' }}>
            <label style={{ display: 'block', marginBottom: '4px', fontWeight: '500', fontSize: '14px' }}>
              Notes
            </label>
            <textarea
              value={editNotes}
              onChange={(e) => setEditNotes(e.target.value)}
              style={{
                width: '100%',
                padding: '8px',
                border: '1px solid #ccc',
                borderRadius: '4px',
                boxSizing: 'border-box',
                minHeight: '80px',
                resize: 'vertical',
                fontFamily: 'inherit',
              }}
              placeholder="Additional information..."
            />
          </div>

          <div style={{ display: 'flex', gap: '10px' }}>
            <button
              onClick={saveNodeEdit}
              style={{
                padding: '8px 16px',
                background: '#3b82f6',
                color: 'white',
                border: 'none',
                borderRadius: '4px',
                cursor: 'pointer',
              }}
            >
              Save
            </button>
            <button
              onClick={() => {
                setEditingNode(null);
                setEditDisplay('');
                setEditValue('');
                setEditNotes('');
              }}
              style={{
                padding: '8px 16px',
                background: '#6b7280',
                color: 'white',
                border: 'none',
                borderRadius: '4px',
                cursor: 'pointer',
              }}
            >
              Cancel
            </button>
          </div>
        </div>
      )}
    </div>
  );
}