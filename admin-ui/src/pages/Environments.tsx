import { useState, useEffect } from 'react';
import { useParams, useNavigate, Link } from 'react-router-dom';
import { getEnvironments, createEnvironment } from '../api/client';
import type { Environment } from '../api/client';

export function Environments() {
  const { projectId } = useParams<{ projectId: string }>();
  const [environments, setEnvironments] = useState<Environment[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');
  const [showCreate, setShowCreate] = useState(false);
  const [newName, setNewName] = useState('');
  const [newKey, setNewKey] = useState('');
  const navigate = useNavigate();

  const fetchEnvironments = async () => {
    if (!projectId) return;
    try {
      const response = await getEnvironments(projectId);
      setEnvironments(response.data);
    } catch {
      setError('Failed to load environments');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchEnvironments();
  }, [projectId]);

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!projectId) return;
    try {
      await createEnvironment(projectId, newName, newKey);
      setNewName('');
      setNewKey('');
      setShowCreate(false);
      fetchEnvironments();
    } catch {
      setError('Failed to create environment');
    }
  };

  if (loading) return <div style={styles.loading}>Loading...</div>;

  return (
    <div style={styles.container}>
      <nav style={styles.breadcrumb}>
        <Link to="/projects" style={styles.link}>Projects</Link>
        <span> / Environments</span>
      </nav>

      <header style={styles.header}>
        <h1 style={styles.title}>Environments</h1>
        <button onClick={() => setShowCreate(true)} style={styles.button}>
          + New Environment
        </button>
      </header>

      {error && <div style={styles.error}>{error}</div>}

      {showCreate && (
        <div style={styles.modal}>
          <form onSubmit={handleCreate} style={styles.form}>
            <h3>Create Environment</h3>
            <input
              placeholder="Name (e.g., Development)"
              value={newName}
              onChange={(e) => setNewName(e.target.value)}
              style={styles.input}
              required
            />
            <input
              placeholder="Key (e.g., development)"
              value={newKey}
              onChange={(e) => setNewKey(e.target.value.toLowerCase().replace(/[^a-z0-9_-]/g, ''))}
              style={styles.input}
              required
              pattern="[a-z][a-z0-9_-]*"
              title="Lowercase letters, numbers, underscores, hyphens. Must start with a letter."
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
        {environments.length === 0 ? (
          <p style={styles.empty}>No environments yet.</p>
        ) : (
          environments.map((env) => (
            <div key={env.id} style={styles.card}>
              <h3 style={styles.cardTitle}>{env.name}</h3>
              <code style={styles.code}>{env.key}</code>
              <p style={styles.cardDescription}>{env.description || 'No description'}</p>
              <button
                onClick={() => navigate(`/projects/${projectId}/environments/${env.id}/flags`)}
                style={styles.button}
              >
                Manage Flags
              </button>
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
  breadcrumb: {
    marginBottom: '1rem',
    color: '#666',
  },
  link: {
    color: '#007bff',
    textDecoration: 'none',
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
    gridTemplateColumns: 'repeat(auto-fill, minmax(280px, 1fr))',
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
  code: {
    display: 'inline-block',
    backgroundColor: '#f5f5f5',
    padding: '0.25rem 0.5rem',
    borderRadius: '4px',
    fontSize: '0.875rem',
    marginBottom: '0.5rem',
  },
  cardDescription: {
    color: '#666',
    margin: '0 0 1rem 0',
  },
  button: {
    padding: '0.5rem 1rem',
    backgroundColor: '#007bff',
    color: 'white',
    border: 'none',
    borderRadius: '4px',
    cursor: 'pointer',
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
