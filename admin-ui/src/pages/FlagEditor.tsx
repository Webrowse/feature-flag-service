import { useState, useEffect } from 'react';
import { useParams, Link } from 'react-router-dom';
import {
  getFlags,
  updateFlag,
  toggleFlag,
  getRules,
  createRule,
  deleteRule,
} from '../api/client';
import type { Flag, Rule } from '../api/client';

export function FlagEditor() {
  const { projectId, environmentId, flagId } = useParams<{
    projectId: string;
    environmentId: string;
    flagId: string;
  }>();
  const [flag, setFlag] = useState<Flag | null>(null);
  const [rules, setRules] = useState<Rule[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');
  const [rollout, setRollout] = useState(0);
  const [showAddRule, setShowAddRule] = useState(false);
  const [newRuleType, setNewRuleType] = useState('user_id');
  const [newRuleValue, setNewRuleValue] = useState('');

  const fetchData = async () => {
    if (!projectId || !environmentId || !flagId) return;
    try {
      const [flagsRes, rulesRes] = await Promise.all([
        getFlags(projectId, environmentId),
        getRules(projectId, environmentId, flagId),
      ]);
      const foundFlag = flagsRes.data.find((f) => f.id === flagId);
      if (foundFlag) {
        setFlag(foundFlag);
        setRollout(foundFlag.rollout_percentage);
      }
      setRules(rulesRes.data);
    } catch {
      setError('Failed to load flag data');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchData();
  }, [projectId, environmentId, flagId]);

  const handleToggle = async () => {
    if (!projectId || !environmentId || !flagId) return;
    try {
      await toggleFlag(projectId, environmentId, flagId);
      fetchData();
    } catch {
      setError('Failed to toggle flag');
    }
  };

  const handleRolloutChange = async () => {
    if (!projectId || !environmentId || !flagId || !flag) return;
    if (rollout === flag.rollout_percentage) return;
    try {
      await updateFlag(projectId, environmentId, flagId, { rollout_percentage: rollout });
      fetchData();
    } catch {
      setError('Failed to update rollout');
    }
  };

  const handleAddRule = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!projectId || !environmentId || !flagId) return;
    try {
      await createRule(projectId, environmentId, flagId, {
        rule_type: newRuleType,
        rule_value: newRuleValue,
      });
      setNewRuleValue('');
      setShowAddRule(false);
      fetchData();
    } catch {
      setError('Failed to add rule');
    }
  };

  const handleDeleteRule = async (ruleId: string) => {
    if (!confirm('Delete this rule?')) return;
    if (!projectId || !environmentId || !flagId) return;
    try {
      await deleteRule(projectId, environmentId, flagId, ruleId);
      fetchData();
    } catch {
      setError('Failed to delete rule');
    }
  };

  if (loading) return <div style={styles.loading}>Loading...</div>;
  if (!flag) return <div style={styles.error}>Flag not found</div>;

  return (
    <div style={styles.container}>
      <nav style={styles.breadcrumb}>
        <Link to="/projects" style={styles.link}>Projects</Link>
        <span> / </span>
        <Link to={`/projects/${projectId}/environments`} style={styles.link}>Environments</Link>
        <span> / </span>
        <Link to={`/projects/${projectId}/environments/${environmentId}/flags`} style={styles.link}>Flags</Link>
        <span> / {flag.name}</span>
      </nav>

      {error && <div style={styles.error}>{error}</div>}

      <div style={styles.card}>
        <h1 style={styles.title}>{flag.name}</h1>
        <code style={styles.code}>{flag.key}</code>

        <div style={styles.section}>
          <h3>Status</h3>
          <button
            onClick={handleToggle}
            style={{
              ...styles.toggleButton,
              backgroundColor: flag.enabled ? '#28a745' : '#dc3545',
            }}
          >
            {flag.enabled ? 'ENABLED' : 'DISABLED'}
          </button>
        </div>

        <div style={styles.section}>
          <h3>Rollout Percentage</h3>
          <div style={styles.rolloutRow}>
            <input
              type="range"
              min="0"
              max="100"
              value={rollout}
              onChange={(e) => setRollout(Number(e.target.value))}
              style={styles.slider}
            />
            <span style={styles.rolloutValue}>{rollout}%</span>
            <button
              onClick={handleRolloutChange}
              style={styles.button}
              disabled={rollout === flag.rollout_percentage}
            >
              Save
            </button>
          </div>
          <p style={styles.hint}>
            When enabled, only this percentage of users will see the feature (based on consistent hashing).
          </p>
        </div>
      </div>

      <div style={styles.card}>
        <div style={styles.sectionHeader}>
          <h2>Targeting Rules</h2>
          <button onClick={() => setShowAddRule(true)} style={styles.button}>
            + Add Rule
          </button>
        </div>

        {showAddRule && (
          <form onSubmit={handleAddRule} style={styles.ruleForm}>
            <select
              value={newRuleType}
              onChange={(e) => setNewRuleType(e.target.value)}
              style={styles.select}
            >
              <option value="user_id">User ID</option>
              <option value="user_email">User Email</option>
              <option value="email_domain">Email Domain</option>
            </select>
            <input
              placeholder={
                newRuleType === 'email_domain'
                  ? '@company.com'
                  : newRuleType === 'user_email'
                  ? 'user@example.com'
                  : 'user-123'
              }
              value={newRuleValue}
              onChange={(e) => setNewRuleValue(e.target.value)}
              style={styles.input}
              required
            />
            <button type="submit" style={styles.button}>Add</button>
            <button type="button" onClick={() => setShowAddRule(false)} style={styles.cancelButton}>
              Cancel
            </button>
          </form>
        )}

        <table style={styles.table}>
          <thead>
            <tr>
              <th style={styles.th}>Type</th>
              <th style={styles.th}>Value</th>
              <th style={styles.th}>Priority</th>
              <th style={styles.th}>Actions</th>
            </tr>
          </thead>
          <tbody>
            {rules.length === 0 ? (
              <tr>
                <td colSpan={4} style={styles.emptyCell}>
                  No targeting rules. Flag applies to all users based on rollout percentage.
                </td>
              </tr>
            ) : (
              rules.map((rule) => (
                <tr key={rule.id}>
                  <td style={styles.td}>
                    <code style={styles.ruleCode}>{rule.rule_type}</code>
                  </td>
                  <td style={styles.td}>{rule.rule_value}</td>
                  <td style={styles.td}>{rule.priority}</td>
                  <td style={styles.td}>
                    <button
                      onClick={() => handleDeleteRule(rule.id)}
                      style={styles.deleteButton}
                    >
                      Delete
                    </button>
                  </td>
                </tr>
              ))
            )}
          </tbody>
        </table>

        <p style={styles.hint}>
          Rules are evaluated in priority order (highest first). If a rule matches, the flag is enabled for that user.
        </p>
      </div>
    </div>
  );
}

