/**
 * Chat page - message thread with a single contact.
 */

import { useEffect } from 'react';
import { useMessagesStore, useContactsStore } from '../stores';
import { MessageList, MessageInput } from '../components';

interface ChatPageProps {
  peerId: string;
  onBack: () => void;
}

export function ChatPage({ peerId, onBack }: ChatPageProps) {
  const { messages, isLoading, sendMessage, setCurrentPeer } = useMessagesStore();
  const { getContact, loadContacts } = useContactsStore();

  useEffect(() => {
    loadContacts();
    setCurrentPeer(peerId);
    return () => setCurrentPeer(null);
  }, [peerId, setCurrentPeer, loadContacts]);

  const contact = getContact(peerId);
  const displayName = contact?.alias || `${peerId.slice(0, 8)}...`;

  const handleSend = async (content: string) => {
    await sendMessage(peerId, content);
  };

  return (
    <div className="flex flex-col h-full bg-gray-950">
      {/* Header */}
      <header className="flex items-center gap-3 p-4 border-b border-gray-800">
        <button
          onClick={onBack}
          className="text-gray-400 hover:text-white p-1"
          aria-label="Back"
        >
          <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 19l-7-7 7-7" />
          </svg>
        </button>
        <div>
          <h1 className="text-white font-medium">{displayName}</h1>
          <p className="text-gray-500 text-xs font-mono">{peerId}</p>
        </div>
      </header>

      {/* Messages */}
      <div className="flex-1 overflow-hidden">
        {isLoading ? (
          <div className="flex items-center justify-center h-full">
            <div className="text-gray-500">Loading messages...</div>
          </div>
        ) : (
          <MessageList messages={messages} />
        )}
      </div>

      {/* Input */}
      <div className="border-t border-gray-800">
        <MessageInput onSend={handleSend} placeholder={`Message ${displayName}`} />
      </div>
    </div>
  );
}
