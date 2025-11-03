import type { Project } from '../types';

interface ProjectMismatchDialogProps {
  onCreateNew: () => void;
  onSelectExisting: () => void;
  projects: Project[];
}

export function ProjectMismatchDialog({ onCreateNew, onSelectExisting, projects }: ProjectMismatchDialogProps) {
  return (
    <div
      style={{
        position: 'fixed',
        top: 0,
        left: 0,
        right: 0,
        bottom: 0,
        backgroundColor: 'rgba(0, 0, 0, 0.5)',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        zIndex: 2000,
      }}
    >
      <div
        style={{
          background: 'white',
          borderRadius: '8px',
          padding: '24px',
          boxShadow: '0 4px 12px rgba(0, 0, 0, 0.15)',
          maxWidth: '500px',
          width: '90%',
        }}
      >
        <h2 style={{ marginTop: 0, marginBottom: '16px' }}>Project Not Found</h2>
        <p style={{ marginBottom: '20px', color: '#666' }}>
          The project stored in your browser doesn't exist in the backend.
          This might happen if the database was reset or the project was deleted.
        </p>

        <div style={{ marginBottom: '16px' }}>
          <p style={{ fontWeight: 'bold', marginBottom: '8px' }}>What would you like to do?</p>
        </div>

        <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
          <button
            onClick={onCreateNew}
            style={{
              padding: '12px 16px',
              background: '#3b82f6',
              color: 'white',
              border: 'none',
              borderRadius: '4px',
              cursor: 'pointer',
              fontSize: '14px',
              fontWeight: '500',
            }}
          >
            Create New Project
          </button>

          {projects.length > 0 && (
            <button
              onClick={onSelectExisting}
              style={{
                padding: '12px 16px',
                background: '#6b7280',
                color: 'white',
                border: 'none',
                borderRadius: '4px',
                cursor: 'pointer',
                fontSize: '14px',
                fontWeight: '500',
              }}
            >
              Select Existing Project ({projects.length} available)
            </button>
          )}
        </div>

        {projects.length === 0 && (
          <p style={{ marginTop: '16px', fontSize: '12px', color: '#999', textAlign: 'center' }}>
            No existing projects found in the backend.
          </p>
        )}
      </div>
    </div>
  );
}