const styles: Record<string, React.CSSProperties> = {
  container: {
    padding: '2rem',
    maxWidth: '900px',
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
  card: {
    backgroundColor: 'white',
    padding: '1.5rem',
    borderRadius: '8px',
    boxShadow: '0 2px 4px rgba(0,0,0,0.1)',
    marginBottom: '1.5rem',
  },
  title: {
    margin: '0 0 0.5rem 0',
  },
  code: {
    display: 'inline-block',
    backgroundColor: '#f5f5f5',
    padding: '0.25rem 0.5rem',
    borderRadius: '4px',
    fontSize: '0.875rem',
  },
  section: {
    marginTop: '1.5rem',
  },
  sectionHeader: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: '1rem',
  },
  toggleButton: {
    padding: '0.5rem 1.5rem',
    color: 'white',
    border: 'none',
    borderRadius: '4px',
    cursor: 'pointer',
    fontWeight: 'bold',
    fontSize: '1rem',
  },
  rolloutRow: {
    display: 'flex',
    alignItems: 'center',
    gap: '1rem',
  },
  slider: {
    flex: 1,
    height: '8px',
  },
  rolloutValue: {
    fontWeight: 'bold',
    minWidth: '50px',
  },
  hint: {
    color: '#666',
    fontSize: '0.875rem',
    marginTop: '0.5rem',
  },
  table: {
    width: '100%',
    borderCollapse: 'collapse',
  },
  th: {
    textAlign: 'left',
    padding: '0.75rem',
    borderBottom: '2px solid #eee',
  },
  td: {
    padding: '0.75rem',
    borderBottom: '1px solid #eee',
  },
  ruleCode: {
    backgroundColor: '#e9ecef',
    padding: '0.125rem 0.375rem',
    borderRadius: '4px',
    fontSize: '0.75rem',
  },
  emptyCell: {
    padding: '1.5rem',
    textAlign: 'center',
    color: '#666',
  },
  ruleForm: {
    display: 'flex',
    gap: '0.5rem',
    marginBottom: '1rem',
    flexWrap: 'wrap',
  },
  select: {
    padding: '0.5rem',
    fontSize: '1rem',
    border: '1px solid #ddd',
    borderRadius: '4px',
  },
  input: {
    padding: '0.5rem',
    fontSize: '1rem',
    border: '1px solid #ddd',
    borderRadius: '4px',
    flex: 1,
    minWidth: '200px',
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
    padding: '0.375rem 0.75rem',
    backgroundColor: '#dc3545',
    color: 'white',
    border: 'none',
    borderRadius: '4px',
    cursor: 'pointer',
    fontSize: '0.875rem',
  },
  cancelButton: {
    padding: '0.5rem 1rem',
    backgroundColor: '#6c757d',
    color: 'white',
    border: 'none',
    borderRadius: '4px',
    cursor: 'pointer',
  },
};
