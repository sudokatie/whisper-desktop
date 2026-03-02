/**
 * MessageBubble component - single message display.
 */

import type { Message } from '../../types';

interface MessageBubbleProps {
  message: Message;
}

export function MessageBubble({ message }: MessageBubbleProps) {
  const isOutgoing = message.direction === 'outgoing';
  const time = new Date(message.timestamp * 1000).toLocaleTimeString([], {
    hour: '2-digit',
    minute: '2-digit',
  });

  const statusIcon = () => {
    switch (message.status) {
      case 'pending':
        return '⏳';
      case 'sent':
        return '✓';
      case 'delivered':
        return '✓✓';
      case 'read':
        return '✓✓';
      case 'failed':
        return '❌';
      default:
        return '';
    }
  };

  return (
    <div className={`flex ${isOutgoing ? 'justify-end' : 'justify-start'} mb-2`}>
      <div
        className={`max-w-[70%] rounded-lg px-4 py-2 ${
          isOutgoing
            ? 'bg-blue-600 text-white rounded-br-none'
            : 'bg-gray-700 text-white rounded-bl-none'
        }`}
      >
        <div className="break-words">{message.content}</div>
        <div className={`text-xs mt-1 flex items-center gap-1 ${
          isOutgoing ? 'text-blue-200' : 'text-gray-400'
        }`}>
          <span>{time}</span>
          {isOutgoing && <span>{statusIcon()}</span>}
        </div>
      </div>
    </div>
  );
}
