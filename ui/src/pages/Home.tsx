import { useState, useRef, useEffect } from 'react';
import { useAuth } from '../components/AuthContext';
import { Navigate } from 'react-router-dom';
import { fetchApi } from '../lib/api';
import { UploadCloud, Image as ImageIcon, Loader2, AlertCircle } from 'lucide-react';

type JobStatus = 'idle' | 'uploading' | 'pending' | 'processing' | 'completed' | 'failed';

interface PollResponse {
  success: boolean;
  status: string;
  image_url?: string;
  error?: string;
  style: string;
}

export default function Home() {
  const { user, isLoading } = useAuth();
  
  const [file, setFile] = useState<File | null>(null);
  const [previewUrl, setPreviewUrl] = useState<string | null>(null);
  
  const [quality, setQuality] = useState('2K');
  const [style, setStyle] = useState('AUTO');
  const [temperature, setTemperature] = useState(0.0);
  
  const [jobStatus, setJobStatus] = useState<JobStatus>('idle');
  const [jobId, setJobId] = useState<string | null>(null);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [resultUrl, setResultUrl] = useState<string | null>(null);
  const [resultStyle, setResultStyle] = useState<string | null>(null);

  const fileInputRef = useRef<HTMLInputElement>(null);

  // Poll for status
  useEffect(() => {
    let intervalId: number | undefined;

    const checkStatus = async () => {
      if (!jobId) return;
      try {
        const res = await fetchApi(`/upscales/${jobId}`);
        const data: PollResponse = await res.json();

        if (!data.success) {
          setJobStatus('failed');
          setErrorMessage(data.error || 'An error occurred during processing.');
          return;
        }

        if (data.status === 'completed') {
          setJobStatus('completed');
          setResultUrl(data.image_url || null);
          setResultStyle(data.style);
        } else if (data.status === 'failed') {
          setJobStatus('failed');
          setErrorMessage(data.error || 'Job failed.');
        } else {
          setJobStatus(data.status as JobStatus); // pending or processing
        }
      } catch (err: any) {
        setJobStatus('failed');
        setErrorMessage(err.message || 'Failed to check status');
      }
    };

    if (jobId && ['pending', 'processing'].includes(jobStatus)) {
      intervalId = window.setInterval(checkStatus, 2000);
    }

    return () => {
      if (intervalId) clearInterval(intervalId);
    };
  }, [jobId, jobStatus]);

  if (isLoading) {
    return <div style={{ textAlign: 'center', marginTop: '4rem', color: 'var(--text-muted)' }}>Loading...</div>;
  }

  if (!user) {
    return <Navigate to="/login" replace />;
  }

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    if (e.target.files && e.target.files.length > 0) {
      const selectedFile = e.target.files[0];
      setFile(selectedFile);
      setPreviewUrl(URL.createObjectURL(selectedFile));
      // Reset state on new file
      setJobStatus('idle');
      setJobId(null);
      setResultUrl(null);
      setErrorMessage(null);
    }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!file) return;

    setJobStatus('uploading');
    setErrorMessage(null);

    const formData = new FormData();
    formData.append('image', file);
    formData.append('quality', quality);
    formData.append('style', style);
    formData.append('temperature', temperature.toString());

    try {
      const res = await fetchApi('/upscale', {
        method: 'POST',
        body: formData,
        // Let the browser set the boundary for multipart/form-data
        // Do not set Content-Type header explicitly for FormData in fetch
      });
      const data = await res.json();
      
      if (data.success) {
        setJobId(data.job_id);
        setJobStatus('pending');
      } else {
        setJobStatus('failed');
        setErrorMessage(data.error || 'Failed to submit job.');
      }
    } catch (err: any) {
      setJobStatus('failed');
      setErrorMessage(err.message || 'An error occurred during submission.');
    }
  };

  const handleReset = () => {
    setFile(null);
    setPreviewUrl(null);
    setJobStatus('idle');
    setJobId(null);
    setResultUrl(null);
    setErrorMessage(null);
    if (fileInputRef.current) {
      fileInputRef.current.value = '';
    }
  };

  return (
    <div style={{ display: 'grid', gap: '2rem', gridTemplateColumns: '1fr 1fr', alignItems: 'start' }}>
      {/* Upload & Config Column */}
      <div className="card">
        <h2 style={{ marginTop: 0 }}>Upscale Image</h2>
        <form onSubmit={handleSubmit} style={{ display: 'flex', flexDirection: 'column', gap: '1.5rem' }}>
          
          {/* File Input */}
          <div 
            style={{ 
              border: '2px dashed var(--border-color)', 
              borderRadius: '8px', 
              padding: '2rem', 
              textAlign: 'center',
              cursor: jobStatus === 'idle' || jobStatus === 'failed' ? 'pointer' : 'default',
              backgroundColor: 'var(--bg-color)',
              position: 'relative',
              overflow: 'hidden'
            }}
            onClick={() => {
              if ((jobStatus === 'idle' || jobStatus === 'failed') && fileInputRef.current) {
                fileInputRef.current.click();
              }
            }}
          >
            <input 
              type="file" 
              accept="image/*" 
              onChange={handleFileChange} 
              ref={fileInputRef} 
              style={{ display: 'none' }} 
              disabled={jobStatus !== 'idle' && jobStatus !== 'failed'}
            />
            {previewUrl ? (
              <img 
                src={previewUrl} 
                alt="Preview" 
                style={{ 
                  maxWidth: '100%', 
                  maxHeight: '300px', 
                  objectFit: 'contain', 
                  borderRadius: '4px' 
                }} 
              />
            ) : (
              <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', gap: '0.5rem', color: 'var(--text-muted)' }}>
                <UploadCloud size={48} strokeWidth={1.5} />
                <p style={{ margin: 0 }}>Click to select an image</p>
              </div>
            )}
          </div>

          {/* Settings */}
          <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '1rem' }}>
            <div>
              <label style={{ display: 'block', marginBottom: '0.5rem', fontSize: '0.875rem' }}>Quality</label>
              <select 
                value={quality} 
                onChange={(e) => setQuality(e.target.value)}
                disabled={jobStatus !== 'idle' && jobStatus !== 'failed'}
                style={{ width: '100%', padding: '0.75rem', borderRadius: '6px', border: '1px solid var(--border-color)', backgroundColor: 'var(--bg-color)', color: 'var(--text-color)' }}
              >
                <option value="2K">2K (2 credits)</option>
                <option value="4K">4K (5 credits)</option>
              </select>
            </div>
            
            <div>
              <label style={{ display: 'block', marginBottom: '0.5rem', fontSize: '0.875rem' }}>Style Override</label>
              <select 
                value={style} 
                onChange={(e) => setStyle(e.target.value)}
                disabled={jobStatus !== 'idle' && jobStatus !== 'failed'}
                style={{ width: '100%', padding: '0.75rem', borderRadius: '6px', border: '1px solid var(--border-color)', backgroundColor: 'var(--bg-color)', color: 'var(--text-color)' }}
              >
                <option value="AUTO">Auto-detect</option>
                <option value="ILLUSTRATION">Illustration</option>
                <option value="PHOTOGRAPHY">Photography</option>
              </select>
            </div>
          </div>

          <div>
            <label style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '0.5rem', fontSize: '0.875rem' }}>
              <span>Temperature</span>
              <span>{temperature.toFixed(1)}</span>
            </label>
            <input 
              type="range" 
              min="0" 
              max="2" 
              step="0.1" 
              value={temperature} 
              onChange={(e) => setTemperature(parseFloat(e.target.value))}
              disabled={jobStatus !== 'idle' && jobStatus !== 'failed'}
              style={{ width: '100%' }}
            />
            <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: '0.75rem', color: 'var(--text-muted)', marginTop: '0.25rem' }}>
              <span>Faithful</span>
              <span>Creative</span>
            </div>
          </div>

          <button 
            type="submit" 
            className="btn" 
            disabled={!file || (jobStatus !== 'idle' && jobStatus !== 'failed')}
            style={{ padding: '1rem', fontSize: '1rem' }}
          >
            {jobStatus === 'idle' || jobStatus === 'failed' ? 'Start Upscale' : 'Processing...'}
          </button>
        </form>
      </div>

      {/* Result / Status Column */}
      <div className="card" style={{ display: 'flex', flexDirection: 'column', minHeight: '400px' }}>
        <h2 style={{ marginTop: 0 }}>Result</h2>
        
        <div style={{ flex: 1, display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center', backgroundColor: 'var(--bg-color)', borderRadius: '8px', border: '1px solid var(--border-color)', padding: '1rem' }}>
          
          {jobStatus === 'idle' && (
             <div style={{ color: 'var(--text-muted)', textAlign: 'center' }}>
               <ImageIcon size={48} strokeWidth={1} style={{ marginBottom: '1rem', opacity: 0.5 }} />
               <p>Your upscaled image will appear here</p>
             </div>
          )}

          {jobStatus === 'uploading' && (
            <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', gap: '1rem', color: 'var(--primary-color)' }}>
              <Loader2 className="animate-spin" size={32} />
              <span>Uploading image...</span>
            </div>
          )}

          {(jobStatus === 'pending' || jobStatus === 'processing') && (
            <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', gap: '1rem', color: 'var(--primary-color)' }}>
              <Loader2 className="animate-spin" size={32} style={{ animation: 'spin 1s linear infinite' }} />
              <span>{jobStatus === 'pending' ? 'Queued...' : 'Upscaling...'}</span>
              <p style={{ fontSize: '0.875rem', color: 'var(--text-muted)', margin: 0 }}>This may take a minute.</p>
              
              <style dangerouslySetInnerHTML={{__html: `
                @keyframes spin { 100% { transform: rotate(360deg); } }
                .animate-spin { animation: spin 1s linear infinite; }
              `}} />
            </div>
          )}

          {jobStatus === 'failed' && (
            <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', gap: '1rem', color: '#ef4444', textAlign: 'center' }}>
              <AlertCircle size={48} />
              <div>
                <strong>Upscale Failed</strong>
                <p style={{ margin: '0.5rem 0 0', fontSize: '0.875rem', color: 'var(--text-muted)' }}>{errorMessage}</p>
              </div>
            </div>
          )}

          {jobStatus === 'completed' && resultUrl && (
            <div style={{ width: '100%', display: 'flex', flexDirection: 'column', gap: '1rem' }}>
              <a href={resultUrl} target="_blank" rel="noreferrer" style={{ display: 'block', overflow: 'hidden', borderRadius: '4px' }}>
                <img 
                  src={resultUrl} 
                  alt="Upscaled result" 
                  style={{ width: '100%', height: 'auto', objectFit: 'contain', maxHeight: '500px', display: 'block' }} 
                />
              </a>
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', fontSize: '0.875rem' }}>
                <span style={{ color: 'var(--text-muted)' }}>Detected Style: <strong>{resultStyle}</strong></span>
                <button onClick={handleReset} className="btn btn-secondary">Upscale Another</button>
              </div>
            </div>
          )}

        </div>
      </div>
    </div>
  );
}
