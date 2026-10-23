import React, { useState, useEffect, useRef } from 'react';
import {
  Server, Globe, Cpu, HardDrive, Activity, Shield, Terminal as TerminalIcon,
  Archive, FileText, Lock, RefreshCw, Play, Square, RotateCcw, CheckCircle,
  AlertTriangle, XCircle, Heart, ChevronRight, Search, Upload, Download, Key
} from 'lucide-react';
import { translations } from './i18n';
import DonationModal from './components/DonationModal';

export default function App() {
  const [lang, setLang] = useState(localStorage.getItem('astral_lang') || 'pt-BR');
  const t = translations[lang] || translations['pt-BR'];

  // Routing state
  const [currentRoute, setCurrentRoute] = useState('/infra/webserver/dashboard');
  const [isDonateOpen, setIsDonateOpen] = useState(false);

  // Auth state
  const [token, setToken] = useState(localStorage.getItem('astral_web_token') || '');
  const [operator, setOperator] = useState(null);
  const [authLoading, setAuthLoading] = useState(false);
  const [authError, setAuthError] = useState('');
  const [loginForm, setLoginForm] = useState({ username: '', password: '', provider: 'pam' });

  // Data state
  const [discovery, setDiscovery] = useState(null);
  const [loading, setLoading] = useState(false);
  const [statusMessage, setStatusMessage] = useState({ type: '', text: '' });

  // Module-specific states
  const [nginxConfig, setNginxConfig] = useState('');
  const [vhosts, setVhosts] = useState([]);
  const [sslCerts, setSslCerts] = useState([]);
  const [tomcatApps, setTomcatApps] = useState([]);
  const [tomcatConnectors, setTomcatConnectors] = useState([]);
  const [jvmConfig, setJvmConfig] = useState(null);
  const [logs, setLogs] = useState([]);
  const [logFilter, setLogFilter] = useState('');
  const [activeLogType, setActiveLogType] = useState('nginx_access');
  const [securitySettings, setSecuritySettings] = useState({
    ip_whitelist: [],
    ip_blacklist: [],
    hsts_enabled: true,
    csp_enabled: true,
    xframe_options: 'SAMEORIGIN',
    rate_limit_enabled: false,
    lock_tomcat_manager: true
  });
  const [backups, setBackups] = useState([]);
  const [terminalInput, setTerminalInput] = useState('');
  const [terminalHistory, setTerminalHistory] = useState([
    { cmd: 'init', output: 'Astral Operational Terminal ready. Restricted to administrative tasks.' }
  ]);
  const logEndRef = useRef(null);

  // Sync route from URL hash if available
  useEffect(() => {
    const hash = window.location.hash.replace('#', '');
    if (hash && hash.startsWith('/infra/webserver/')) {
      setCurrentRoute(hash);
    }
  }, []);

  const navigateTo = (route) => {
    setCurrentRoute(route);
    window.location.hash = route;
  };

  const changeLang = (l) => {
    setLang(l);
    localStorage.setItem('astral_lang', l);
  };

  // Auth fetch wrapper
  const apiFetch = async (endpoint, options = {}) => {
    const headers = {
      'Content-Type': 'application/json',
      ...(token ? { 'Authorization': `Bearer ${token}` } : {}),
      ...(options.headers || {})
    };
    try {
      const res = await fetch(endpoint, { ...options, headers });
      if (res.status === 401) {
        setToken('');
        localStorage.removeItem('astral_web_token');
        setOperator(null);
        return null;
      }
      return await res.json();
    } catch (err) {
      console.error(`API Error [${endpoint}]:`, err);
      return null;
    }
  };

  // Initial Auth Check
  useEffect(() => {
    const checkMe = async () => {
      // Check query parameter token first (SSO)
      const urlParams = new URLSearchParams(window.location.search);
      const queryToken = urlParams.get('token');
      const activeTok = queryToken || token;

      if (!activeTok) return;

      const res = await fetch('/api/auth/me', {
        headers: { 'Authorization': `Bearer ${activeTok}` }
      });
      if (res.ok) {
        const data = await res.json();
        if (data.success && data.data) {
          setOperator(data.data);
          setToken(activeTok);
          localStorage.setItem('astral_web_token', activeTok);
          loadAllData(activeTok);
        }
      } else {
        setToken('');
        localStorage.removeItem('astral_web_token');
      }
    };
    checkMe();
  }, []);

  const handleLogin = async (e) => {
    e.preventDefault();
    setAuthLoading(true);
    setAuthError('');
    try {
      const res = await fetch('/api/auth/login', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(loginForm)
      });
      const data = await res.json();
      if (data.success && data.data) {
        setToken(data.data.token);
        localStorage.setItem('astral_web_token', data.data.token);
        setOperator(data.data.operator);
        loadAllData(data.data.token);
      } else {
        setAuthError(data.message || t.login.error);
      }
    } catch (err) {
      setAuthError('Connection error to Web Server Admin API');
    } finally {
      setAuthLoading(false);
    }
  };

  const handleLogout = async () => {
    await apiFetch('/api/auth/logout', { method: 'POST' });
    setToken('');
    localStorage.removeItem('astral_web_token');
    setOperator(null);
  };

  const loadAllData = async (tok) => {
    setLoading(true);
    try {
      // Full discovery
      const discRes = await fetch('/api/discovery', {
        headers: { 'Authorization': `Bearer ${tok}` }
      });
      if (discRes.ok) {
        const d = await discRes.json();
        if (d.data) setDiscovery(d.data);
      }

      // Vhosts
      const vhostRes = await fetch('/api/nginx/vhosts', {
        headers: { 'Authorization': `Bearer ${tok}` }
      });
      if (vhostRes.ok) {
        const v = await vhostRes.json();
        if (v.data) setVhosts(v.data);
      }

      // SSL
      const sslRes = await fetch('/api/nginx/ssl', {
        headers: { 'Authorization': `Bearer ${tok}` }
      });
      if (sslRes.ok) {
        const s = await sslRes.json();
        if (s.data) setSslCerts(s.data);
      }

      // Tomcat Apps & Connectors
      const appsRes = await fetch('/api/tomcat/apps', {
        headers: { 'Authorization': `Bearer ${tok}` }
      });
      if (appsRes.ok) {
        const a = await appsRes.json();
        if (a.data) setTomcatApps(a.data);
      }

      const connRes = await fetch('/api/tomcat/connectors', {
        headers: { 'Authorization': `Bearer ${tok}` }
      });
      if (connRes.ok) {
        const c = await connRes.json();
        if (c.data) setTomcatConnectors(c.data);
      }

      // JVM
      const jvmRes = await fetch('/api/tomcat/jvm', {
        headers: { 'Authorization': `Bearer ${tok}` }
      });
      if (jvmRes.ok) {
        const j = await jvmRes.json();
        if (j.data) setJvmConfig(j.data);
      }

      // Security
      const secRes = await fetch('/api/security/settings', {
        headers: { 'Authorization': `Bearer ${tok}` }
      });
      if (secRes.ok) {
        const sec = await secRes.json();
        if (sec.data) setSecuritySettings(sec.data);
      }

      // Backups
      const bkRes = await fetch('/api/backups', {
        headers: { 'Authorization': `Bearer ${tok}` }
      });
      if (bkRes.ok) {
        const bk = await bkRes.json();
        if (bk.data) setBackups(bk.data);
      }

      // Default Logs
      fetchLogs('nginx_access', tok);
    } catch (e) {
      console.error(e);
    } finally {
      setLoading(false);
    }
  };

  const fetchLogs = async (type, tok = token) => {
    setActiveLogType(type);
    const endpoint = type.startsWith('nginx')
      ? `/api/nginx/logs?log_type=${type.replace('nginx_', '')}&lines=100`
      : `/api/tomcat/logs?log_type=${type.replace('tomcat_', '')}&lines=100`;
    const res = await apiFetch(endpoint);
    if (res && res.data) {
      setLogs(res.data);
    }
  };

  // Nginx controls
  const handleNginxControl = async (action) => {
    const res = await apiFetch('/api/nginx/control', {
      method: 'POST',
      body: JSON.stringify({ action })
    });
    if (res && res.success) {
      setStatusMessage({ type: 'success', text: `Nginx: ${res.message}` });
      loadAllData(token);
    } else {
      setStatusMessage({ type: 'error', text: res ? res.message : 'Falha ao controlar Nginx' });
    }
  };

  const handleNginxTest = async () => {
    const res = await apiFetch('/api/nginx/test');
    if (res && res.success) {
      setStatusMessage({ type: 'success', text: res.message });
    } else {
      setStatusMessage({ type: 'error', text: res ? res.message : 'Erro sintático no Nginx' });
    }
  };

  // Tomcat controls
  const handleTomcatControl = async (action) => {
    const res = await apiFetch('/api/tomcat/control', {
      method: 'POST',
      body: JSON.stringify({ action })
    });
    if (res && res.success) {
      setStatusMessage({ type: 'success', text: `Tomcat: ${res.message}` });
      loadAllData(token);
    } else {
      setStatusMessage({ type: 'error', text: res ? res.message : 'Falha ao controlar Tomcat' });
    }
  };

  // Security Save
  const handleSaveSecurity = async () => {
    const res = await apiFetch('/api/security/apply', {
      method: 'POST',
      body: JSON.stringify(securitySettings)
    });
    if (res && res.success) {
      setStatusMessage({ type: 'success', text: 'Configurações de segurança aplicadas com sucesso.' });
    } else {
      setStatusMessage({ type: 'error', text: 'Erro ao salvar segurança.' });
    }
  };

  // Terminal Exec
  const handleTerminalSubmit = async (e) => {
    e.preventDefault();
    if (!terminalInput.trim()) return;
    const cmd = terminalInput;
    setTerminalInput('');
    const res = await apiFetch('/api/terminal/exec', {
      method: 'POST',
      body: JSON.stringify({ command: cmd })
    });
    const output = res && res.data ? res.data.output : (res?.message || 'Erro ao executar comando.');
    setTerminalHistory(prev => [...prev, { cmd, output }]);
  };

  // ============================================================================
  // RENDER LOGIN IF NOT AUTHENTICATED
  // ============================================================================
  if (!operator) {
    return (
      <div className="min-h-screen bg-slate-950 flex flex-col items-center justify-center p-4 relative overflow-hidden">
        {/* Futuristic background decorations */}
        <div className="absolute top-1/4 left-1/3 w-96 h-96 bg-cyan-600/10 rounded-full blur-3xl pointer-events-none"></div>
        <div className="absolute bottom-1/4 right-1/3 w-96 h-96 bg-indigo-600/10 rounded-full blur-3xl pointer-events-none"></div>

        <div className="w-full max-w-md bg-slate-900 border border-slate-800 rounded-2xl shadow-2xl p-8 relative z-10">
          <div className="flex items-center justify-between pb-6 border-b border-slate-800 mb-6">
            <div className="flex items-center gap-3">
              <div className="p-2.5 rounded-xl bg-cyan-500/10 text-cyan-400 border border-cyan-500/20">
                <Globe className="w-6 h-6" />
              </div>
              <div>
                <h1 className="text-lg font-bold text-white tracking-wide">{t.appName}</h1>
                <p className="text-xs text-slate-400">{t.subtitle}</p>
              </div>
            </div>
            {/* Lang Dropdown */}
            <select
              value={lang}
              onChange={(e) => changeLang(e.target.value)}
              className="bg-slate-950 border border-slate-700 text-xs text-slate-300 rounded px-2 py-1 outline-none"
            >
              <option value="pt-BR">🇧🇷 PT</option>
              <option value="en-US">🇺🇸 EN</option>
              <option value="es-ES">🇪🇸 ES</option>
              <option value="fr-FR">🇫🇷 FR</option>
            </select>
          </div>

          <h2 className="text-base font-semibold text-white mb-1">{t.login.title}</h2>
          <p className="text-xs text-slate-400 mb-6">{t.login.description}</p>

          {authError && (
            <div className="mb-4 p-3 rounded-lg bg-red-950/50 border border-red-800/80 text-xs text-red-300 flex items-center gap-2">
              <AlertTriangle className="w-4 h-4 shrink-0 text-red-400" />
              <span>{authError}</span>
            </div>
          )}

          <form onSubmit={handleLogin} className="space-y-4">
            <div>
              <label className="block text-xs font-medium text-slate-300 mb-1">{t.login.username}</label>
              <input
                type="text"
                required
                value={loginForm.username}
                onChange={(e) => setLoginForm({ ...loginForm, username: e.target.value })}
                placeholder="root, admin ou usuário administrativo"
                className="w-full px-3 py-2 text-sm bg-slate-950 border border-slate-800 rounded-lg text-white focus:outline-none focus:border-cyan-500 transition"
              />
            </div>

            <div>
              <label className="block text-xs font-medium text-slate-300 mb-1">{t.login.password}</label>
              <input
                type="password"
                required
                value={loginForm.password}
                onChange={(e) => setLoginForm({ ...loginForm, password: e.target.value })}
                placeholder="••••••••••••"
                className="w-full px-3 py-2 text-sm bg-slate-950 border border-slate-800 rounded-lg text-white focus:outline-none focus:border-cyan-500 transition"
              />
            </div>

            <div>
              <label className="block text-xs font-medium text-slate-300 mb-1">{t.login.provider}</label>
              <select
                value={loginForm.provider}
                onChange={(e) => setLoginForm({ ...loginForm, provider: e.target.value })}
                className="w-full px-3 py-2 text-sm bg-slate-950 border border-slate-800 rounded-lg text-white focus:outline-none focus:border-cyan-500 transition"
              >
                <option value="pam">{t.login.pam}</option>
                <option value="samba_ad">{t.login.samba}</option>
              </select>
            </div>

            <button
              type="submit"
              disabled={authLoading}
              className="w-full mt-2 py-2.5 px-4 rounded-lg bg-cyan-600 hover:bg-cyan-500 text-white font-medium text-sm transition shadow-lg shadow-cyan-900/30 flex items-center justify-center gap-2"
            >
              {authLoading ? <RefreshCw className="w-4 h-4 animate-spin" /> : <Lock className="w-4 h-4" />}
              <span>{t.actions.login}</span>
            </button>
          </form>

          <div className="mt-6 pt-4 border-t border-slate-800/80 flex items-center justify-between">
            <button
              onClick={() => setIsDonateOpen(true)}
              className="text-xs text-pink-400 hover:text-pink-300 flex items-center gap-1 transition"
            >
              <Heart className="w-3.5 h-3.5 fill-pink-500/20" /> {t.actions.donate}
            </button>
            <span className="text-[11px] text-slate-500">Astral Platform HCI v2.4</span>
          </div>
        </div>

        <DonationModal isOpen={isDonateOpen} onClose={() => setIsDonateOpen(false)} t={t} />
      </div>
    );
  }

  // ============================================================================
  // AUTHENTICATED SPA INTERFACE
  // ============================================================================
  const navItems = [
    { id: '/infra/webserver/dashboard', label: t.nav.dashboard, icon: Globe },
    { id: '/infra/webserver/nginx', label: t.nav.nginx, icon: Server },
    { id: '/infra/webserver/tomcat', label: t.nav.tomcat, icon: Cpu },
    { id: '/infra/webserver/vhosts', label: t.nav.vhosts, icon: HardDrive },
    { id: '/infra/webserver/reverse-proxy', label: t.nav.reverseProxy, icon: Activity },
    { id: '/infra/webserver/ssl', label: t.nav.ssl, icon: Lock },
    { id: '/infra/webserver/monitoring', label: t.nav.monitoring, icon: Activity },
    { id: '/infra/webserver/security', label: t.nav.security, icon: Shield },
    { id: '/infra/webserver/logs', label: t.nav.logs, icon: FileText },
    { id: '/infra/webserver/backups', label: t.nav.backups, icon: Archive },
    { id: '/infra/webserver/terminal', label: t.nav.terminal, icon: TerminalIcon },
  ];

  return (
    <div className="min-h-screen bg-slate-950 text-slate-100 flex flex-col font-sans">
      {/* Top Header */}
      <header className="h-14 border-b border-slate-800 bg-slate-900/80 backdrop-blur px-4 flex items-center justify-between sticky top-0 z-40">
        <div className="flex items-center gap-3">
          <div className="p-1.5 rounded-lg bg-cyan-500/10 text-cyan-400 border border-cyan-500/20">
            <Globe className="w-5 h-5" />
          </div>
          <div>
            <div className="flex items-center gap-2">
              <span className="text-xs text-slate-400 font-mono">Infraestrutura</span>
              <ChevronRight className="w-3 h-3 text-slate-600" />
              <span className="text-sm font-bold text-white tracking-wide">Web Server</span>
            </div>
          </div>
        </div>

        {/* Global Status Badges */}
        <div className="hidden md:flex items-center gap-3 text-xs">
          <div className="flex items-center gap-1.5 px-2.5 py-1 rounded-full bg-slate-800/80 border border-slate-700">
            <span className="w-2 h-2 rounded-full bg-emerald-400 animate-pulse"></span>
            <span className="text-slate-300">Nginx:</span>
            <span className="font-semibold text-emerald-300">{discovery?.nginx?.status || 'Ativo'}</span>
          </div>
          <div className="flex items-center gap-1.5 px-2.5 py-1 rounded-full bg-slate-800/80 border border-slate-700">
            <span className="w-2 h-2 rounded-full bg-cyan-400 animate-pulse"></span>
            <span className="text-slate-300">Tomcat:</span>
            <span className="font-semibold text-cyan-300">{discovery?.tomcat?.status || 'Ativo'}</span>
          </div>
        </div>

        {/* User, Lang, Donate, Logout */}
        <div className="flex items-center gap-3">
          <button
            onClick={() => setIsDonateOpen(true)}
            className="flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium rounded-lg bg-pink-500/10 text-pink-300 border border-pink-500/20 hover:bg-pink-500/20 transition"
          >
            <Heart className="w-3.5 h-3.5 fill-pink-500/20" />
            <span className="hidden sm:inline">{t.actions.donate}</span>
          </button>

          <select
            value={lang}
            onChange={(e) => changeLang(e.target.value)}
            className="bg-slate-950 border border-slate-700 text-xs text-slate-300 rounded px-2 py-1 outline-none"
          >
            <option value="pt-BR">🇧🇷 PT</option>
            <option value="en-US">🇺🇸 EN</option>
            <option value="es-ES">🇪🇸 ES</option>
            <option value="fr-FR">🇫🇷 FR</option>
          </select>

          <div className="flex items-center gap-2 pl-2 border-l border-slate-800">
            <div className="text-right hidden sm:block">
              <p className="text-xs font-semibold text-white leading-none">{operator.username}</p>
              <p className="text-[10px] text-cyan-400 font-mono leading-tight">{operator.provider.toUpperCase()}</p>
            </div>
            <button
              onClick={handleLogout}
              title={t.actions.logout}
              className="p-1.5 text-slate-400 hover:text-red-400 hover:bg-slate-800 rounded transition"
            >
              <XCircle className="w-4 h-4" />
            </button>
          </div>
        </div>
      </header>

      {/* Main Layout Body */}
      <div className="flex-1 flex overflow-hidden">
        {/* Sidebar Navigation */}
        <aside className="w-64 bg-slate-900 border-r border-slate-800 flex flex-col shrink-0">
          <div className="p-3 text-[11px] font-semibold text-slate-400 uppercase tracking-wider">
            Módulos Unificados
          </div>
          <nav className="flex-1 px-2 space-y-0.5 overflow-y-auto">
            {navItems.map((item) => {
              const Icon = item.icon;
              const isActive = currentRoute === item.id;
              return (
                <button
                  key={item.id}
                  onClick={() => navigateTo(item.id)}
                  className={`w-full flex items-center gap-2.5 px-3 py-2 rounded-lg text-xs font-medium transition ${
                    isActive
                      ? 'bg-cyan-500/10 text-cyan-400 border border-cyan-500/20'
                      : 'text-slate-400 hover:text-slate-200 hover:bg-slate-800/60'
                  }`}
                >
                  <Icon className="w-4 h-4 shrink-0" />
                  <span className="truncate">{item.label}</span>
                </button>
              );
            })}
          </nav>

          {/* Quick Refresh */}
          <div className="p-3 border-t border-slate-800">
            <button
              onClick={() => loadAllData(token)}
              disabled={loading}
              className="w-full flex items-center justify-center gap-2 px-3 py-2 text-xs font-medium rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-200 transition"
            >
              <RefreshCw className={`w-3.5 h-3.5 ${loading ? 'animate-spin' : ''}`} />
              <span>{t.actions.refresh}</span>
            </button>
          </div>
        </aside>

        {/* Main Content Area */}
        <main className="flex-1 overflow-y-auto p-6 bg-slate-950">
          {/* Status Alert Banner */}
          {statusMessage.text && (
            <div className={`mb-4 p-3 rounded-lg border text-xs flex items-center justify-between ${
              statusMessage.type === 'error'
                ? 'bg-red-950/60 border-red-800 text-red-200'
                : 'bg-emerald-950/60 border-emerald-800 text-emerald-200'
            }`}>
              <div className="flex items-center gap-2">
                {statusMessage.type === 'error' ? <AlertTriangle className="w-4 h-4" /> : <CheckCircle className="w-4 h-4" />}
                <span>{statusMessage.text}</span>
              </div>
              <button onClick={() => setStatusMessage({ type: '', text: '' })}>
                <XCircle className="w-4 h-4 text-slate-400 hover:text-white" />
              </button>
            </div>
          )}

          {/* ================================================================== */}
          {/* 1. DASHBOARD */}
          {/* ================================================================== */}
          {currentRoute === '/infra/webserver/dashboard' && (
            <div className="space-y-6">
              <div>
                <h2 className="text-xl font-bold text-white">{t.dashboard.overview}</h2>
                <p className="text-xs text-slate-400">Ambiente único de gerenciamento Nginx e Apache Tomcat.</p>
              </div>

              {/* Nginx & Tomcat Cards Grid */}
              <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                {/* Nginx Card */}
                <div className="p-5 rounded-xl bg-slate-900 border border-slate-800 space-y-4">
                  <div className="flex items-center justify-between pb-3 border-b border-slate-800">
                    <div className="flex items-center gap-2.5">
                      <div className="p-2 rounded-lg bg-emerald-500/10 text-emerald-400">
                        <Server className="w-5 h-5" />
                      </div>
                      <div>
                        <h3 className="text-sm font-bold text-white">{t.dashboard.nginxTitle}</h3>
                        <p className="text-[11px] text-slate-400">Versão: {discovery?.nginx?.version || '1.26.x'}</p>
                      </div>
                    </div>
                    <span className="px-2.5 py-1 text-[11px] font-semibold rounded-full bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
                      {discovery?.nginx?.status || 'Ativo'}
                    </span>
                  </div>

                  <div className="grid grid-cols-2 gap-3 text-xs">
                    <div className="p-3 rounded-lg bg-slate-950 border border-slate-800/80">
                      <span className="text-slate-400 block text-[11px]">{t.dashboard.workers}</span>
                      <span className="text-base font-bold text-white font-mono">{discovery?.nginx?.workers_count || 4}</span>
                    </div>
                    <div className="p-3 rounded-lg bg-slate-950 border border-slate-800/80">
                      <span className="text-slate-400 block text-[11px]">{t.dashboard.activeConns}</span>
                      <span className="text-base font-bold text-white font-mono">{discovery?.nginx?.active_connections || 12}</span>
                    </div>
                    <div className="p-3 rounded-lg bg-slate-950 border border-slate-800/80">
                      <span className="text-slate-400 block text-[11px]">{t.dashboard.rps}</span>
                      <span className="text-base font-bold text-white font-mono">{discovery?.nginx?.requests_per_sec || 45.2}</span>
                    </div>
                    <div className="p-3 rounded-lg bg-slate-950 border border-slate-800/80">
                      <span className="text-slate-400 block text-[11px]">{t.dashboard.vhostsCount}</span>
                      <span className="text-base font-bold text-white font-mono">{vhosts.length || 3}</span>
                    </div>
                  </div>

                  <div className="pt-2 flex items-center gap-2">
                    <button onClick={() => handleNginxControl('restart')} className="flex-1 py-1.5 px-3 text-xs font-medium bg-slate-800 hover:bg-slate-700 text-white rounded-lg transition">
                      {t.actions.restart}
                    </button>
                    <button onClick={() => handleNginxControl('reload')} className="flex-1 py-1.5 px-3 text-xs font-medium bg-cyan-600 hover:bg-cyan-500 text-white rounded-lg transition">
                      {t.actions.reload}
                    </button>
                  </div>
                </div>

                {/* Tomcat Card */}
                <div className="p-5 rounded-xl bg-slate-900 border border-slate-800 space-y-4">
                  <div className="flex items-center justify-between pb-3 border-b border-slate-800">
                    <div className="flex items-center gap-2.5">
                      <div className="p-2 rounded-lg bg-cyan-500/10 text-cyan-400">
                        <Cpu className="w-5 h-5" />
                      </div>
                      <div>
                        <h3 className="text-sm font-bold text-white">{t.dashboard.tomcatTitle}</h3>
                        <p className="text-[11px] text-slate-400">Versão: {discovery?.tomcat?.version || '10.1.60'}</p>
                      </div>
                    </div>
                    <span className="px-2.5 py-1 text-[11px] font-semibold rounded-full bg-cyan-500/10 text-cyan-400 border border-cyan-500/20">
                      {discovery?.tomcat?.status || 'Ativo'}
                    </span>
                  </div>

                  <div className="grid grid-cols-2 gap-3 text-xs">
                    <div className="p-3 rounded-lg bg-slate-950 border border-slate-800/80">
                      <span className="text-slate-400 block text-[11px]">{t.dashboard.deployedApps}</span>
                      <span className="text-base font-bold text-white font-mono">{tomcatApps.length || 3}</span>
                    </div>
                    <div className="p-3 rounded-lg bg-slate-950 border border-slate-800/80">
                      <span className="text-slate-400 block text-[11px]">{t.dashboard.activeSessions}</span>
                      <span className="text-base font-bold text-white font-mono">{discovery?.tomcat?.active_sessions || 8}</span>
                    </div>
                    <div className="p-3 rounded-lg bg-slate-950 border border-slate-800/80">
                      <span className="text-slate-400 block text-[11px]">{t.dashboard.jvmHeap}</span>
                      <span className="text-base font-bold text-white font-mono">{discovery?.tomcat?.heap_used_mb || 342} MB</span>
                    </div>
                    <div className="p-3 rounded-lg bg-slate-950 border border-slate-800/80">
                      <span className="text-slate-400 block text-[11px]">{t.dashboard.gcType}</span>
                      <span className="text-base font-bold text-white font-mono">{jvmConfig?.gc_type || 'G1GC'}</span>
                    </div>
                  </div>

                  <div className="pt-2 flex items-center gap-2">
                    <button onClick={() => handleTomcatControl('restart')} className="flex-1 py-1.5 px-3 text-xs font-medium bg-slate-800 hover:bg-slate-700 text-white rounded-lg transition">
                      {t.actions.restart}
                    </button>
                    <button onClick={() => handleTomcatControl('reload')} className="flex-1 py-1.5 px-3 text-xs font-medium bg-cyan-600 hover:bg-cyan-500 text-white rounded-lg transition">
                      {t.actions.reload}
                    </button>
                  </div>
                </div>
              </div>

              {/* Host System Telemetry */}
              <div className="p-5 rounded-xl bg-slate-900 border border-slate-800">
                <h3 className="text-sm font-bold text-white mb-4">{t.dashboard.systemTitle}</h3>
                <div className="grid grid-cols-1 sm:grid-cols-3 gap-4">
                  <div className="p-4 rounded-lg bg-slate-950 border border-slate-800/80">
                    <div className="flex items-center justify-between text-xs text-slate-400 mb-2">
                      <span>{t.dashboard.cpuUsage}</span>
                      <span className="font-mono text-cyan-400">{discovery?.system?.cpu_usage_pct || 14.8}%</span>
                    </div>
                    <div className="w-full h-2 rounded-full bg-slate-800 overflow-hidden">
                      <div className="h-full bg-cyan-500" style={{ width: `${discovery?.system?.cpu_usage_pct || 14.8}%` }}></div>
                    </div>
                  </div>

                  <div className="p-4 rounded-lg bg-slate-950 border border-slate-800/80">
                    <div className="flex items-center justify-between text-xs text-slate-400 mb-2">
                      <span>{t.dashboard.ramUsage}</span>
                      <span className="font-mono text-emerald-400">{discovery?.system?.ram_usage_pct || 42.1}%</span>
                    </div>
                    <div className="w-full h-2 rounded-full bg-slate-800 overflow-hidden">
                      <div className="h-full bg-emerald-500" style={{ width: `${discovery?.system?.ram_usage_pct || 42.1}%` }}></div>
                    </div>
                  </div>

                  <div className="p-4 rounded-lg bg-slate-950 border border-slate-800/80">
                    <div className="flex items-center justify-between text-xs text-slate-400 mb-2">
                      <span>{t.dashboard.diskUsage}</span>
                      <span className="font-mono text-indigo-400">{discovery?.system?.disk_usage_pct || 28.5}%</span>
                    </div>
                    <div className="w-full h-2 rounded-full bg-slate-800 overflow-hidden">
                      <div className="h-full bg-indigo-500" style={{ width: `${discovery?.system?.disk_usage_pct || 28.5}%` }}></div>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          )}

          {/* ================================================================== */}
          {/* 2. NGINX */}
          {/* ================================================================== */}
          {currentRoute === '/infra/webserver/nginx' && (
            <div className="space-y-6">
              <div className="flex items-center justify-between">
                <div>
                  <h2 className="text-xl font-bold text-white">{t.nginx.title}</h2>
                  <p className="text-xs text-slate-400">Controle de serviço e validação de sintaxe (/etc/nginx/)</p>
                </div>
                <div className="flex items-center gap-2">
                  <button onClick={handleNginxTest} className="px-3 py-1.5 text-xs font-medium bg-slate-800 hover:bg-slate-700 text-white rounded-lg transition flex items-center gap-1.5">
                    <CheckCircle className="w-3.5 h-3.5 text-emerald-400" />
                    <span>{t.actions.testConfig}</span>
                  </button>
                  <button onClick={() => handleNginxControl('restart')} className="px-3 py-1.5 text-xs font-medium bg-red-950 hover:bg-red-900 text-red-200 border border-red-800 rounded-lg transition">
                    {t.actions.restart}
                  </button>
                  <button onClick={() => handleNginxControl('reload')} className="px-3 py-1.5 text-xs font-medium bg-cyan-600 hover:bg-cyan-500 text-white rounded-lg transition">
                    {t.actions.reload}
                  </button>
                </div>
              </div>

              {/* Status Banner */}
              <div className="p-4 rounded-xl bg-slate-900 border border-slate-800 flex items-center justify-between">
                <div>
                  <span className="text-xs text-slate-400 block">Status do Daemon:</span>
                  <span className="text-base font-bold text-emerald-400 font-mono">active (running) - PID {discovery?.nginx?.pid || 1284}</span>
                </div>
                <div className="text-right text-xs text-slate-400">
                  <p>Worker Processes: {discovery?.nginx?.workers_count || 4}</p>
                  <p>Portas em Escuta: 80 (HTTP), 443 (HTTPS)</p>
                </div>
              </div>

              {/* Config Viewer */}
              <div className="p-5 rounded-xl bg-slate-900 border border-slate-800 space-y-3">
                <div className="flex items-center justify-between">
                  <h3 className="text-sm font-bold text-white">{t.nginx.configTitle}</h3>
                  <span className="text-[11px] text-slate-500">{t.nginx.saveNotice}</span>
                </div>
                <pre className="p-4 rounded-lg bg-slate-950 border border-slate-800 text-xs font-mono text-slate-300 overflow-x-auto max-h-96">
{`user nginx;
worker_processes auto;
error_log /var/log/nginx/error.log notice;
pid /run/nginx.pid;

events {
    worker_connections 1024;
}

http {
    include       /etc/nginx/mime.types;
    default_type  application/octet-stream;
    log_format  main  '$remote_addr - $remote_user [$time_local] "$request" '
                      '$status $body_bytes_sent "$http_referer" '
                      '"$http_user_agent" "$http_x_forwarded_for"';
    access_log  /var/log/nginx/access.log  main;
    sendfile        on;
    keepalive_timeout  65;

    include /etc/nginx/conf.d/*.conf;
    include /etc/nginx/default.d/*.conf;
}`}
                </pre>
              </div>
            </div>
          )}

          {/* ================================================================== */}
          {/* 3. TOMCAT */}
          {/* ================================================================== */}
          {currentRoute === '/infra/webserver/tomcat' && (
            <div className="space-y-6">
              <div className="flex items-center justify-between">
                <div>
                  <h2 className="text-xl font-bold text-white">{t.tomcat.title}</h2>
                  <p className="text-xs text-slate-400">Gerenciamento do Apache Tomcat 10.1, conectores e aplicações WAR</p>
                </div>
                <div className="flex items-center gap-2">
                  <button onClick={() => handleTomcatControl('restart')} className="px-3 py-1.5 text-xs font-medium bg-red-950 hover:bg-red-900 text-red-200 border border-red-800 rounded-lg transition">
                    {t.actions.restart}
                  </button>
                  <button onClick={() => handleTomcatControl('reload')} className="px-3 py-1.5 text-xs font-medium bg-cyan-600 hover:bg-cyan-500 text-white rounded-lg transition">
                    {t.actions.reload}
                  </button>
                </div>
              </div>

              {/* Deployed Apps Table */}
              <div className="p-5 rounded-xl bg-slate-900 border border-slate-800 space-y-4">
                <div className="flex items-center justify-between">
                  <h3 className="text-sm font-bold text-white">{t.tomcat.appsTitle}</h3>
                  <button className="px-3 py-1.5 text-xs font-medium bg-cyan-600 hover:bg-cyan-500 text-white rounded-lg transition flex items-center gap-1.5">
                    <Upload className="w-3.5 h-3.5" />
                    <span>{t.tomcat.deployWar}</span>
                  </button>
                </div>

                <div className="overflow-x-auto">
                  <table className="w-full text-left text-xs">
                    <thead>
                      <tr className="border-b border-slate-800 text-slate-400">
                        <th className="pb-2">{t.tomcat.appPath}</th>
                        <th className="pb-2">{t.tomcat.appName}</th>
                        <th className="pb-2">{t.tomcat.appStatus}</th>
                        <th className="pb-2">{t.tomcat.appSessions}</th>
                        <th className="pb-2 text-right">{t.actions.close}</th>
                      </tr>
                    </thead>
                    <tbody className="divide-y divide-slate-800/60 font-mono">
                      {tomcatApps.length > 0 ? (
                        tomcatApps.map((app, idx) => (
                          <tr key={idx} className="hover:bg-slate-800/30">
                            <td className="py-2.5 text-cyan-400 font-semibold">{app.context_path}</td>
                            <td className="py-2.5 text-slate-300">{app.name}</td>
                            <td className="py-2.5">
                              <span className="px-2 py-0.5 rounded text-[10px] bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
                                {app.status}
                              </span>
                            </td>
                            <td className="py-2.5 text-slate-400">{app.active_sessions}</td>
                            <td className="py-2.5 text-right font-sans">
                              <button className="text-red-400 hover:text-red-300 text-xs">{t.actions.delete}</button>
                            </td>
                          </tr>
                        ))
                      ) : (
                        <tr>
                          <td colSpan="5" className="py-4 text-center text-slate-500 font-sans">
                            Aplicações padrão em /opt/tomcat/webapps (ROOT, manager, host-manager)
                          </td>
                        </tr>
                      )}
                    </tbody>
                  </table>
                </div>
              </div>

              {/* Connectors & JVM */}
              <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                <div className="p-5 rounded-xl bg-slate-900 border border-slate-800 space-y-3">
                  <h3 className="text-sm font-bold text-white">{t.tomcat.connectorsTitle}</h3>
                  <div className="space-y-2 text-xs">
                    <div className="p-3 rounded-lg bg-slate-950 border border-slate-800 flex justify-between">
                      <span className="text-slate-400">HTTP/1.1 Conector:</span>
                      <span className="font-mono text-cyan-400 font-semibold">Porta 8080 (TCP)</span>
                    </div>
                    <div className="p-3 rounded-lg bg-slate-950 border border-slate-800 flex justify-between">
                      <span className="text-slate-400">AJP/1.3 Conector:</span>
                      <span className="font-mono text-slate-300 font-semibold">Porta 8009 (Desabilitado)</span>
                    </div>
                  </div>
                </div>

                <div className="p-5 rounded-xl bg-slate-900 border border-slate-800 space-y-3">
                  <h3 className="text-sm font-bold text-white">{t.tomcat.jvmTitle}</h3>
                  <div className="space-y-2 text-xs font-mono">
                    <div className="p-3 rounded-lg bg-slate-950 border border-slate-800">
                      <span className="text-slate-500 block text-[10px]">JAVA_OPTS</span>
                      <span className="text-slate-300">-Xms512m -Xmx2048m -XX:+UseG1GC</span>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          )}

          {/* ================================================================== */}
          {/* 4. VHOSTS */}
          {/* ================================================================== */}
          {currentRoute === '/infra/webserver/vhosts' && (
            <div className="space-y-6">
              <div className="flex items-center justify-between">
                <div>
                  <h2 className="text-xl font-bold text-white">{t.vhosts.title}</h2>
                  <p className="text-xs text-slate-400">Gerenciador de Virtual Hosts (sites-available / sites-enabled)</p>
                </div>
                <button className="px-3 py-1.5 text-xs font-medium bg-cyan-600 hover:bg-cyan-500 text-white rounded-lg transition flex items-center gap-1.5">
                  <span>+ {t.vhosts.newVhost}</span>
                </button>
              </div>

              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                {vhosts.length > 0 ? (
                  vhosts.map((vh, idx) => (
                    <div key={idx} className="p-4 rounded-xl bg-slate-900 border border-slate-800 space-y-3">
                      <div className="flex items-center justify-between">
                        <span className="text-sm font-bold text-white font-mono">{vh.domain}</span>
                        <span className={`px-2 py-0.5 rounded text-[10px] ${
                          vh.enabled ? 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/20' : 'bg-slate-800 text-slate-400'
                        }`}>
                          {vh.enabled ? t.status.enabled : t.status.disabled}
                        </span>
                      </div>
                      <div className="text-xs text-slate-400 space-y-1">
                        <p><span className="text-slate-500">{t.vhosts.listen}:</span> {vh.port}</p>
                        <p><span className="text-slate-500">{t.vhosts.root}:</span> {vh.root_path}</p>
                        <p><span className="text-slate-500">{t.vhosts.sslEnabled}:</span> {vh.ssl_enabled ? 'Sim' : 'Não'}</p>
                      </div>
                      <div className="pt-2 flex items-center justify-end gap-2 border-t border-slate-800">
                        <button className="px-2.5 py-1 text-xs text-cyan-400 hover:underline">{t.vhosts.editVhost}</button>
                      </div>
                    </div>
                  ))
                ) : (
                  <div className="col-span-2 p-8 text-center text-slate-500 text-xs border border-dashed border-slate-800 rounded-xl">
                    Nenhum Virtual Host adicional configurado. Servindo bloco padrão.
                  </div>
                )}
              </div>
            </div>
          )}

          {/* ================================================================== */}
          {/* 5. REVERSE PROXY */}
          {/* ================================================================== */}
          {currentRoute === '/infra/webserver/reverse-proxy' && (
            <div className="space-y-6">
              <div>
                <h2 className="text-xl font-bold text-white">{t.reverseProxy.title}</h2>
                <p className="text-xs text-slate-400">Balanceamento de carga e proxy reverso para backends</p>
              </div>

              <div className="p-5 rounded-xl bg-slate-900 border border-slate-800 space-y-4">
                <h3 className="text-sm font-bold text-white">{t.reverseProxy.upstreams}</h3>
                <div className="p-4 rounded-lg bg-slate-950 border border-slate-800 space-y-2">
                  <div className="flex items-center justify-between text-xs">
                    <span className="font-mono text-cyan-400 font-bold">upstream tomcat_cluster</span>
                    <span className="text-slate-400">{t.reverseProxy.algorithm}: Round-Robin</span>
                  </div>
                  <div className="text-xs font-mono text-slate-300 space-y-1">
                    <p>server 127.0.0.1:8080 max_fails=3 fail_timeout=10s;</p>
                  </div>
                </div>

                <h3 className="text-sm font-bold text-white pt-2">{t.reverseProxy.locations}</h3>
                <div className="p-4 rounded-lg bg-slate-950 border border-slate-800 space-y-2 text-xs font-mono text-slate-300">
                  <p className="text-cyan-400 font-bold">location /webserver/ {"{"}</p>
                  <p className="pl-4">proxy_pass http://127.0.0.1:8096;</p>
                  <p className="pl-4">proxy_http_version 1.1;</p>
                  <p className="pl-4">proxy_set_header Upgrade $http_upgrade;</p>
                  <p className="pl-4">proxy_set_header Connection "upgrade";</p>
                  <p>{"}"}</p>
                </div>
              </div>
            </div>
          )}

          {/* ================================================================== */}
          {/* 6. SSL */}
          {/* ================================================================== */}
          {currentRoute === '/infra/webserver/ssl' && (
            <div className="space-y-6">
              <div>
                <h2 className="text-xl font-bold text-white">{t.ssl.title}</h2>
                <p className="text-xs text-slate-400">Certificados X.509, renovação automática Step-CA e cifras TLS</p>
              </div>

              <div className="p-5 rounded-xl bg-slate-900 border border-slate-800 space-y-4">
                <h3 className="text-sm font-bold text-white">{t.ssl.certList}</h3>
                <div className="space-y-3">
                  {sslCerts.length > 0 ? (
                    sslCerts.map((c, idx) => (
                      <div key={idx} className="p-4 rounded-lg bg-slate-950 border border-slate-800 flex items-center justify-between">
                        <div>
                          <p className="text-sm font-bold text-white font-mono">{c.domain}</p>
                          <p className="text-xs text-slate-400">Emissor: {c.issuer}</p>
                          <p className="text-[11px] text-slate-500">Validade: {c.valid_until}</p>
                        </div>
                        <span className="px-2.5 py-1 text-xs font-semibold rounded-full bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
                          {c.days_left} dias restantes
                        </span>
                      </div>
                    ))
                  ) : (
                    <div className="p-4 rounded-lg bg-slate-950 border border-slate-800 flex items-center justify-between">
                      <div>
                        <p className="text-sm font-bold text-white font-mono">dc-erp.corp.lan</p>
                        <p className="text-xs text-slate-400">Emissor: CORP Root CA (Smallstep PKI)</p>
                        <p className="text-[11px] text-slate-500">Renovação automática via cron diário</p>
                      </div>
                      <span className="px-2.5 py-1 text-xs font-semibold rounded-full bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
                        Válido (365 dias)
                      </span>
                    </div>
                  )}
                </div>
              </div>
            </div>
          )}

          {/* ================================================================== */}
          {/* 7. MONITORING */}
          {/* ================================================================== */}
          {currentRoute === '/infra/webserver/monitoring' && (
            <div className="space-y-6">
              <div>
                <h2 className="text-xl font-bold text-white">{t.monitoring.title}</h2>
                <p className="text-xs text-slate-400">Telemetria ao vivo, taxa de tráfego e sockets de rede</p>
              </div>

              <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                <div className="p-5 rounded-xl bg-slate-900 border border-slate-800 space-y-3">
                  <h3 className="text-sm font-bold text-white">{t.monitoring.sockets}</h3>
                  <div className="space-y-2 text-xs font-mono">
                    <div className="p-2.5 rounded bg-slate-950 border border-slate-800 flex justify-between">
                      <span className="text-slate-400">TCP *:80</span>
                      <span className="text-emerald-400">nginx: master</span>
                    </div>
                    <div className="p-2.5 rounded bg-slate-950 border border-slate-800 flex justify-between">
                      <span className="text-slate-400">TCP *:443</span>
                      <span className="text-emerald-400">nginx: master</span>
                    </div>
                    <div className="p-2.5 rounded bg-slate-950 border border-slate-800 flex justify-between">
                      <span className="text-slate-400">TCP *:8080</span>
                      <span className="text-cyan-400">java (tomcat)</span>
                    </div>
                    <div className="p-2.5 rounded bg-slate-950 border border-slate-800 flex justify-between">
                      <span className="text-slate-400">TCP 127.0.0.1:8096</span>
                      <span className="text-indigo-400">astral-webserver</span>
                    </div>
                  </div>
                </div>

                <div className="p-5 rounded-xl bg-slate-900 border border-slate-800 space-y-3">
                  <h3 className="text-sm font-bold text-white">{t.monitoring.threadPool}</h3>
                  <div className="p-4 rounded-lg bg-slate-950 border border-slate-800 space-y-3 text-xs">
                    <div className="flex justify-between">
                      <span className="text-slate-400">Threads Ativas:</span>
                      <span className="font-mono text-cyan-400 font-bold">14 / 200</span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-slate-400">Threads em Espera:</span>
                      <span className="font-mono text-slate-300 font-bold">186</span>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          )}

          {/* ================================================================== */}
          {/* 8. SECURITY */}
          {/* ================================================================== */}
          {currentRoute === '/infra/webserver/security' && (
            <div className="space-y-6">
              <div className="flex items-center justify-between">
                <div>
                  <h2 className="text-xl font-bold text-white">{t.security.title}</h2>
                  <p className="text-xs text-slate-400">Firewall de aplicação, cabeçalhos de segurança e listas de acesso</p>
                </div>
                <button
                  onClick={handleSaveSecurity}
                  className="px-4 py-2 text-xs font-medium bg-cyan-600 hover:bg-cyan-500 text-white rounded-lg transition"
                >
                  {t.actions.save}
                </button>
              </div>

              <div className="p-5 rounded-xl bg-slate-900 border border-slate-800 space-y-4">
                <h3 className="text-sm font-bold text-white">{t.security.headers}</h3>
                <div className="grid grid-cols-1 sm:grid-cols-2 gap-4 text-xs">
                  <label className="flex items-center gap-3 p-3 rounded-lg bg-slate-950 border border-slate-800 cursor-pointer">
                    <input
                      type="checkbox"
                      checked={securitySettings.hsts_enabled}
                      onChange={(e) => setSecuritySettings({ ...securitySettings, hsts_enabled: e.target.checked })}
                      className="rounded bg-slate-900 border-slate-700 text-cyan-500 focus:ring-0"
                    />
                    <div>
                      <span className="font-semibold text-white block">{t.security.hsts}</span>
                      <span className="text-slate-400 text-[11px]">Strict-Transport-Security: max-age=31536000</span>
                    </div>
                  </label>

                  <label className="flex items-center gap-3 p-3 rounded-lg bg-slate-950 border border-slate-800 cursor-pointer">
                    <input
                      type="checkbox"
                      checked={securitySettings.lock_tomcat_manager}
                      onChange={(e) => setSecuritySettings({ ...securitySettings, lock_tomcat_manager: e.target.checked })}
                      className="rounded bg-slate-900 border-slate-700 text-cyan-500 focus:ring-0"
                    />
                    <div>
                      <span className="font-semibold text-white block">{t.security.managerLock}</span>
                      <span className="text-slate-400 text-[11px]">Restringe /manager apenas ao localhost</span>
                    </div>
                  </label>
                </div>
              </div>
            </div>
          )}

          {/* ================================================================== */}
          {/* 9. LOGS */}
          {/* ================================================================== */}
          {currentRoute === '/infra/webserver/logs' && (
            <div className="space-y-4">
              <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4">
                <div>
                  <h2 className="text-xl font-bold text-white">{t.logs.title}</h2>
                  <p className="text-xs text-slate-400">Transmissão contínua de eventos do Nginx e Apache Tomcat</p>
                </div>
                <div className="flex items-center gap-2">
                  <button
                    onClick={() => fetchLogs('nginx_access')}
                    className={`px-3 py-1.5 text-xs rounded-lg font-medium transition ${
                      activeLogType === 'nginx_access' ? 'bg-cyan-600 text-white' : 'bg-slate-800 text-slate-300'
                    }`}
                  >
                    Nginx Access
                  </button>
                  <button
                    onClick={() => fetchLogs('nginx_error')}
                    className={`px-3 py-1.5 text-xs rounded-lg font-medium transition ${
                      activeLogType === 'nginx_error' ? 'bg-cyan-600 text-white' : 'bg-slate-800 text-slate-300'
                    }`}
                  >
                    Nginx Error
                  </button>
                  <button
                    onClick={() => fetchLogs('tomcat_catalina')}
                    className={`px-3 py-1.5 text-xs rounded-lg font-medium transition ${
                      activeLogType === 'tomcat_catalina' ? 'bg-cyan-600 text-white' : 'bg-slate-800 text-slate-300'
                    }`}
                  >
                    Tomcat Catalina
                  </button>
                </div>
              </div>

              {/* Filter */}
              <div className="relative">
                <Search className="w-4 h-4 text-slate-500 absolute left-3 top-2.5" />
                <input
                  type="text"
                  value={logFilter}
                  onChange={(e) => setLogFilter(e.target.value)}
                  placeholder={t.logs.searchLogs}
                  className="w-full pl-9 pr-3 py-2 text-xs bg-slate-900 border border-slate-800 rounded-lg text-white focus:outline-none focus:border-cyan-500"
                />
              </div>

              {/* Log View Container */}
              <div className="p-4 rounded-xl bg-slate-950 border border-slate-800 font-mono text-xs text-slate-300 h-[500px] overflow-y-auto space-y-1">
                {logs
                  .filter(l => !logFilter || l.toLowerCase().includes(logFilter.toLowerCase()))
                  .map((log, idx) => (
                    <div key={idx} className="hover:bg-slate-900/50 py-0.5 px-1 rounded break-all">
                      {log}
                    </div>
                  ))}
                <div ref={logEndRef} />
              </div>
            </div>
          )}

          {/* ================================================================== */}
          {/* 10. BACKUPS */}
          {/* ================================================================== */}
          {currentRoute === '/infra/webserver/backups' && (
            <div className="space-y-6">
              <div className="flex items-center justify-between">
                <div>
                  <h2 className="text-xl font-bold text-white">{t.backups.title}</h2>
                  <p className="text-xs text-slate-400">{t.backups.snapshotNotice}</p>
                </div>
                <button
                  onClick={async () => {
                    const res = await apiFetch('/api/backups/create', { method: 'POST' });
                    if (res && res.success) {
                      setStatusMessage({ type: 'success', text: 'Snapshot criado com sucesso.' });
                      loadAllData(token);
                    }
                  }}
                  className="px-3 py-1.5 text-xs font-medium bg-cyan-600 hover:bg-cyan-500 text-white rounded-lg transition"
                >
                  + {t.backups.createSnapshot}
                </button>
              </div>

              <div className="space-y-3">
                {backups.length > 0 ? (
                  backups.map((bk, idx) => (
                    <div key={idx} className="p-4 rounded-xl bg-slate-900 border border-slate-800 flex items-center justify-between">
                      <div>
                        <p className="text-sm font-bold text-white font-mono">{bk.id}</p>
                        <p className="text-xs text-slate-400">{t.backups.createdAt}: {bk.created_at}</p>
                        <p className="text-[11px] text-slate-500 font-mono">SHA256: {bk.sha256}</p>
                      </div>
                      <button className="px-3 py-1.5 text-xs font-medium bg-slate-800 hover:bg-slate-700 text-white rounded-lg transition">
                        {t.backups.rollback}
                      </button>
                    </div>
                  ))
                ) : (
                  <div className="p-8 text-center text-slate-500 text-xs border border-dashed border-slate-800 rounded-xl">
                    Nenhum snapshot registrado ainda. Crie o primeiro backup das configurações.
                  </div>
                )}
              </div>
            </div>
          )}

          {/* ================================================================== */}
          {/* 11. TERMINAL */}
          {/* ================================================================== */}
          {currentRoute === '/infra/webserver/terminal' && (
            <div className="space-y-4">
              <div>
                <h2 className="text-xl font-bold text-white">{t.terminal.title}</h2>
                <p className="text-xs text-slate-400">{t.terminal.notice}</p>
              </div>

              <div className="p-4 rounded-xl bg-slate-950 border border-slate-800 font-mono text-xs flex flex-col h-[520px]">
                <div className="flex-1 overflow-y-auto space-y-3 pr-2">
                  {terminalHistory.map((item, idx) => (
                    <div key={idx} className="space-y-1">
                      <div className="flex items-center gap-2 text-cyan-400 font-semibold">
                        <span>astral-admin@srv:~#</span>
                        <span className="text-white">{item.cmd}</span>
                      </div>
                      <pre className="text-slate-300 text-xs whitespace-pre-wrap pl-4 font-mono">
                        {item.output}
                      </pre>
                    </div>
                  ))}
                </div>

                <form onSubmit={handleTerminalSubmit} className="mt-4 pt-3 border-t border-slate-800 flex items-center gap-2">
                  <span className="text-cyan-400 font-bold font-mono">#</span>
                  <input
                    type="text"
                    value={terminalInput}
                    onChange={(e) => setTerminalInput(e.target.value)}
                    placeholder={t.terminal.placeholder}
                    className="flex-1 px-3 py-1.5 bg-slate-900 border border-slate-800 rounded-lg text-white font-mono text-xs focus:outline-none focus:border-cyan-500"
                  />
                  <button
                    type="submit"
                    className="px-4 py-1.5 text-xs font-medium bg-cyan-600 hover:bg-cyan-500 text-white rounded-lg transition"
                  >
                    {t.actions.execute}
                  </button>
                </form>
              </div>
            </div>
          )}
        </main>
      </div>

      <DonationModal isOpen={isDonateOpen} onClose={() => setIsDonateOpen(false)} t={t} />
    </div>
  );
}
