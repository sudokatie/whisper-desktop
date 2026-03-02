/**
 * Onboarding page - create new identity.
 */

import { useState, FormEvent } from 'react';
import { useIdentityStore } from '../stores';
import { Button, Input } from '../components';

export function OnboardingPage() {
  const [passphrase, setPassphrase] = useState('');
  const [confirmPassphrase, setConfirmPassphrase] = useState('');
  const [localError, setLocalError] = useState<string | null>(null);
  const { createIdentity, isLoading, error, clearError } = useIdentityStore();

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault();
    setLocalError(null);

    if (passphrase.length < 8) {
      setLocalError('Passphrase must be at least 8 characters');
      return;
    }

    if (passphrase !== confirmPassphrase) {
      setLocalError('Passphrases do not match');
      return;
    }

    await createIdentity(passphrase);
  };

  const displayError = localError || error;

  return (
    <div className="min-h-screen bg-gray-950 flex items-center justify-center p-4">
      <div className="w-full max-w-md">
        <div className="text-center mb-8">
          <h1 className="text-3xl font-bold text-white mb-2">Welcome to Whisper</h1>
          <p className="text-gray-400">Create your encrypted identity</p>
        </div>

        <div className="bg-gray-900 rounded-lg p-6 mb-6">
          <h2 className="text-white font-medium mb-3">How it works</h2>
          <ul className="text-gray-400 text-sm space-y-2">
            <li>• Your identity is a cryptographic keypair generated on your device</li>
            <li>• Your passphrase encrypts this key locally - we never see it</li>
            <li>• Messages are end-to-end encrypted between you and your contacts</li>
            <li>• If you forget your passphrase, your data cannot be recovered</li>
          </ul>
        </div>

        <form onSubmit={handleSubmit} className="space-y-4">
          <Input
            type="password"
            label="Passphrase"
            placeholder="Choose a strong passphrase"
            value={passphrase}
            onChange={(e) => {
              setPassphrase(e.target.value);
              setLocalError(null);
              if (error) clearError();
            }}
            autoFocus
          />

          <Input
            type="password"
            label="Confirm Passphrase"
            placeholder="Enter passphrase again"
            value={confirmPassphrase}
            onChange={(e) => {
              setConfirmPassphrase(e.target.value);
              setLocalError(null);
            }}
            error={displayError || undefined}
          />

          <Button
            type="submit"
            variant="primary"
            size="lg"
            className="w-full"
            disabled={isLoading || !passphrase.trim() || !confirmPassphrase.trim()}
          >
            {isLoading ? 'Creating Identity...' : 'Create Identity'}
          </Button>
        </form>

        <p className="text-center text-gray-500 text-sm mt-6">
          Choose a passphrase you can remember. Write it down somewhere safe.
        </p>
      </div>
    </div>
  );
}
