import { useCallback, useState } from 'react';
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
import { createNode, updateNode } from './api';
import type { OSINTNode } from './types';
import { NodeTypeInfo } from './types';

const initialNodes: Node[] = [];
const initialEdges: Edge[] = [];

// Default project ID for demo
const DEFAULT_PROJECT_ID = '550e8400-e29b-41d4-a716-446655440000';

export default function App() {
  const [nodes, setNodes, onNodesChange] = useNodesState(initialNodes);
  const [edges, setEdges, onEdgesChange] = useEdgesState(initialEdges);
  const [showMenu, setShowMenu] = useState(false);
  const [menuPosition, setMenuPosition] = useState({ x: 0, y: 0 });
  const [editingNode, setEditingNode] = useState<string | null>(null);
  const [editDisplay, setEditDisplay] = useState('');

  const onConnect: OnConnect = useCallback(
    (params) => setEdges((eds) => addEdge(params, eds)),
    [setEdges]
  );

  const nodeTypes = Object.keys(NodeTypeInfo);

  const onPaneClick = useCallback((event: React.MouseEvent) => {
    setMenuPosition({
      x: event.clientX,
      y: event.clientY,
    });
    setShowMenu(true);
    setEditingNode(null);
  }, []);

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

  const createOSINTNode = useCallback(async (nodeType: string) => {
    const reactFlowBounds = document.querySelector('.react-flow')?.getBoundingClientRect();
    const x = reactFlowBounds ? menuPosition.x - reactFlowBounds.left - 90 : 100;
    const y = reactFlowBounds ? menuPosition.y - reactFlowBounds.top - 40 : 100;

    const nodeId = uuidv4();
    
    const osintNode: OSINTNode = {
      id: nodeId,
      project_id: DEFAULT_PROJECT_ID,
      node_type: nodeType,
      display: `New ${NodeTypeInfo[nodeType]?.label || nodeType}`,
      value: '',
      updated: new Date().toISOString(),
      notes: '',
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
      console.log('Node saved to backend');
    } catch (error) {
      console.error('Failed to save node to backend:', error);
      // Optionally show user notification
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
          
          // Update backend
          updateNode(updatedOSINTNode).catch(error => 
            console.error('Failed to update node in backend:', error)
          );

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
  }, [editingNode, editDisplay, setNodes]);

  const handleNodesChange: OnNodesChange = useCallback((changes) => {
    onNodesChange(changes);
    
    // Update position changes in backend
    changes.forEach(change => {
      if (change.type === 'position' && change.position) {
        const node = nodes.find(n => n.id === change.id);
        if (node?.data.osintNode) {
          const updatedNode = {
            ...node.data.osintNode,
            pos_x: Math.round(change.position.x),
            pos_y: Math.round(change.position.y),
            updated: new Date().toISOString(),
          };
          updateNode(updatedNode).catch(error => 
            console.error('Failed to update node position:', error)
          );
        }
      }
    });
  }, [onNodesChange, nodes]);

  return (
    <div style={{ width: '100vw', height: '100vh', position: 'relative' }}>
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