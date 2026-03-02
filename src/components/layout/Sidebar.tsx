/**
 * Sidebar component - navigation and conversation list.
 */

import { useMessagesStore, useIdentityStore } from '../../stores';

interface SidebarProps {
  currentView: 'conversations' | 'contacts' | 'settings';
  onViewChange: (view: 'conversations' | 'contacts' | 'settings') => void;
  onConversationSelect: (peerId: string) => void;
  selectedPeerId: string | null;
}

export function Sidebar({ currentView, onViewChange, onConversationSelect, selectedPeerId }: SidebarProps) {
  const { conversations, unreadCount } = useMessagesStore();
  const { state: identityState } = useIdentityStore();

  const peerId = identityState.status === 'unlocked' ? identityState.identity.peerId : null;

  return (
    <aside className="w-64 bg-gray-900 border-r border-gray-800 flex flex-col h-full">
      {/* User info */}
      <div className="p-4 border-b border-gray-800">
        <div className="text-sm text-gray-400">Signed in as</div>
        <div className="text-white font-mono text-xs truncate">
          {peerId ? `${peerId.slice(0, 8)}...` : 'Locked'}
        </div>
      </div>

      {/* Navigation */}
      <nav className="p-2 border-b border-gray-800">
        <button
          onClick={() => onViewChange('conversations')}
          className={`w-full text-left px-3 py-2 rounded flex items-center gap-2 ${
            currentView === 'conversations' ? 'bg-gray-800 text-white' : 'text-gray-400 hover:bg-gray-800'
          }`}
        >
          <span>💬</span>
          <span>Messages</span>
          {unreadCount > 0 && (
            <span className="ml-auto bg-blue-600 text-white text-xs px-2 py-0.5 rounded-full">
              {unreadCount}
            </span>
          )}
        </button>
        <button
          onClick={() => onViewChange('contacts')}
          className={`w-full text-left px-3 py-2 rounded flex items-center gap-2 ${
            currentView === 'contacts' ? 'bg-gray-800 text-white' : 'text-gray-400 hover:bg-gray-800'
          }`}
        >
          <span>👥</span>
          <span>Contacts</span>
        </button>
        <button
          onClick={() => onViewChange('settings')}
          className={`w-full text-left px-3 py-2 rounded flex items-center gap-2 ${
            currentView === 'settings' ? 'bg-gray-800 text-white' : 'text-gray-400 hover:bg-gray-800'
          }`}
        >
          <span>⚙️</span>
          <span>Settings</span>
        </button>
      </nav>

      {/* Conversation list */}
      {currentView === 'conversations' && (
        <div className="flex-1 overflow-y-auto">
          {conversations.length === 0 ? (
            <div className="p-4 text-gray-500 text-sm">No conversations yet</div>
          ) : (
            conversations.map((conv) => (
              <button
                key={conv.peerId}
                onClick={() => onConversationSelect(conv.peerId)}
                className={`w-full text-left p-3 border-b border-gray-800 hover:bg-gray-800 ${
                  selectedPeerId === conv.peerId ? 'bg-gray-800' : ''
                }`}
              >
                <div className="flex items-center justify-between">
                  <span className="text-white font-medium truncate">
                    {conv.alias || conv.peerId.slice(0, 8)}
                  </span>
                  {conv.unreadCount > 0 && (
                    <span className="bg-blue-600 text-white text-xs px-2 py-0.5 rounded-full">
                      {conv.unreadCount}
                    </span>
                  )}
                </div>
                {conv.lastMessage && (
                  <div className="text-gray-400 text-sm truncate mt-1">
                    {conv.lastMessage}
                  </div>
                )}
              </button>
            ))
          )}
        </div>
      )}
    </aside>
  );
}
