import { useState, useEffect } from 'react';
import { Outlet, Link, useNavigate } from 'react-router-dom';
import { Moon, Sun, LogOut } from 'lucide-react';
import { useAuth } from './AuthContext';

export default function Layout() {
  const [theme, setTheme] = useState<'dark' | 'light'>('dark');
  const { user, signOut } = useAuth();
  const navigate = useNavigate();

  useEffect(() => {
    const htmlElement = document.documentElement;
    if (theme === 'light') {
      htmlElement.classList.add('light');
      htmlElement.classList.remove('dark');
    } else {
      htmlElement.classList.add('dark');
      htmlElement.classList.remove('light');
    }
  }, [theme]);

  const toggleTheme = () => setTheme((prev) => (prev === 'dark' ? 'light' : 'dark'));

  const handleSignOut = async () => {
    await signOut();
    navigate('/login');
  };

  return (
    <div className="app-container">
      <header className="header">
        <div className="logo">
          <Link to="/" style={{ color: 'inherit', textDecoration: 'none' }}>
            Gemini Upscaler
          </Link>
        </div>
        <nav style={{ display: 'flex', gap: '1rem', alignItems: 'center' }}>
          {user ? (
            <>
              <span style={{ fontSize: '0.875rem', color: 'var(--text-muted)' }}>
                {user.email}
              </span>
              <button className="btn btn-secondary" onClick={handleSignOut} aria-label="Sign Out">
                <LogOut size={18} />
              </button>
            </>
          ) : (
            <Link to="/login" className="btn btn-secondary" style={{ textDecoration: 'none' }}>
              Login
            </Link>
          )}
          <button className="btn btn-secondary" onClick={toggleTheme} aria-label="Toggle Theme">
            {theme === 'dark' ? <Sun size={18} /> : <Moon size={18} />}
          </button>
        </nav>
      </header>
      
      <main className="main-content">
        <Outlet />
      </main>
    </div>
  );
}
