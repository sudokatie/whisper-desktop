/**
 * Settings page - application configuration.
 */

import { useEffect, useState } from 'react';
import { useSettingsStore, useIdentityStore } from '../stores';
import { Button, Input } from '../components';
import type { Theme } from '../types';

interface SettingsPageProps {
  onLinkDevice: () => void;
}

export function SettingsPage({ onLinkDevice }: SettingsPageProps) {
  const { settings, isLoading, loadSettings, setSetting } = useSettingsStore();
  const { state: identityState, lock, changePassphrase } = useIdentityStore();

  const [showChangePassphrase, setShowChangePassphrase] = useState(false);
  const [oldPassphrase, setOldPassphrase] = useState('');
  const [newPassphrase, setNewPassphrase] = useState('');
  const [confirmPassphrase, setConfirmPassphrase] = useState('');
  const [passphraseError, setPassphraseError] = useState<string | null>(null);
  const [passphraseSuccess, setPassphraseSuccess] = useState(false);

  useEffect(() => {
    loadSettings();
  }, [loadSettings]);

  const peerId = identityState.status === 'unlocked' ? identityState.identity.peerId : null;

  const handleChangePassphrase = async () => {
    setPassphraseError(null);

    if (newPassphrase.length < 8) {
      setPassphraseError('New passphrase must be at least 8 characters');
      return;
    }

    if (newPassphrase !== confirmPassphrase) {
      setPassphraseError('Passphrases do not match');
      return;
    }

    try {
      await changePassphrase(oldPassphrase, newPassphrase);
      setShowChangePassphrase(false);
      setOldPassphrase('');
      setNewPassphrase('');
      setConfirmPassphrase('');
      setPassphraseSuccess(true);
      setTimeout(() => setPassphraseSuccess(false), 3000);
    } catch (e) {
      setPassphraseError(String(e));
    }
  };

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-gray-500">Loading settings...</div>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full bg-gray-950 overflow-y-auto">
      {/* Header */}
      <header className="p-4 border-b border-gray-800">
        <h1 className="text-xl font-bold text-white">Settings</h1>
      </header>

      <div className="p-4 space-y-6">
        {/* Account section */}
        <section>
          <h2 className="text-lg font-medium text-white mb-4">Account</h2>
          <div className="bg-gray-900 rounded-lg p-4 space-y-4">
            <div>
              <label className="text-gray-400 text-sm">Your Peer ID</label>
              <div className="text-white font-mono text-sm mt-1 break-all">
                {peerId || 'Locked'}
              </div>
            </div>
            <div className="flex gap-2">
              <Button variant="secondary" onClick={onLinkDevice}>
                Link Device
              </Button>
              <Button variant="secondary" onClick={lock}>
                Lock Now
              </Button>
            </div>

            {passphraseSuccess && (
              <div className="text-green-400 text-sm">Passphrase changed successfully</div>
            )}

            {showChangePassphrase ? (
              <div className="space-y-3 pt-2">
                <Input
                  type="password"
                  label="Current Passphrase"
                  value={oldPassphrase}
                  onChange={(e) => setOldPassphrase(e.target.value)}
                />
                <Input
                  type="password"
                  label="New Passphrase"
                  value={newPassphrase}
                  onChange={(e) => setNewPassphrase(e.target.value)}
                />
                <Input
                  type="password"
                  label="Confirm New Passphrase"
                  value={confirmPassphrase}
                  onChange={(e) => setConfirmPassphrase(e.target.value)}
                  error={passphraseError || undefined}
                />
                <div className="flex gap-2">
                  <Button onClick={handleChangePassphrase}>Save</Button>
                  <Button variant="secondary" onClick={() => setShowChangePassphrase(false)}>
                    Cancel
                  </Button>
                </div>
              </div>
            ) : (
              <Button variant="secondary" onClick={() => setShowChangePassphrase(true)}>
                Change Passphrase
              </Button>
            )}
          </div>
        </section>

        {/* Appearance section */}
        <section>
          <h2 className="text-lg font-medium text-white mb-4">Appearance</h2>
          <div className="bg-gray-900 rounded-lg p-4">
            <label className="text-gray-400 text-sm">Theme</label>
            <div className="flex gap-2 mt-2">
              {(['light', 'dark', 'system'] as Theme[]).map((theme) => (
                <button
                  key={theme}
                  onClick={() => setSetting('theme', theme)}
                  className={`px-4 py-2 rounded-lg text-sm capitalize ${
                    settings.theme === theme
                      ? 'bg-blue-600 text-white'
                      : 'bg-gray-800 text-gray-400 hover:bg-gray-700'
                  }`}
                >
                  {theme}
                </button>
              ))}
            </div>
          </div>
        </section>

        {/* Behavior section */}
        <section>
          <h2 className="text-lg font-medium text-white mb-4">Behavior</h2>
          <div className="bg-gray-900 rounded-lg divide-y divide-gray-800">
            <SettingToggle
              label="Start minimized"
              description="Start in the system tray"
              checked={settings.startMinimized}
              onChange={(v) => setSetting('startMinimized', v)}
            />
            <SettingToggle
              label="Launch on startup"
              description="Open when you log in"
              checked={settings.launchOnStartup}
              onChange={(v) => setSetting('launchOnStartup', v)}
            />
            <SettingToggle
              label="Close to tray"
              description="Keep running when window is closed"
              checked={settings.closeToTray}
              onChange={(v) => setSetting('closeToTray', v)}
            />
          </div>
        </section>

        {/* Notifications section */}
        <section>
          <h2 className="text-lg font-medium text-white mb-4">Notifications</h2>
          <div className="bg-gray-900 rounded-lg divide-y divide-gray-800">
            <SettingToggle
              label="Enable notifications"
              description="Show desktop notifications"
              checked={settings.notificationsEnabled}
              onChange={(v) => setSetting('notificationsEnabled', v)}
            />
            <SettingToggle
              label="Show message preview"
              description="Display message content in notifications"
              checked={settings.showPreview}
              onChange={(v) => setSetting('showPreview', v)}
            />
            <SettingToggle
              label="Notification sound"
              description="Play a sound for new messages"
              checked={settings.soundEnabled}
              onChange={(v) => setSetting('soundEnabled', v)}
            />
          </div>
        </section>

        {/* Privacy section */}
        <section>
          <h2 className="text-lg font-medium text-white mb-4">Privacy</h2>
          <div className="bg-gray-900 rounded-lg divide-y divide-gray-800">
            <div className="p-4">
              <label className="text-white text-sm">Auto-lock timeout</label>
              <p className="text-gray-500 text-xs mt-1">Lock after inactivity (0 = disabled)</p>
              <div className="flex items-center gap-2 mt-2">
                <input
                  type="number"
                  min={0}
                  max={60}
                  value={settings.autoLockMinutes}
                  onChange={(e) => setSetting('autoLockMinutes', parseInt(e.target.value) || 0)}
                  className="w-20 bg-gray-800 border border-gray-700 rounded px-3 py-2 text-white text-sm"
                />
                <span className="text-gray-400 text-sm">minutes</span>
              </div>
            </div>
            <SettingToggle
              label="Clear data on exit"
              description="Delete all data when app closes"
              checked={settings.clearOnExit}
              onChange={(v) => setSetting('clearOnExit', v)}
            />
          </div>
        </section>

        {/* Network section */}
        <section>
          <h2 className="text-lg font-medium text-white mb-4">Network</h2>
          <div className="bg-gray-900 rounded-lg p-4">
            <Input
              label="Relay Server"
              value={settings.relayUrl}
              onChange={(e) => setSetting('relayUrl', e.target.value)}
              placeholder="wss://relay.example.com"
            />
          </div>
        </section>
      </div>
    </div>
  );
}

interface SettingToggleProps {
  label: string;
  description: string;
  checked: boolean;
  onChange: (value: boolean) => void;
}

function SettingToggle({ label, description, checked, onChange }: SettingToggleProps) {
  return (
    <div className="flex items-center justify-between p-4">
      <div>
        <div className="text-white text-sm">{label}</div>
        <div className="text-gray-500 text-xs mt-0.5">{description}</div>
      </div>
      <button
        role="switch"
        aria-checked={checked}
        onClick={() => onChange(!checked)}
        className={`relative w-11 h-6 rounded-full transition-colors ${
          checked ? 'bg-blue-600' : 'bg-gray-700'
        }`}
      >
        <span
          className={`absolute top-0.5 w-5 h-5 bg-white rounded-full transition-transform ${
            checked ? 'left-5' : 'left-0.5'
          }`}
        />
      </button>
    </div>
  );
}
