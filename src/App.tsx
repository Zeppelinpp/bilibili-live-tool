import { useState, useEffect } from 'react';
import { AppProvider } from '@/context/AppContext';
import { useUI } from '@/context/AppContext';
import { useUser } from '@/context/AppContext';
import { loadSavedConfig } from '@/hooks/useTauri';
import Sidebar from '@/components/Sidebar';
import StreamPanel from '@/components/StreamPanel';
import DanmakuPanel from '@/components/DanmakuPanel';
import AccountPanel from '@/components/AccountPanel';
import SettingsPanel from '@/components/SettingsPanel';
import ConsolePanel from '@/components/ConsolePanel';
import { Sun, Moon, Terminal } from 'lucide-react';

function AppContent() {
  const [activeTab, setActiveTab] = useState('stream');
  const { isDark, setIsDark, consoleOpen, setConsoleOpen } = useUI();
  const { setUser } = useUser();

  useEffect(() => {
    loadSavedConfig()
      .then((u) => {
        if (u) setUser(u);
      })
      .catch(() => {});
  }, [setUser]);

  const renderPanel = () => {
    switch (activeTab) {
      case 'stream': return <StreamPanel />;
      case 'danmaku': return <DanmakuPanel />;
      case 'account': return <AccountPanel />;
      case 'settings': return <SettingsPanel />;
      default: return <StreamPanel />;
    }
  };

  return (
    <div className="flex h-screen bg-white text-stone-800 dark:bg-stone-950 dark:text-stone-200 overflow-hidden">
      <Sidebar activeTab={activeTab} onTabChange={setActiveTab} />
      <div className="flex-1 flex flex-col min-w-0">
        <div className="flex items-center justify-end px-4 h-10 gap-2">
          <button
            onClick={() => { setIsDark(!isDark); document.documentElement.classList.toggle('dark'); }}
            className="w-7 h-7 rounded-md flex items-center justify-center text-stone-400 hover:text-stone-600 dark:hover:text-stone-300 hover:bg-stone-100 dark:hover:bg-stone-900 transition"
          >
            {isDark ? <Sun size={14} /> : <Moon size={14} />}
          </button>
          <button
            onClick={() => setConsoleOpen(!consoleOpen)}
            className={`w-7 h-7 rounded-md flex items-center justify-center transition ${consoleOpen ? 'text-stone-800 dark:text-stone-200 bg-stone-200 dark:bg-stone-800' : 'text-stone-400 hover:text-stone-600 dark:hover:text-stone-300 hover:bg-stone-100 dark:hover:bg-stone-900'}`}
          >
            <Terminal size={14} />
          </button>
        </div>
        {renderPanel()}
        <ConsolePanel open={consoleOpen} />
      </div>
    </div>
  );
}

function App() {
  return (
    <AppProvider>
      <AppContent />
    </AppProvider>
  );
}

export default App;
