/**
 * Settings store - manages application settings.
 */

import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import type { Settings, Theme } from '../types';
import { DEFAULT_SETTINGS } from '../types';

interface SettingsStore {
  settings: Settings;
  isLoading: boolean;
  error: string | null;
  
  // Actions
  loadSettings: () => Promise<void>;
  updateSettings: (settings: Partial<Settings>) => Promise<void>;
  setSetting: <K extends keyof Settings>(key: K, value: Settings[K]) => Promise<void>;
  resetSettings: () => Promise<void>;
  clearError: () => void;
}

// Convert backend settings format to frontend
function parseSettings(raw: Record<string, string>): Settings {
  return {
    theme: (raw.theme as Theme) || DEFAULT_SETTINGS.theme,
    startMinimized: raw.start_minimized === 'true',
    launchOnStartup: raw.launch_on_startup === 'true',
    closeToTray: raw.close_to_tray !== 'false', // default true
    notificationsEnabled: raw.notifications_enabled !== 'false', // default true
    showPreview: raw.show_preview !== 'false', // default true
    soundEnabled: raw.sound_enabled !== 'false', // default true
    autoLockMinutes: parseInt(raw.auto_lock_minutes || '5', 10),
    clearOnExit: raw.clear_on_exit === 'true',
    relayUrl: raw.relay_url || DEFAULT_SETTINGS.relayUrl,
  };
}

// Convert frontend settings to backend format
function serializeSettings(settings: Settings): Record<string, string> {
  return {
    theme: settings.theme,
    start_minimized: String(settings.startMinimized),
    launch_on_startup: String(settings.launchOnStartup),
    close_to_tray: String(settings.closeToTray),
    notifications_enabled: String(settings.notificationsEnabled),
    show_preview: String(settings.showPreview),
    sound_enabled: String(settings.soundEnabled),
    auto_lock_minutes: String(settings.autoLockMinutes),
    clear_on_exit: String(settings.clearOnExit),
    relay_url: settings.relayUrl,
  };
}

export const useSettingsStore = create<SettingsStore>((set, get) => ({
  settings: DEFAULT_SETTINGS,
  isLoading: false,
  error: null,

  loadSettings: async () => {
    set({ isLoading: true, error: null });
    try {
      const result = await invoke<Record<string, string>>('get_settings');
      const settings = parseSettings(result);
      set({ settings, isLoading: false });
    } catch (e) {
      set({ error: String(e), isLoading: false });
    }
  },

  updateSettings: async (partial: Partial<Settings>) => {
    const current = get().settings;
    const updated = { ...current, ...partial };
    
    set({ isLoading: true, error: null });
    try {
      const serialized = serializeSettings(updated);
      await invoke('update_settings', { settings: serialized });
      set({ settings: updated, isLoading: false });
    } catch (e) {
      set({ error: String(e), isLoading: false });
    }
  },

  setSetting: async <K extends keyof Settings>(key: K, value: Settings[K]) => {
    const backendKey = key.replace(/[A-Z]/g, m => `_${m.toLowerCase()}`);
    
    set({ isLoading: true, error: null });
    try {
      await invoke('set_setting', { key: backendKey, value: String(value) });
      set(state => ({
        settings: { ...state.settings, [key]: value },
        isLoading: false,
      }));
    } catch (e) {
      set({ error: String(e), isLoading: false });
    }
  },

  resetSettings: async () => {
    await get().updateSettings(DEFAULT_SETTINGS);
  },

  clearError: () => set({ error: null }),
}));
