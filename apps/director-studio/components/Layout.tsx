import React from 'react';
import { LayoutGrid, FolderOpen, Settings } from 'lucide-react';

interface LayoutProps {
  children: React.ReactNode;
  currentRoute: string;
  onNavigate: (route: string) => void;
}

export const Layout: React.FC<LayoutProps> = ({ children, currentRoute, onNavigate }) => {
  const navItems = [
    { id: '/', label: 'Studio Board', icon: LayoutGrid },
    { id: '/assets', label: 'Assets', icon: FolderOpen },
    { id: '/settings', label: 'Engine Config', icon: Settings },
  ];

  return (
    <div className="flex h-screen w-screen bg-black overflow-hidden font-sans">
      {/* Sidebar */}
      <aside className="w-64 bg-[#0a0a0a] border-r border-white/10 flex flex-col">
        <div className="p-6">
            <h1 className="text-white font-bold text-lg tracking-tight flex items-center gap-2">
                <div className="w-3 h-3 bg-[#7000FF] rounded-sm shadow-[0_0_10px_#7000FF]"></div>
                Director Studio
            </h1>
            <div className="mt-2 flex items-center gap-2 text-xs text-gray-500 font-mono">
                <span className="w-2 h-2 bg-[#00FF94] rounded-full animate-pulse"></span>
                v0.1.0 • Connected
            </div>
        </div>

        <nav className="flex-1 px-3 space-y-1">
            {navItems.map(item => (
                <button
                    key={item.id}
                    onClick={() => onNavigate(item.id)}
                    className={`w-full flex items-center gap-3 px-3 py-2 rounded-md text-sm font-medium transition-all ${
                        currentRoute === item.id 
                        ? 'bg-[#1a1a1a] text-white border border-white/10 shadow-sm' 
                        : 'text-gray-500 hover:text-gray-300 hover:bg-[#111]'
                    }`}
                >
                    <item.icon size={18} />
                    {item.label}
                </button>
            ))}
        </nav>

        <div className="p-4 border-t border-white/10">
            <div className="bg-[#141414] rounded p-3 border border-white/5 shadow-inner">
                <div className="text-[10px] text-gray-500 uppercase font-bold mb-2 tracking-wider">Engine Status</div>
                <div className="flex justify-between items-center text-xs text-gray-300 mb-1">
                    <span>Frame Time</span>
                    <span className="font-mono text-[#00FF94]">16.4ms</span>
                </div>
                <div className="flex justify-between items-center text-xs text-gray-300">
                    <span>Memory</span>
                    <span className="font-mono">1.2 GB</span>
                </div>
            </div>
        </div>
      </aside>

      {/* Main Content */}
      <main className="flex-1 relative overflow-hidden bg-black">
        {children}
      </main>
    </div>
  );
};