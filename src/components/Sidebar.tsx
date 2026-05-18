import { useUser } from '@/context/AppContext';
import { useLive } from '@/context/AppContext';
import { RadioTower, MessageSquare, User, Settings } from 'lucide-react';

interface SidebarProps {
  activeTab: string;
  onTabChange: (tab: string) => void;
}

const navItems = [
  { id: 'stream', label: '推流设置', icon: RadioTower },
  { id: 'danmaku', label: '弹幕监控', icon: MessageSquare },
  { id: 'account', label: '账户管理', icon: User },
];

export default function Sidebar({ activeTab, onTabChange }: SidebarProps) {
  const { user } = useUser();
  const { isLive } = useLive();

  return (
    <div className="w-52 bg-stone-50 dark:bg-stone-950 border-r border-stone-200 dark:border-stone-800 flex flex-col shrink-0">
      {/* User Card */}
      <div className="px-3 pb-3 mb-2 border-b border-stone-200 dark:border-stone-800">
        <div className="flex items-center gap-2.5 p-2.5 rounded-lg hover:bg-stone-100 dark:hover:bg-stone-900 transition cursor-pointer">
          <div className="w-8 h-8 rounded-full bg-stone-400 flex items-center justify-center text-white text-xs font-semibold">
            {user?.uname?.[0] ?? '?'}
          </div>
          <div className="flex-1 min-w-0">
            <div className="text-[13px] font-medium truncate">{user?.uname ?? '未登录'}</div>
            <div className="text-[11px] text-stone-400 truncate">
              {user ? `LV${user.level} · ${user.uid}` : '点击登录'}
            </div>
          </div>
        </div>
      </div>

      {/* Navigation */}
      <nav className="flex-1 px-2 space-y-0.5">
        {navItems.map((item) => {
          const isActive = activeTab === item.id;
          return (
            <button
              key={item.id}
              onClick={() => onTabChange(item.id)}
              className={`w-full flex items-center gap-3 px-3 py-2 rounded-lg text-[13px] font-medium transition ${
                isActive
                  ? 'bg-stone-200 dark:bg-stone-800 text-stone-900 dark:text-stone-100'
                  : 'text-stone-500 dark:text-stone-400 hover:bg-stone-100 dark:hover:bg-stone-900 hover:text-stone-900 dark:hover:text-stone-100'
              }`}
            >
              <item.icon size={16} />
              <span>{item.label}</span>
            </button>
          );
        })}

        <div className="pt-4 mt-4 border-t border-stone-200 dark:border-stone-800">
          <button
            onClick={() => onTabChange('settings')}
            className={`w-full flex items-center gap-3 px-3 py-2 rounded-lg text-[13px] font-medium transition ${
              activeTab === 'settings'
                ? 'bg-stone-200 dark:bg-stone-800 text-stone-900 dark:text-stone-100'
                : 'text-stone-500 dark:text-stone-400 hover:bg-stone-100 dark:hover:bg-stone-900 hover:text-stone-900 dark:hover:text-stone-100'
            }`}
          >
            <Settings size={16} />
            <span>设置</span>
          </button>
        </div>
      </nav>

      {/* Live Status */}
      <div className="px-3 pt-2 border-t border-stone-200 dark:border-stone-800">
        <div className="flex items-center gap-2 px-3 py-2">
          <span className={`w-1.5 h-1.5 rounded-full ${isLive ? 'bg-green-500 animate-pulse' : 'bg-stone-300'}`} />
          <span className="text-[11px] text-stone-400">{isLive ? '直播中' : '未开播'}</span>
        </div>
      </div>
    </div>
  );
}
