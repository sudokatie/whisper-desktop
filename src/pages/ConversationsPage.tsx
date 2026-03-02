/**
 * Conversations page - main view showing conversation list.
 */

import { useEffect } from 'react';
import { useMessagesStore, useContactsStore } from '../stores';

interface ConversationsPageProps {
  onSelectConversation: (peerId: string) => void;
  onNewMessage: () => void;
}

export function ConversationsPage({ onSelectConversation, onNewMessage }: ConversationsPageProps) {
  const { conversations, isLoading, loadConversations } = useMessagesStore();
  const { contacts, loadContacts } = useContactsStore();

  useEffect(() => {
    loadConversations();
    loadContacts();
  }, [loadConversations, loadContacts]);

  const getDisplayName = (peerId: string) => {
    const contact = contacts.find(c => c.peerId === peerId);
    return contact?.alias || `${peerId.slice(0, 8)}...`;
  };

  const formatTime = (timestamp?: number) => {
    if (!timestamp) return '';
    const date = new Date(timestamp * 1000);
    const now = new Date();
    const diff = now.getTime() - date.getTime();
    const days = Math.floor(diff / (1000 * 60 * 60 * 24));

    if (days === 0) {
      return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
    } else if (days === 1) {
      return 'Yesterday';
    } else if (days < 7) {
      return date.toLocaleDateString([], { weekday: 'short' });
    } else {
      return date.toLocaleDateString([], { month: 'short', day: 'numeric' });
    }
  };

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-gray-500">Loading conversations...</div>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full bg-gray-950">
      {/* Header */}
      <header className="flex items-center justify-between p-4 border-b border-gray-800">
        <h1 className="text-xl font-bold text-white">Messages</h1>
        <button
          onClick={onNewMessage}
          className="bg-blue-600 hover:bg-blue-700 text-white px-4 py-2 rounded-lg text-sm font-medium"
        >
          New Message
        </button>
      </header>

      {/* Conversation list */}
      <div className="flex-1 overflow-y-auto">
        {conversations.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-full text-center p-8">
            <div className="text-gray-500 mb-4">No conversations yet</div>
            <button
              onClick={onNewMessage}
              className="text-blue-400 hover:text-blue-300"
            >
              Start a new message
            </button>
          </div>
        ) : (
          <ul>
            {conversations.map((conv) => (
              <li key={conv.peerId}>
                <button
                  onClick={() => onSelectConversation(conv.peerId)}
                  className="w-full text-left p-4 border-b border-gray-800 hover:bg-gray-900 transition-colors"
                >
                  <div className="flex items-center justify-between mb-1">
                    <span className="text-white font-medium">
                      {getDisplayName(conv.peerId)}
                    </span>
                    <span className="text-gray-500 text-xs">
                      {formatTime(conv.lastMessageAt)}
                    </span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-gray-400 text-sm truncate max-w-[80%]">
                      {conv.lastMessage || 'No messages'}
                    </span>
                    {conv.unreadCount > 0 && (
                      <span className="bg-blue-600 text-white text-xs px-2 py-0.5 rounded-full">
                        {conv.unreadCount}
                      </span>
                    )}
                  </div>
                </button>
              </li>
            ))}
          </ul>
        )}
      </div>
    </div>
  );
}
