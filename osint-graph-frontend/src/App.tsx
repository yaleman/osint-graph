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
import { createNode, updateNode, createProject, fetchNodesByProject, getProject, fetchProjects } from './api';
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
  const [showMenu, setShowMenu] = useState(false);
  const [menuPosition, setMenuPosition] = useState({ x: 0, y: 0 });
  const [editingNode, setEditingNode] = useState<string | null>(null);
  const [editDisplay, setEditDisplay] = useState('');
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
    (params) => setEdges((eds) => addEdge(params, eds)),
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
              setIsLoading(false);
            } catch (error) {
              console.error('Failed to load nodes:', error);
              toast.error('Failed to load nodes for this project');
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
  }, [setNodes, getNodeColor]);

  const nodeTypes = Object.keys(NodeTypeInfo);

  const handleCreateNewProject = useCallback(async () => {
    try {
      const project = await createProject();
      localStorage.setItem(PROJECT_ID_KEY, project.id);
      setCurrentProject(project);
      setShowMismatchDialog(false);
      setNodes([]);
      toast.success(`Created new project: ${project.name}`);
    } catch (error) {
      console.error('Failed to create project:', error);
      toast.error('Failed to create new project');
    }
  }, [setNodes]);

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

        // Load nodes for this project
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
        toast.success(`Switched to project: ${project.name}`);
      }
    } catch (error) {
      console.error('Failed to switch project:', error);
      toast.error('Failed to switch project');
    }
  }, [setNodes, getNodeColor]);

  const onPaneClick = useCallback((event: React.MouseEvent) => {
    setMenuPosition({
      x: event.clientX,
      y: event.clientY,
    });
    setShowMenu(true);
    setEditingNode(null);
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

    const reactFlowBounds = document.querySelector('.react-flow')?.getBoundingClientRect();
    const x = reactFlowBounds ? menuPosition.x - reactFlowBounds.left - 90 : 100;
    const y = reactFlowBounds ? menuPosition.y - reactFlowBounds.top - 40 : 100;

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
    setShowMenu(false);

    // Automatically open edit UI for the new node
    setEditingNode(nodeId);
    setEditDisplay(osintNode.display);

    // Save to backend
    try {
      await createNode(osintNode);
      toast.success('Node created successfully');
    } catch (error) {
      console.error('Failed to save node to backend:', error);
      toast.error('Failed to save node to backend');
    }
  }, [menuPosition, setNodes, getNodeColor]);

  const handleNodeDoubleClick = useCallback((event: React.MouseEvent, node: Node) => {
    event.stopPropagation();
    setEditingNode(node.id);
    setEditDisplay(node.data.osintNode?.display || node.data.label || '');
  }, []);

  const saveNodeEdit = useCallback(async () => {
    if (!editingNode) return;

    setNodes((nds) => 
      nds.map((node) => {
        if (node.id === editingNode) {
          const updatedOSINTNode = {
            ...node.data.osintNode,
            display: editDisplay,
            updated: new Date().toISOString(),
          };

          // Use debounced update for display name changes
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
  }, [editingNode, editDisplay, setNodes, debouncedUpdateNode]);

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
        onPaneClick={onPaneClick}
        onNodeDoubleClick={handleNodeDoubleClick}
        fitView
      >
        <Controls />
        <MiniMap />
        <Background />
      </ReactFlow>

      {showMenu && (
        <div
          style={{
            position: 'fixed',
            left: menuPosition.x,
            top: menuPosition.y,
            background: 'white',
            border: '1px solid #ccc',
            borderRadius: '8px',
            padding: '8px',
            boxShadow: '0 4px 12px rgba(0, 0, 0, 0.15)',
            zIndex: 1000,
            maxHeight: '400px',
            overflowY: 'auto',
          }}
        >
          <div style={{ marginBottom: '8px', fontWeight: 'bold' }}>Add Node:</div>
          {nodeTypes.map((type) => (
            <button
              key={type}
              onClick={() => createOSINTNode(type)}
              style={{
                display: 'block',
                width: '100%',
                padding: '8px 12px',
                margin: '2px 0',
                border: 'none',
                borderRadius: '4px',
                background: getNodeColor(type),
                color: 'white',
                cursor: 'pointer',
                textAlign: 'left',
              }}
            >
              {NodeTypeInfo[type]?.label || type}
            </button>
          ))}
          <button
            onClick={() => setShowMenu(false)}
            style={{
              display: 'block',
              width: '100%',
              padding: '4px',
              margin: '4px 0 0 0',
              border: '1px solid #ccc',
              borderRadius: '4px',
              background: 'white',
              cursor: 'pointer',
            }}
          >
            Cancel
          </button>
        </div>
      )}

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
          <input
            type="text"
            value={editDisplay}
            onChange={(e) => setEditDisplay(e.target.value)}
            style={{
              width: '100%',
              padding: '8px',
              border: '1px solid #ccc',
              borderRadius: '4px',
              marginBottom: '10px',
            }}
            placeholder="Enter display name"
            autoFocus
            onKeyDown={(e) => {
              if (e.key === 'Enter') saveNodeEdit();
              if (e.key === 'Escape') {
                setEditingNode(null);
                setEditDisplay('');
              }
            }}
          />
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