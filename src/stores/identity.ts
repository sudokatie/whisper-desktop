/**
 * Identity store - manages user identity and lock state.
 */

import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import type { IdentityState } from '../types';

interface IdentityStore {
  state: IdentityState;
  isLoading: boolean;
  error: string | null;
  
  // Actions
  checkIdentity: () => Promise<void>;
  createIdentity: (passphrase: string) => Promise<void>;
  unlock: (passphrase: string) => Promise<void>;
  lock: () => Promise<void>;
  changePassphrase: (oldPassphrase: string, newPassphrase: string) => Promise<void>;
  getLinkQR: () => Promise<string>;
  clearError: () => void;
}

export const useIdentityStore = create<IdentityStore>((set) => ({
  state: { status: 'locked' },
  isLoading: false,
  error: null,

  checkIdentity: async () => {
    set({ isLoading: true, error: null });
    try {
      const result = await invoke<{ peer_id: string; public_key: string; is_locked: boolean; created_at: number } | null>('get_identity');
      if (result === null) {
        set({ state: { status: 'none' }, isLoading: false });
      } else if (result.is_locked) {
        set({ state: { status: 'locked' }, isLoading: false });
      } else {
        set({
          state: {
            status: 'unlocked',
            identity: {
              peerId: result.peer_id,
              publicKey: result.public_key,
              createdAt: result.created_at,
            },
          },
          isLoading: false,
        });
      }
    } catch (e) {
      set({ error: String(e), isLoading: false });
    }
  },

  createIdentity: async (passphrase: string) => {
    set({ isLoading: true, error: null });
    try {
      const result = await invoke<{ peer_id: string; public_key: string; created_at: number }>('create_identity', { passphrase });
      set({
        state: {
          status: 'unlocked',
          identity: {
            peerId: result.peer_id,
            publicKey: result.public_key,
            createdAt: result.created_at,
          },
        },
        isLoading: false,
      });
    } catch (e) {
      set({ error: String(e), isLoading: false });
    }
  },

  unlock: async (passphrase: string) => {
    set({ isLoading: true, error: null });
    try {
      const result = await invoke<{ peer_id: string; public_key: string; created_at: number }>('unlock', { passphrase });
      set({
        state: {
          status: 'unlocked',
          identity: {
            peerId: result.peer_id,
            publicKey: result.public_key,
            createdAt: result.created_at,
          },
        },
        isLoading: false,
      });
    } catch (e) {
      set({ error: String(e), isLoading: false });
    }
  },

  lock: async () => {
    set({ isLoading: true, error: null });
    try {
      await invoke('lock');
      set({ state: { status: 'locked' }, isLoading: false });
    } catch (e) {
      set({ error: String(e), isLoading: false });
    }
  },

  changePassphrase: async (oldPassphrase: string, newPassphrase: string) => {
    set({ isLoading: true, error: null });
    try {
      await invoke('change_passphrase', { oldPassphrase, newPassphrase });
      set({ isLoading: false });
    } catch (e) {
      set({ error: String(e), isLoading: false });
    }
  },

  getLinkQR: async () => {
    const result = await invoke<string>('get_link_qr');
    return result;
  },

  clearError: () => set({ error: null }),
}));
