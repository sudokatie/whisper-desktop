/**
 * Unlock page - passphrase entry for locked identity.
 */

import { useState, FormEvent } from 'react';
import { useIdentityStore } from '../stores';
import { Button, Input } from '../components';

export function UnlockPage() {
  const [passphrase, setPassphrase] = useState('');
  const { unlock, isLoading, error, clearError } = useIdentityStore();

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault();
    if (!passphrase.trim()) return;
    await unlock(passphrase);
  };

  return (
    <div className="min-h-screen bg-gray-950 flex items-center justify-center p-4">
      <div className="w-full max-w-md">
        <div className="text-center mb-8">
          <h1 className="text-3xl font-bold text-white mb-2">Whisper</h1>
          <p className="text-gray-400">End-to-end encrypted messaging</p>
        </div>

        <form onSubmit={handleSubmit} className="space-y-6">
          <div>
            <Input
              type="password"
              label="Passphrase"
              placeholder="Enter your passphrase"
              value={passphrase}
              onChange={(e) => {
                setPassphrase(e.target.value);
                if (error) clearError();
              }}
              error={error || undefined}
              autoFocus
            />
          </div>

          <Button
            type="submit"
            variant="primary"
            size="lg"
            className="w-full"
            disabled={isLoading || !passphrase.trim()}
          >
            {isLoading ? 'Unlocking...' : 'Unlock'}
          </Button>
        </form>

        <p className="text-center text-gray-500 text-sm mt-8">
          Your messages are encrypted locally with your passphrase.
        </p>
      </div>
    </div>
  );
}
