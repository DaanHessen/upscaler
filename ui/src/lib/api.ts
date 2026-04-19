import { supabase } from './supabase';

export async function fetchApi(endpoint: string, options: RequestInit = {}) {
  const { data: { session } } = await supabase.auth.getSession();
  
  const headers = new Headers(options.headers || {});
  
  if (session?.access_token) {
    headers.set('Authorization', `Bearer ${session.access_token}`);
  }

  // Use proxy path in dev mode, absolute path when not available
  const url = endpoint.startsWith('/') ? endpoint : `/${endpoint}`;

  const response = await fetch(url, {
    ...options,
    headers,
  });

  if (!response.ok) {
    // Attempt to read error message if provided by backend
    let message = 'API request failed';
    try {
      const errorData = await response.json();
      message = errorData.error || message;
    } catch {
      // Fallback
      message = response.statusText || message;
    }
    throw new Error(message);
  }

  return response;
}
