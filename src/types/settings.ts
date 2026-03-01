/**
 * Settings types for Whisper Desktop.
 */

/** Application theme. */
export type Theme = 'light' | 'dark' | 'system';

/** Application settings. */
export interface Settings {
  // General
  /** Color theme */
  theme: Theme;
  /** Start minimized to tray */
  startMinimized: boolean;
  /** Launch on system startup */
  launchOnStartup: boolean;
  /** Close to tray instead of quit */
  closeToTray: boolean;

  // Notifications
  /** Enable desktop notifications */
  notificationsEnabled: boolean;
  /** Show message preview in notifications */
  showPreview: boolean;
  /** Play notification sound */
  soundEnabled: boolean;

  // Privacy
  /** Auto-lock timeout in minutes (0 = disabled) */
  autoLockMinutes: number;
  /** Clear data on exit */
  clearOnExit: boolean;

  // Network
  /** Relay server URL */
  relayUrl: string;
}

/** Default settings. */
export const DEFAULT_SETTINGS: Settings = {
  theme: 'system',
  startMinimized: false,
  launchOnStartup: false,
  closeToTray: true,
  notificationsEnabled: true,
  showPreview: true,
  soundEnabled: true,
  autoLockMinutes: 5,
  clearOnExit: false,
  relayUrl: 'wss://relay.whisper.chat',
};
