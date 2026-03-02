/**
 * Contacts store - manages contact list.
 */

import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import type { Contact, TrustLevel } from '../types';

interface ContactsStore {
  contacts: Contact[];
  isLoading: boolean;
  error: string | null;
  
  // Actions
  loadContacts: () => Promise<void>;
  addContact: (peerId: string, alias?: string, publicKey?: number[]) => Promise<Contact>;
  addContactFromQR: (qrData: string, alias?: string) => Promise<Contact>;
  updateAlias: (peerId: string, alias: string) => Promise<void>;
  updateTrust: (peerId: string, trustLevel: TrustLevel) => Promise<void>;
  deleteContact: (peerId: string) => Promise<void>;
  getContact: (peerId: string) => Contact | undefined;
  clearError: () => void;
}

export const useContactsStore = create<ContactsStore>((set, get) => ({
  contacts: [],
  isLoading: false,
  error: null,

  loadContacts: async () => {
    set({ isLoading: true, error: null });
    try {
      const result = await invoke<Array<{
        peer_id: string;
        alias: string;
        public_key: number[];
        trust_level: string;
        created_at: number;
        updated_at: number;
      }>>('get_contacts');
      
      const contacts: Contact[] = result.map(c => ({
        peerId: c.peer_id,
        alias: c.alias,
        publicKey: Array.from(c.public_key).map(b => b.toString(16).padStart(2, '0')).join(''),
        trustLevel: c.trust_level as TrustLevel,
        createdAt: c.created_at,
        updatedAt: c.updated_at,
      }));
      
      set({ contacts, isLoading: false });
    } catch (e) {
      set({ error: String(e), isLoading: false });
    }
  },

  addContact: async (peerId: string, alias?: string, publicKey?: number[]) => {
    set({ isLoading: true, error: null });
    try {
      const result = await invoke<{
        peer_id: string;
        alias: string;
        public_key: number[];
        trust_level: string;
        created_at: number;
        updated_at: number;
      }>('add_contact', { peerId, alias, publicKey: publicKey ?? [] });
      
      const contact: Contact = {
        peerId: result.peer_id,
        alias: result.alias,
        publicKey: Array.from(result.public_key).map(b => b.toString(16).padStart(2, '0')).join(''),
        trustLevel: result.trust_level as TrustLevel,
        createdAt: result.created_at,
        updatedAt: result.updated_at,
      };
      
      set(state => ({
        contacts: [...state.contacts, contact],
        isLoading: false,
      }));
      
      return contact;
    } catch (e) {
      set({ error: String(e), isLoading: false });
      throw e;
    }
  },

  addContactFromQR: async (qrData: string, alias?: string) => {
    set({ isLoading: true, error: null });
    try {
      const result = await invoke<{
        peer_id: string;
        alias: string;
        public_key: number[];
        trust_level: string;
        created_at: number;
        updated_at: number;
      }>('add_contact_from_qr', { qrData, alias });
      
      const contact: Contact = {
        peerId: result.peer_id,
        alias: result.alias,
        publicKey: Array.from(result.public_key).map(b => b.toString(16).padStart(2, '0')).join(''),
        trustLevel: result.trust_level as TrustLevel,
        createdAt: result.created_at,
        updatedAt: result.updated_at,
      };
      
      set(state => ({
        contacts: [...state.contacts, contact],
        isLoading: false,
      }));
      
      return contact;
    } catch (e) {
      set({ error: String(e), isLoading: false });
      throw e;
    }
  },

  updateAlias: async (peerId: string, alias: string) => {
    try {
      await invoke('update_contact_alias', { peerId, alias });
      set(state => ({
        contacts: state.contacts.map(c =>
          c.peerId === peerId ? { ...c, alias, updatedAt: Date.now() / 1000 } : c
        ),
      }));
    } catch (e) {
      set({ error: String(e) });
    }
  },

  updateTrust: async (peerId: string, trustLevel: TrustLevel) => {
    try {
      await invoke('update_contact_trust', { peerId, trustLevel });
      set(state => ({
        contacts: state.contacts.map(c =>
          c.peerId === peerId ? { ...c, trustLevel, updatedAt: Date.now() / 1000 } : c
        ),
      }));
    } catch (e) {
      set({ error: String(e) });
    }
  },

  deleteContact: async (peerId: string) => {
    try {
      await invoke('delete_contact', { peerId });
      set(state => ({
        contacts: state.contacts.filter(c => c.peerId !== peerId),
      }));
    } catch (e) {
      set({ error: String(e) });
    }
  },

  getContact: (peerId: string) => {
    return get().contacts.find(c => c.peerId === peerId);
  },

  clearError: () => set({ error: null }),
}));
