
export interface NodeTypeMenuProps {
  x: number;
  y: number;
  onSelect: (nodeType: string) => void;
  onClose: () => void;
}

export interface NodeTypeOption {
  type: string;
  label: string;
  description: string;
}

export const nodeTypeOptions: NodeTypeOption[] = [
  { type: 'person', label: 'Person', description: 'Individual person' },
  { type: 'image', label: 'Image File', description: 'Image or photo file' },
  { type: 'domain', label: 'Domain Name', description: 'Internet domain name' },
  { type: 'ip', label: 'IP Address', description: 'Internet Protocol address' },
  { type: 'phone', label: 'Phone Number', description: 'Telephone number' },
  { type: 'url', label: 'URL', description: 'Web address/link' },
  { type: 'email', label: 'Email', description: 'Email address' },
  { type: 'location', label: 'Location', description: 'Physical location' },
  { type: 'organization', label: 'Organization', description: 'Company or group' },
  { type: 'document', label: 'Document', description: 'File or document' },
];

export function NodeTypeMenu({ x, y, onSelect, onClose }: NodeTypeMenuProps) {
  return (
    <div
      className="node-type-menu"
      style={{
        position: 'absolute',
        left: x,
        top: y,
        background: 'white',
        border: '1px solid #ccc',
        borderRadius: '8px',
        boxShadow: '0 4px 12px rgba(0, 0, 0, 0.15)',
        zIndex: 1000,
        minWidth: '200px',
        maxHeight: '400px',
        overflowY: 'auto',
      }}
    >
      <div style={{ padding: '8px 0' }}>
        <div style={{ padding: '8px 16px', fontSize: '14px', fontWeight: 'bold', borderBottom: '1px solid #eee' }}>
          Select Node Type
        </div>
        {nodeTypeOptions.map((option) => (
          <div
            key={option.type}
            className="node-type-option"
            style={{
              padding: '12px 16px',
              cursor: 'pointer',
              borderBottom: '1px solid #f5f5f5',
            }}
            onClick={() => onSelect(option.type)}
            onMouseEnter={(e) => {
              e.currentTarget.style.backgroundColor = '#f0f0f0';
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.backgroundColor = 'transparent';
            }}
          >
            <div style={{ fontWeight: '500', marginBottom: '2px' }}>
              {option.label}
            </div>
            <div style={{ fontSize: '12px', color: '#666' }}>
              {option.description}
            </div>
          </div>
        ))}
        <div
          style={{
            padding: '8px 16px',
            cursor: 'pointer',
            textAlign: 'center',
            borderTop: '1px solid #eee',
            fontSize: '12px',
            color: '#999',
          }}
          onClick={onClose}
        >
          Cancel
        </div>
      </div>
    </div>
  );
}