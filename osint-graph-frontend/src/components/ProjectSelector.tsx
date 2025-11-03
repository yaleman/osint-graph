import React, { useState, useEffect } from 'react';
import { fetchProjects } from '../api';
import type { Project } from '../types';

interface ProjectSelectorProps {
  currentProject: Project | null;
  onProjectChange: (projectId: string) => void;
  onCreateNew: () => void;
}

export function ProjectSelector({ currentProject, onProjectChange, onCreateNew }: ProjectSelectorProps) {
  const [isOpen, setIsOpen] = useState(false);
  const [projects, setProjects] = useState<Project[]>([]);
  const [loading, setLoading] = useState(false);

  const loadProjects = async () => {
    setLoading(true);
    try {
      const projectList = await fetchProjects();
      setProjects(projectList);
    } catch (error) {
      console.error('Failed to load projects:', error);
      setProjects([]);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    if (isOpen) {
      loadProjects();
    }
  }, [isOpen]);

  return (
    <div style={{ position: 'fixed', top: '10px', left: '10px', zIndex: 1000 }}>
      <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
        <button
          onClick={() => setIsOpen(!isOpen)}
          style={{
            padding: '8px 12px',
            background: 'white',
            border: '1px solid #ccc',
            borderRadius: '4px',
            cursor: 'pointer',
            display: 'flex',
            alignItems: 'center',
            gap: '8px',
            fontWeight: '500',
          }}
        >
          <span>📁</span>
          <span>{currentProject?.name || 'No Project'}</span>
          <span style={{ fontSize: '10px' }}>{isOpen ? '▲' : '▼'}</span>
        </button>

        <button
          onClick={onCreateNew}
          style={{
            padding: '8px 12px',
            background: '#3b82f6',
            color: 'white',
            border: 'none',
            borderRadius: '4px',
            cursor: 'pointer',
            fontWeight: '500',
          }}
          title="Create New Project"
        >
          + New
        </button>
      </div>

      {isOpen && (
        <div
          style={{
            marginTop: '8px',
            background: 'white',
            border: '1px solid #ccc',
            borderRadius: '4px',
            boxShadow: '0 4px 12px rgba(0, 0, 0, 0.15)',
            minWidth: '250px',
            maxHeight: '400px',
            overflowY: 'auto',
          }}
        >
          {loading ? (
            <div style={{ padding: '12px', textAlign: 'center', color: '#666' }}>
              Loading projects...
            </div>
          ) : projects.length === 0 ? (
            <div style={{ padding: '12px', textAlign: 'center', color: '#666' }}>
              No projects found
            </div>
          ) : (
            <div>
              {projects.map((project) => (
                <div
                  key={project.id}
                  onClick={() => {
                    onProjectChange(project.id);
                    setIsOpen(false);
                  }}
                  style={{
                    padding: '12px',
                    cursor: 'pointer',
                    borderBottom: '1px solid #eee',
                    background: currentProject?.id === project.id ? '#f0f9ff' : 'white',
                    fontWeight: currentProject?.id === project.id ? '600' : 'normal',
                  }}
                  onMouseEnter={(e) => {
                    if (currentProject?.id !== project.id) {
                      e.currentTarget.style.background = '#f9fafb';
                    }
                  }}
                  onMouseLeave={(e) => {
                    if (currentProject?.id !== project.id) {
                      e.currentTarget.style.background = 'white';
                    }
                  }}
                >
                  <div style={{ fontWeight: 'inherit' }}>{project.name}</div>
                  <div style={{ fontSize: '11px', color: '#999', marginTop: '4px' }}>
                    {project.id}
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      )}
    </div>
  );
}
