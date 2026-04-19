import { useState, useEffect } from 'react';
import { Outlet, Link, useNavigate, useLocation } from 'react-router-dom';
import { Moon, Sun, LogOut, Image as ImageIcon, History, Settings, Shield } from 'lucide-react';
import { useAuth } from './AuthContext';

export default function Layout() {
  const [theme, setTheme] = useState<'dark' | 'light'>('dark');
  const { user, signOut } = useAuth();
  const navigate = useNavigate();
  const location = useLocation();

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

  const NavLink = ({ to, children, icon: Icon }: { to: string, children: React.ReactNode, icon: React.ElementType }) => {
    const isActive = location.pathname === to;
    return (
      <Link 
        to={to} 
        style={{ 
          display: 'flex', 
          alignItems: 'center', 
          gap: '0.5rem', 
          padding: '0.5rem 0.75rem', 
          borderRadius: '6px',
          textDecoration: 'none',
          color: isActive ? 'var(--primary-color)' : 'var(--text-muted)',
          backgroundColor: isActive ? 'rgba(59, 130, 246, 0.1)' : 'transparent',
          fontWeight: isActive ? 500 : 400,
          transition: 'all 0.2s ease'
        }}
      >
        <Icon size={18} />
        <span className="hide-mobile">{children}</span>
      </Link>
    );
  };

  return (
    <div className="app-container">
      <header className="header">
        <div className="logo" style={{ display: 'flex', alignItems: 'center', gap: '2rem' }}>
          <Link to="/" style={{ color: 'inherit', textDecoration: 'none' }}>
            Gemini Upscaler
          </Link>
          
          {user && (
            <nav style={{ display: 'flex', gap: '0.5rem' }}>
              <NavLink to="/" icon={ImageIcon}>Upscale</NavLink>
              <NavLink to="/history" icon={History}>History</NavLink>
              <NavLink to="/settings" icon={Settings}>Settings</NavLink>
              <NavLink to="/admin" icon={Shield}>Admin</NavLink>
            </nav>
          )}
        </div>
        
        <nav style={{ display: 'flex', gap: '1rem', alignItems: 'center' }}>
          {user ? (
            <>
              <span style={{ fontSize: '0.875rem', color: 'var(--text-muted)' }} className="hide-mobile">
                {user.email}
              </span>
              <button className="btn btn-secondary" onClick={handleSignOut} aria-label="Sign Out" title="Sign Out">
                <LogOut size={18} />
              </button>
            </>
          ) : (
            <Link to="/login" className="btn btn-secondary" style={{ textDecoration: 'none' }}>
              Login
            </Link>
          )}
          <button className="btn btn-secondary" onClick={toggleTheme} aria-label="Toggle Theme" title="Toggle Theme">
            {theme === 'dark' ? <Sun size={18} /> : <Moon size={18} />}
          </button>
        </nav>
      </header>
      
      <main className="main-content">
        <Outlet />
      </main>

      <style dangerouslySetInnerHTML={{__html: `
        @media (max-width: 768px) {
          .hide-mobile { display: none !important; }
        }
      `}} />
    </div>
  );
}
