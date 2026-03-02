/**
 * TitleBar component - window title and controls.
 */

import { useIdentityStore } from '../../stores';

interface TitleBarProps {
  title?: string;
  onLock?: () => void;
}

export function TitleBar({ title = 'Whisper', onLock }: TitleBarProps) {
  const { state: identityState, lock } = useIdentityStore();
  const isUnlocked = identityState.status === 'unlocked';

  const handleLock = async () => {
    await lock();
    onLock?.();
  };

  return (
    <header className="h-12 bg-gray-900 border-b border-gray-800 flex items-center justify-between px-4 draggable">
      <div className="flex items-center gap-2">
        <span className="text-white font-semibold">{title}</span>
      </div>
      
      <div className="flex items-center gap-2 no-drag">
        {isUnlocked && (
          <button
            onClick={handleLock}
            className="text-gray-400 hover:text-white px-2 py-1 rounded hover:bg-gray-800"
            title="Lock"
          >
            🔒
          </button>
        )}
      </div>
    </header>
  );
}
