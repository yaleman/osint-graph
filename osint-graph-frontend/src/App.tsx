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
} from 'reactflow';
import 'reactflow/dist/style.css';

const initialNodes: Node[] = [];
const initialEdges: Edge[] = [];

export default function App() {
  const [nodes, setNodes, onNodesChange] = useNodesState(initialNodes);
  const [edges, setEdges, onEdgesChange] = useEdgesState(initialEdges);
  const [showMenu, setShowMenu] = useState(false);
  const [menuPosition, setMenuPosition] = useState({ x: 0, y: 0 });

  const onConnect: OnConnect = useCallback(
    (params) => setEdges((eds) => addEdge(params, eds)),
    [setEdges]
  );

  const nodeTypes = [
    'person',
    'domain', 
    'ip',
    'phone',
    'email',
    'url'
  ];

  const onPaneClick = useCallback((event: React.MouseEvent) => {
    setMenuPosition({
      x: event.clientX,
      y: event.clientY,
    });
    setShowMenu(true);
  }, []);

  const addNode = useCallback((nodeType: string) => {
    const newNode: Node = {
      id: `${nodeType}-${Date.now()}`,
      type: 'default',
      position: { x: Math.random() * 400, y: Math.random() * 400 },
      data: { label: `${nodeType}` },
      style: {
        background: getNodeColor(nodeType),
        color: 'white',
        border: '1px solid #222',
        width: 180,
      },
    };

    setNodes((nds) => nds.concat(newNode));
    setShowMenu(false);
  }, [setNodes]);

  function getNodeColor(nodeType: string): string {
    const colors: Record<string, string> = {
      person: '#3b82f6',
      domain: '#f59e0b', 
      ip: '#ef4444',
      phone: '#8b5cf6',
      email: '#ec4899',
      url: '#06b6d4'
    };
    return colors[nodeType] || '#6b7280';
  }

  return (
    <div style={{ width: '100vw', height: '100vh', position: 'relative' }}>
      <ReactFlow
        nodes={nodes}
        edges={edges}
        onNodesChange={onNodesChange}
        onEdgesChange={onEdgesChange}
        onConnect={onConnect}
        onPaneClick={onPaneClick}
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
          }}
        >
          <div style={{ marginBottom: '8px', fontWeight: 'bold' }}>Add Node:</div>
          {nodeTypes.map((type) => (
            <button
              key={type}
              onClick={() => addNode(type)}
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
              }}
            >
              {type}
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
    </div>
  );
}