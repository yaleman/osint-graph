import type { NodeProps } from "reactflow";
import { Handle, Position } from "reactflow";

export interface OSINTNodeData {
  label: string;
  content?: string;
  metadata?: Record<string, any>;
}

const NodeContainer = ({ children, className = "", style = {} }: { 
  children: React.ReactNode; 
  className?: string; 
  style?: React.CSSProperties;
}) => (
  <div
    className={`react-flow__node-default ${className}`}
    style={{
      padding: '8px 12px',
      border: '2px solid',
      borderRadius: '8px',
      background: 'white',
      minWidth: '120px',
      fontSize: '12px',
      ...style,
    }}
  >
    {children}
    <Handle type="source" position={Position.Right} />
    <Handle type="target" position={Position.Left} />
  </div>
);

export function PersonNode({ data }: NodeProps<OSINTNodeData>) {
  return (
    <NodeContainer style={{ borderColor: '#3b82f6', color: '#1e40af' }}>
      <div style={{ fontWeight: 'bold', marginBottom: '4px' }}>👤 Person</div>
      <div>{data.label}</div>
      {data.content && <div style={{ fontSize: '10px', color: '#666' }}>{data.content}</div>}
    </NodeContainer>
  );
}

export function ImageNode({ data }: NodeProps<OSINTNodeData>) {
  return (
    <NodeContainer style={{ borderColor: '#10b981', color: '#065f46' }}>
      <div style={{ fontWeight: 'bold', marginBottom: '4px' }}>🖼️ Image</div>
      <div>{data.label}</div>
      {data.content && <div style={{ fontSize: '10px', color: '#666' }}>{data.content}</div>}
    </NodeContainer>
  );
}

export function DomainNode({ data }: NodeProps<OSINTNodeData>) {
  return (
    <NodeContainer style={{ borderColor: '#f59e0b', color: '#92400e' }}>
      <div style={{ fontWeight: 'bold', marginBottom: '4px' }}>🌐 Domain</div>
      <div>{data.label}</div>
      {data.content && <div style={{ fontSize: '10px', color: '#666' }}>{data.content}</div>}
    </NodeContainer>
  );
}

export function IPNode({ data }: NodeProps<OSINTNodeData>) {
  return (
    <NodeContainer style={{ borderColor: '#ef4444', color: '#991b1b' }}>
      <div style={{ fontWeight: 'bold', marginBottom: '4px' }}>📡 IP Address</div>
      <div>{data.label}</div>
      {data.content && <div style={{ fontSize: '10px', color: '#666' }}>{data.content}</div>}
    </NodeContainer>
  );
}

export function PhoneNode({ data }: NodeProps<OSINTNodeData>) {
  return (
    <NodeContainer style={{ borderColor: '#8b5cf6', color: '#5b21b6' }}>
      <div style={{ fontWeight: 'bold', marginBottom: '4px' }}>📱 Phone</div>
      <div>{data.label}</div>
      {data.content && <div style={{ fontSize: '10px', color: '#666' }}>{data.content}</div>}
    </NodeContainer>
  );
}

export function URLNode({ data }: NodeProps<OSINTNodeData>) {
  return (
    <NodeContainer style={{ borderColor: '#06b6d4', color: '#164e63' }}>
      <div style={{ fontWeight: 'bold', marginBottom: '4px' }}>🔗 URL</div>
      <div>{data.label}</div>
      {data.content && <div style={{ fontSize: '10px', color: '#666' }}>{data.content}</div>}
    </NodeContainer>
  );
}

export function EmailNode({ data }: NodeProps<OSINTNodeData>) {
  return (
    <NodeContainer style={{ borderColor: '#ec4899', color: '#be185d' }}>
      <div style={{ fontWeight: 'bold', marginBottom: '4px' }}>📧 Email</div>
      <div>{data.label}</div>
      {data.content && <div style={{ fontSize: '10px', color: '#666' }}>{data.content}</div>}
    </NodeContainer>
  );
}

export function LocationNode({ data }: NodeProps<OSINTNodeData>) {
  return (
    <NodeContainer style={{ borderColor: '#84cc16', color: '#365314' }}>
      <div style={{ fontWeight: 'bold', marginBottom: '4px' }}>📍 Location</div>
      <div>{data.label}</div>
      {data.content && <div style={{ fontSize: '10px', color: '#666' }}>{data.content}</div>}
    </NodeContainer>
  );
}

export function OrganizationNode({ data }: NodeProps<OSINTNodeData>) {
  return (
    <NodeContainer style={{ borderColor: '#f97316', color: '#9a3412' }}>
      <div style={{ fontWeight: 'bold', marginBottom: '4px' }}>🏢 Organization</div>
      <div>{data.label}</div>
      {data.content && <div style={{ fontSize: '10px', color: '#666' }}>{data.content}</div>}
    </NodeContainer>
  );
}

export function DocumentNode({ data }: NodeProps<OSINTNodeData>) {
  return (
    <NodeContainer style={{ borderColor: '#6b7280', color: '#374151' }}>
      <div style={{ fontWeight: 'bold', marginBottom: '4px' }}>📄 Document</div>
      <div>{data.label}</div>
      {data.content && <div style={{ fontSize: '10px', color: '#666' }}>{data.content}</div>}
    </NodeContainer>
  );
}