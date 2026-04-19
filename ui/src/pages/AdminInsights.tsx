import { useState, useEffect } from 'react';
import { fetchApi } from '../lib/api';
import { Loader2, AlertCircle } from 'lucide-react';
import { Navigate } from 'react-router-dom';

interface ModerationLog {
  id: string;
  user_id: string;
  path: string;
  created_at: string;
  url?: string;
}

export default function AdminInsights() {
  const [logs, setLogs] = useState<ModerationLog[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchInsights = async () => {
      try {
        const res = await fetchApi('/admin/insights');
        const data = await res.json();
        setLogs(data);
      } catch (err: any) {
        setError(err.message || 'Failed to load insights');
      } finally {
        setLoading(false);
      }
    };
    fetchInsights();
  }, []);

  if (loading) {
    return (
      <div style={{ display: 'flex', justifyContent: 'center', marginTop: '4rem', color: 'var(--text-muted)' }}>
        <Loader2 className="animate-spin" size={32} />
        <style dangerouslySetInnerHTML={{__html: `
          @keyframes spin { 100% { transform: rotate(360deg); } }
          .animate-spin { animation: spin 1s linear infinite; }
        `}} />
      </div>
    );
  }

  if (error) {
    // If forbidden, probably not an admin.
    if (error.toLowerCase().includes('forbidden') || error.toLowerCase().includes('admin access required')) {
      return <Navigate to="/" replace />;
    }
    
    return (
      <div className="card" style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', color: '#ef4444' }}>
        <AlertCircle size={48} style={{ marginBottom: '1rem' }} />
        <p>{error}</p>
      </div>
    );
  }

  return (
    <div>
      <h2 style={{ marginTop: 0, marginBottom: '2rem' }}>Admin Insights</h2>
      
      <div className="card" style={{ marginBottom: '2rem' }}>
        <h3 style={{ margin: '0 0 1rem' }}>Recent Moderation Flags</h3>
        <p style={{ color: 'var(--text-muted)', marginBottom: '1.5rem' }}>Images that triggered the NSFW filter.</p>
        
        {logs.length === 0 ? (
          <p style={{ color: 'var(--text-muted)' }}>No recent moderation logs.</p>
        ) : (
          <div style={{ overflowX: 'auto' }}>
            <table style={{ width: '100%', borderCollapse: 'collapse', textAlign: 'left' }}>
              <thead>
                <tr style={{ borderBottom: '1px solid var(--border-color)' }}>
                  <th style={{ padding: '0.75rem', color: 'var(--text-muted)', fontWeight: 500 }}>Date</th>
                  <th style={{ padding: '0.75rem', color: 'var(--text-muted)', fontWeight: 500 }}>User ID</th>
                  <th style={{ padding: '0.75rem', color: 'var(--text-muted)', fontWeight: 500 }}>Image</th>
                </tr>
              </thead>
              <tbody>
                {logs.map((log) => (
                  <tr key={log.id} style={{ borderBottom: '1px solid var(--border-color)' }}>
                    <td style={{ padding: '0.75rem' }}>{new Date(log.created_at).toLocaleString()}</td>
                    <td style={{ padding: '0.75rem', fontFamily: 'monospace', fontSize: '0.875rem' }}>{log.user_id}</td>
                    <td style={{ padding: '0.75rem' }}>
                      {log.url ? (
                        <a href={log.url} target="_blank" rel="noreferrer" style={{ color: 'var(--primary-color)' }}>View Source</a>
                      ) : (
                        <span style={{ color: 'var(--text-muted)' }}>Expired</span>
                      )}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>
    </div>
  );
}
