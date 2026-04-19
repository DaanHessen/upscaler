import { useState, useEffect } from 'react';
import { fetchApi } from '../lib/api';
import { Loader2, AlertCircle } from 'lucide-react';
import { useSearchParams } from 'react-router-dom';

export default function Settings() {
  const [balance, setBalance] = useState<number | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [checkoutLoading, setCheckoutLoading] = useState<string | null>(null);
  const [searchParams] = useSearchParams();
  const paymentStatus = searchParams.get('payment');

  useEffect(() => {
    const fetchBalance = async () => {
      try {
        const res = await fetchApi('/balance');
        const data = await res.json();
        setBalance(data.credits);
      } catch (err: any) {
        setError(err.message || 'Failed to load balance');
      } finally {
        setLoading(false);
      }
    };
    fetchBalance();
  }, []);

  const handleCheckout = async (tier: string) => {
    setCheckoutLoading(tier);
    setError(null);
    try {
      const res = await fetchApi('/checkout', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ tier }),
      });
      const data = await res.json();
      if (data.url) {
        window.location.href = data.url;
      } else {
        throw new Error('No checkout URL returned');
      }
    } catch (err: any) {
      setError(err.message || 'Failed to initiate checkout');
      setCheckoutLoading(null);
    }
  };

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

  return (
    <div style={{ maxWidth: '800px', margin: '0 auto' }}>
      <h2 style={{ marginTop: 0, marginBottom: '2rem' }}>Settings & Credits</h2>
      
      {paymentStatus === 'success' && (
        <div style={{ padding: '1rem', backgroundColor: 'rgba(34, 197, 94, 0.1)', color: '#22c55e', borderRadius: '6px', marginBottom: '2rem' }}>
          Payment successful! Your credits have been added.
        </div>
      )}
      
      {paymentStatus === 'cancelled' && (
        <div style={{ padding: '1rem', backgroundColor: 'rgba(239, 68, 68, 0.1)', color: '#ef4444', borderRadius: '6px', marginBottom: '2rem' }}>
          Payment was cancelled.
        </div>
      )}

      {error && (
        <div style={{ padding: '1rem', backgroundColor: 'rgba(239, 68, 68, 0.1)', color: '#ef4444', borderRadius: '6px', marginBottom: '2rem', display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
          <AlertCircle size={20} />
          {error}
        </div>
      )}

      <div className="card" style={{ marginBottom: '2rem', display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <div>
          <h3 style={{ margin: '0 0 0.5rem' }}>Current Balance</h3>
          <p style={{ margin: 0, color: 'var(--text-muted)' }}>Use credits to upscale images.</p>
        </div>
        <div style={{ fontSize: '2rem', fontWeight: 700, color: 'var(--primary-color)' }}>
          {balance !== null ? balance : '-'} <span style={{ fontSize: '1rem', fontWeight: 400, color: 'var(--text-muted)' }}>credits</span>
        </div>
      </div>

      <h3 style={{ marginBottom: '1rem' }}>Buy Credits</h3>
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(240px, 1fr))', gap: '1.5rem' }}>
        
        <div className="card" style={{ display: 'flex', flexDirection: 'column' }}>
          <h4 style={{ margin: '0 0 0.5rem', fontSize: '1.25rem' }}>Starter</h4>
          <div style={{ fontSize: '2rem', fontWeight: 700, marginBottom: '1rem' }}>$5</div>
          <ul style={{ margin: '0 0 1.5rem', paddingLeft: '1.5rem', color: 'var(--text-muted)', flex: 1 }}>
            <li>50 Credits</li>
            <li>~25 2K upscales</li>
          </ul>
          <button 
            className="btn" 
            style={{ width: '100%' }}
            onClick={() => handleCheckout('starter')}
            disabled={!!checkoutLoading}
          >
            {checkoutLoading === 'starter' ? <Loader2 size={18} className="animate-spin" /> : 'Purchase'}
          </button>
        </div>

        <div className="card" style={{ display: 'flex', flexDirection: 'column', border: '2px solid var(--primary-color)' }}>
          <h4 style={{ margin: '0 0 0.5rem', fontSize: '1.25rem', color: 'var(--primary-color)' }}>Pro</h4>
          <div style={{ fontSize: '2rem', fontWeight: 700, marginBottom: '1rem' }}>$15</div>
          <ul style={{ margin: '0 0 1.5rem', paddingLeft: '1.5rem', color: 'var(--text-muted)', flex: 1 }}>
            <li>200 Credits</li>
            <li>~100 2K upscales</li>
            <li>Better value</li>
          </ul>
          <button 
            className="btn" 
            style={{ width: '100%' }}
            onClick={() => handleCheckout('pro')}
            disabled={!!checkoutLoading}
          >
            {checkoutLoading === 'pro' ? <Loader2 size={18} className="animate-spin" /> : 'Purchase'}
          </button>
        </div>

      </div>
    </div>
  );
}
