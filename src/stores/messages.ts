/**
 * Messages store - manages conversations and messages.
 */

import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type { Message, Conversation } from '../types';

interface MessagesStore {
  conversations: Conversation[];
  currentPeerId: string | null;
  messages: Message[];
  unreadCount: number;
  isLoading: boolean;
  error: string | null;
  
  // Actions
  loadConversations: () => Promise<void>;
  loadMessages: (peerId: string, limit?: number, offset?: number) => Promise<void>;
  sendMessage: (peerId: string, content: string) => Promise<void>;
  markRead: (peerId: string) => Promise<void>;
  refreshUnreadCount: () => Promise<void>;
  setCurrentPeer: (peerId: string | null) => void;
  addMessage: (message: Message) => void;
  clearError: () => void;
  setupListeners: () => Promise<() => void>;
}

export const useMessagesStore = create<MessagesStore>((set, get) => ({
  conversations: [],
  currentPeerId: null,
  messages: [],
  unreadCount: 0,
  isLoading: false,
  error: null,

  loadConversations: async () => {
    set({ isLoading: true, error: null });
    try {
      const result = await invoke<Array<{
        peer_id: string;
        last_message: number[] | null;
        last_message_at: number | null;
        unread_count: number;
      }>>('get_conversations');
      
      const conversations: Conversation[] = result.map(c => ({
        peerId: c.peer_id,
        lastMessage: c.last_message ? new TextDecoder().decode(new Uint8Array(c.last_message)) : undefined,
        lastMessageAt: c.last_message_at ?? undefined,
        unreadCount: c.unread_count,
      }));
      
      set({ conversations, isLoading: false });
    } catch (e) {
      set({ error: String(e), isLoading: false });
    }
  },

  loadMessages: async (peerId: string, limit = 50, offset = 0) => {
    set({ isLoading: true, error: null, currentPeerId: peerId });
    try {
      const result = await invoke<Array<{
        id: string;
        peer_id: string;
        content: number[];
        timestamp: number;
        status: string;
        direction: string;
      }>>('get_messages', { peerId, limit, offset });
      
      const messages: Message[] = result.map(m => ({
        id: m.id,
        peerId: m.peer_id,
        content: new TextDecoder().decode(new Uint8Array(m.content)),
        timestamp: m.timestamp,
        status: m.status as Message['status'],
        direction: m.direction as Message['direction'],
      }));
      
      // Messages come in reverse chronological order, reverse for display
      set({ messages: messages.reverse(), isLoading: false });
    } catch (e) {
      set({ error: String(e), isLoading: false });
    }
  },

  sendMessage: async (peerId: string, content: string) => {
    try {
      const result = await invoke<{
        id: string;
        peer_id: string;
        content: number[];
        timestamp: number;
        status: string;
        direction: string;
      }>('send_message', { peerId, content });
      
      const message: Message = {
        id: result.id,
        peerId: result.peer_id,
        content: new TextDecoder().decode(new Uint8Array(result.content)),
        timestamp: result.timestamp,
        status: result.status as Message['status'],
        direction: result.direction as Message['direction'],
      };
      
      // Add to current messages if viewing this conversation
      const state = get();
      if (state.currentPeerId === peerId) {
        set({ messages: [...state.messages, message] });
      }
      
      // Refresh conversations
      get().loadConversations();
    } catch (e) {
      set({ error: String(e) });
    }
  },

  markRead: async (peerId: string) => {
    try {
      await invoke('mark_read', { peerId });
      // Refresh unread count
      get().refreshUnreadCount();
      get().loadConversations();
    } catch (e) {
      set({ error: String(e) });
    }
  },

  refreshUnreadCount: async () => {
    try {
      const count = await invoke<number>('get_unread_count');
      set({ unreadCount: count });
    } catch (e) {
      // Silently fail for unread count
    }
  },

  setCurrentPeer: (peerId: string | null) => {
    set({ currentPeerId: peerId });
    if (peerId) {
      get().loadMessages(peerId);
      get().markRead(peerId);
    } else {
      set({ messages: [] });
    }
  },

  addMessage: (message: Message) => {
    const state = get();
    if (state.currentPeerId === message.peerId) {
      set({ messages: [...state.messages, message] });
    }
    get().loadConversations();
    get().refreshUnreadCount();
  },

  clearError: () => set({ error: null }),

  setupListeners: async () => {
    const unlisten = await listen<Message>('message-received', (event) => {
      get().addMessage(event.payload);
    });
    return unlisten;
  },
}));
