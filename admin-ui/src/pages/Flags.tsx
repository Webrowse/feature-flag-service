import { useState, useEffect } from 'react';
import { useParams, useNavigate, Link } from 'react-router-dom';
import { getFlags, createFlag, toggleFlag, deleteFlag } from '../api/client';
import type { Flag } from '../api/client';

export function Flags() {
  const { projectId, environmentId } = useParams<{ projectId: string; environmentId: string }>();
  const [flags, setFlags] = useState<Flag[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');
  const [showCreate, setShowCreate] = useState(false);
  const [newKey, setNewKey] = useState('');
  const [newName, setNewName] = useState('');
  const navigate = useNavigate();

  const fetchFlags = async () => {
    if (!projectId || !environmentId) return;
    try {
      const response = await getFlags(projectId, environmentId);
      setFlags(response.data);
    } catch {
      setError('Failed to load flags');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchFlags();
  }, [projectId, environmentId]);

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!projectId || !environmentId) return;
    try {
      await createFlag(projectId, environmentId, newKey, newName);
      setNewKey('');
      setNewName('');
      setShowCreate(false);
      fetchFlags();
    } catch {
      setError('Failed to create flag');
    }
  };

  const handleToggle = async (flagId: string) => {
    if (!projectId || !environmentId) return;
    try {
      await toggleFlag(projectId, environmentId, flagId);
      fetchFlags();
    } catch {
      setError('Failed to toggle flag');
    }
  };

  const handleDelete = async (flagId: string) => {
    if (!confirm('Delete this flag?')) return;
    if (!projectId || !environmentId) return;
    try {
      await deleteFlag(projectId, environmentId, flagId);
      fetchFlags();
    } catch {
      setError('Failed to delete flag');
    }
  };

  if (loading) return <div style={styles.loading}>Loading...</div>;

  return (
    <div style={styles.container}>
      <nav style={styles.breadcrumb}>
        <Link to="/projects" style={styles.link}>Projects</Link>
        <span> / </span>
        <Link to={`/projects/${projectId}/environments`} style={styles.link}>Environments</Link>
        <span> / Flags</span>
      </nav>

      <header style={styles.header}>
        <h1 style={styles.title}>Feature Flags</h1>
        <button onClick={() => setShowCreate(true)} style={styles.button}>
          + New Flag
        </button>
      </header>

      {error && <div style={styles.error}>{error}</div>}

      {showCreate && (
        <div style={styles.modal}>
          <form onSubmit={handleCreate} style={styles.form}>
            <h3>Create Flag</h3>
            <input
              placeholder="Key (e.g., enable_dark_mode)"
              value={newKey}
              onChange={(e) => setNewKey(e.target.value.toLowerCase().replace(/[^a-z0-9_]/g, ''))}
              style={styles.input}
              required
            />
            <input
              placeholder="Name (e.g., Enable Dark Mode)"
              value={newName}
              onChange={(e) => setNewName(e.target.value)}
              style={styles.input}
              required
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

      <table style={styles.table}>
        <thead>
          <tr>
            <th style={styles.th}>Flag</th>
            <th style={styles.th}>Status</th>
            <th style={styles.th}>Rollout</th>
            <th style={styles.th}>Actions</th>
          </tr>
        </thead>
        <tbody>
          {flags.length === 0 ? (
            <tr>
              <td colSpan={4} style={styles.empty}>No flags yet. Create your first flag!</td>
            </tr>
          ) : (
            flags.map((flag) => (
              <tr key={flag.id}>
                <td style={styles.td}>
                  <strong>{flag.name}</strong>
                  <br />
                  <code style={styles.code}>{flag.key}</code>
                </td>
                <td style={styles.td}>
                  <button
                    onClick={() => handleToggle(flag.id)}
                    style={{
                      ...styles.toggleButton,
                      backgroundColor: flag.enabled ? '#28a745' : '#dc3545',
                    }}
                  >
                    {flag.enabled ? 'ON' : 'OFF'}
                  </button>
                </td>
                <td style={styles.td}>
                  {flag.rollout_percentage}%
                </td>
                <td style={styles.td}>
                  <button
                    onClick={() => navigate(`/projects/${projectId}/environments/${environmentId}/flags/${flag.id}`)}
                    style={styles.button}
                  >
                    Edit
                  </button>
                  <button
                    onClick={() => handleDelete(flag.id)}
                    style={{ ...styles.deleteButton, marginLeft: '0.5rem' }}
                  >
                    Delete
                  </button>
                </td>
              </tr>
            ))
          )}
        </tbody>
      </table>
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
  table: {
    width: '100%',
    backgroundColor: 'white',
    borderRadius: '8px',
    boxShadow: '0 2px 4px rgba(0,0,0,0.1)',
    borderCollapse: 'collapse',
  },
  th: {
    textAlign: 'left',
    padding: '1rem',
    borderBottom: '2px solid #eee',
  },
  td: {
    padding: '1rem',
    borderBottom: '1px solid #eee',
  },
  code: {
    backgroundColor: '#f5f5f5',
    padding: '0.125rem 0.375rem',
    borderRadius: '4px',
    fontSize: '0.75rem',
  },
  toggleButton: {
    padding: '0.375rem 0.75rem',
    color: 'white',
    border: 'none',
    borderRadius: '4px',
    cursor: 'pointer',
    fontWeight: 'bold',
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
    padding: '2rem',
  },
};
