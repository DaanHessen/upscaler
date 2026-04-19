import { useState, useEffect } from 'react';
import { fetchApi } from '../lib/api';
import { Loader2, AlertCircle } from 'lucide-react';

interface HistoryItem {
  id: string;
  status: string;
  created_at: string;
  quality: string;
  style?: string;
  temperature: number;
  image_url?: string;
  error?: string;
}

export default function History() {
  const [items, setItems] = useState<HistoryItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchHistory = async () => {
      try {
        const res = await fetchApi('/history');
        const data = await res.json();
        setItems(data);
      } catch (err: any) {
        setError(err.message || 'Failed to load history');
      } finally {
        setLoading(false);
      }
    };
    fetchHistory();
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
    return (
      <div className="card" style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', color: '#ef4444' }}>
        <AlertCircle size={48} style={{ marginBottom: '1rem' }} />
        <p>{error}</p>
      </div>
    );
  }

  if (items.length === 0) {
    return (
      <div className="card" style={{ textAlign: 'center', color: 'var(--text-muted)', padding: '4rem 2rem' }}>
        <p>No upscale history found.</p>
      </div>
    );
  }

  return (
    <div>
      <h2 style={{ marginTop: 0, marginBottom: '2rem' }}>Upscale History</h2>
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(280px, 1fr))', gap: '1.5rem' }}>
        {items.map((item) => (
          <div key={item.id} className="card" style={{ padding: '1rem', display: 'flex', flexDirection: 'column' }}>
            <div style={{ flex: 1, backgroundColor: 'var(--bg-color)', borderRadius: '6px', overflow: 'hidden', display: 'flex', alignItems: 'center', justifyContent: 'center', minHeight: '200px' }}>
              {item.image_url ? (
                <a href={item.image_url} target="_blank" rel="noreferrer" style={{ display: 'block', width: '100%', height: '100%' }}>
                  <img src={item.image_url} alt="Upscaled result" style={{ width: '100%', height: '100%', objectFit: 'cover' }} />
                </a>
              ) : (
                <div style={{ padding: '1rem', textAlign: 'center', color: item.status === 'failed' ? '#ef4444' : 'var(--text-muted)' }}>
                  {item.status === 'failed' ? (
                    <>
                      <AlertCircle size={24} style={{ margin: '0 auto 0.5rem' }} />
                      <span style={{ fontSize: '0.875rem' }}>{item.error || 'Failed'}</span>
                    </>
                  ) : (
                    <span>{item.status}</span>
                  )}
                </div>
              )}
            </div>
            <div style={{ marginTop: '1rem', fontSize: '0.875rem', color: 'var(--text-muted)' }}>
              <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '0.25rem' }}>
                <span>Quality: <strong>{item.quality}</strong></span>
                <span>{new Date(item.created_at).toLocaleDateString()}</span>
              </div>
              <div style={{ display: 'flex', justifyContent: 'space-between' }}>
                <span>Style: <strong>{item.style || 'AUTO'}</strong></span>
                <span>Temp: <strong>{item.temperature.toFixed(1)}</strong></span>
              </div>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
