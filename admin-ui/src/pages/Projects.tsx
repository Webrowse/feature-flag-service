import { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { getProjects, createProject, deleteProject } from '../api/client';
import type { Project } from '../api/client';
import { useAuth } from '../hooks/useAuth';

export function Projects() {
  const [projects, setProjects] = useState<Project[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');
  const [showCreate, setShowCreate] = useState(false);
  const [newName, setNewName] = useState('');
  const [newDescription, setNewDescription] = useState('');
  const navigate = useNavigate();
  const { logout } = useAuth();

  const fetchProjects = async () => {
    try {
      const response = await getProjects();
      setProjects(response.data);
    } catch {
      setError('Failed to load projects');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchProjects();
  }, []);

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault();
    try {
      await createProject(newName, newDescription || undefined);
      setNewName('');
      setNewDescription('');
      setShowCreate(false);
      fetchProjects();
    } catch {
      setError('Failed to create project');
    }
  };

  const handleDelete = async (id: string) => {
    if (!confirm('Delete this project? All environments and flags will be deleted.')) return;
    try {
      await deleteProject(id);
      fetchProjects();
    } catch {
      setError('Failed to delete project');
    }
  };

  if (loading) return <div style={styles.loading}>Loading...</div>;

  return (
    <div style={styles.container}>
      <header style={styles.header}>
        <h1 style={styles.title}>Projects</h1>
        <div>
          <button onClick={() => setShowCreate(true)} style={styles.button}>
            + New Project
          </button>
          <button onClick={logout} style={styles.logoutButton}>
            Logout
          </button>
        </div>
      </header>

      {error && <div style={styles.error}>{error}</div>}

      {showCreate && (
        <div style={styles.modal}>
          <form onSubmit={handleCreate} style={styles.form}>
            <h3>Create Project</h3>
            <input
              placeholder="Project name"
              value={newName}
              onChange={(e) => setNewName(e.target.value)}
              style={styles.input}
              required
            />
            <input
              placeholder="Description (optional)"
              value={newDescription}
              onChange={(e) => setNewDescription(e.target.value)}
              style={styles.input}
            />
            <div style={styles.formActions}>
              <button type="button" onClick={() => setShowCreate(false)} style={styles.cancelButton}>
                Cancel
              </button>
              <button type="submit" style={styles.button}>
                Create
              </button>
            </div>
          </form>
        </div>
      )}

      <div style={styles.grid}>
        {projects.length === 0 ? (
          <p style={styles.empty}>No projects yet. Create your first project!</p>
        ) : (
          projects.map((project) => (
            <div key={project.id} style={styles.card}>
              <h3 style={styles.cardTitle}>{project.name}</h3>
              <p style={styles.cardDescription}>{project.description || 'No description'}</p>
              <div style={styles.sdkKey}>
                <small>SDK Key:</small>
                <code style={styles.code}>{project.sdk_key}</code>
              </div>
              <div style={styles.cardActions}>
                <button
                  onClick={() => navigate(`/projects/${project.id}/environments`)}
                  style={styles.button}
                >
                  Environments
                </button>
                <button
                  onClick={() => handleDelete(project.id)}
                  style={styles.deleteButton}
                >
                  Delete
                </button>
              </div>
            </div>
          ))
        )}
      </div>
    </div>
  );
}

const styles: Record<string, React.CSSProperties> = {
  container: {
    padding: '2rem',
    maxWidth: '1200px',
    margin: '0 auto',
  },
  header: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: '2rem',
  },
  title: {
    margin: 0,
  },
  loading: {
    padding: '2rem',
    textAlign: 'center',
  },
  error: {
    backgroundColor: '#fee',
    color: '#c00',
    padding: '0.75rem',
    borderRadius: '4px',
    marginBottom: '1rem',
  },
  grid: {
    display: 'grid',
    gridTemplateColumns: 'repeat(auto-fill, minmax(300px, 1fr))',
    gap: '1rem',
  },
  card: {
    backgroundColor: 'white',
    padding: '1.5rem',
    borderRadius: '8px',
    boxShadow: '0 2px 4px rgba(0,0,0,0.1)',
  },
  cardTitle: {
    margin: '0 0 0.5rem 0',
  },
  cardDescription: {
    color: '#666',
    margin: '0 0 1rem 0',
  },
  sdkKey: {
    marginBottom: '1rem',
  },
  code: {
    display: 'block',
    backgroundColor: '#f5f5f5',
    padding: '0.5rem',
    borderRadius: '4px',
    fontSize: '0.75rem',
    wordBreak: 'break-all',
    marginTop: '0.25rem',
  },
  cardActions: {
    display: 'flex',
    gap: '0.5rem',
  },
  button: {
    padding: '0.5rem 1rem',
    backgroundColor: '#007bff',
    color: 'white',
    border: 'none',
    borderRadius: '4px',
    cursor: 'pointer',
  },
  deleteButton: {
    padding: '0.5rem 1rem',
    backgroundColor: '#dc3545',
    color: 'white',
    border: 'none',
    borderRadius: '4px',
    cursor: 'pointer',
  },
  logoutButton: {
    padding: '0.5rem 1rem',
    backgroundColor: '#6c757d',
    color: 'white',
    border: 'none',
    borderRadius: '4px',
    cursor: 'pointer',
    marginLeft: '0.5rem',
  },
  cancelButton: {
    padding: '0.5rem 1rem',
    backgroundColor: '#6c757d',
    color: 'white',
    border: 'none',
    borderRadius: '4px',
    cursor: 'pointer',
  },
  modal: {
    backgroundColor: 'white',
    padding: '1.5rem',
    borderRadius: '8px',
    boxShadow: '0 2px 4px rgba(0,0,0,0.1)',
    marginBottom: '2rem',
  },
  form: {
    display: 'flex',
    flexDirection: 'column',
    gap: '1rem',
  },
  input: {
    padding: '0.75rem',
    fontSize: '1rem',
    border: '1px solid #ddd',
    borderRadius: '4px',
  },
  formActions: {
    display: 'flex',
    justifyContent: 'flex-end',
    gap: '0.5rem',
  },
  empty: {
    color: '#666',
    textAlign: 'center',
    gridColumn: '1 / -1',
  },
};
