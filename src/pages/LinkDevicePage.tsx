/**
 * Link device page - display QR code for device linking.
 */

import { useEffect, useState } from 'react';
import { useIdentityStore } from '../stores';
import { Button } from '../components';

interface LinkDevicePageProps {
  onBack: () => void;
}

export function LinkDevicePage({ onBack }: LinkDevicePageProps) {
  const { state: identityState, getLinkQR } = useIdentityStore();
  const [qrData, setQrData] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);

  const peerId = identityState.status === 'unlocked' ? identityState.identity.peerId : null;

  useEffect(() => {
    const loadQR = async () => {
      try {
        const data = await getLinkQR();
        setQrData(data);
      } catch (e) {
        setError(String(e));
      } finally {
        setIsLoading(false);
      }
    };
    loadQR();
  }, [getLinkQR]);

  const handleCopyPeerId = async () => {
    if (!peerId) return;
    try {
      await navigator.clipboard.writeText(peerId);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch {
      // Clipboard API not available
    }
  };

  return (
    <div className="flex flex-col h-full bg-gray-950">
      {/* Header */}
      <header className="flex items-center gap-3 p-4 border-b border-gray-800">
        <button
          onClick={onBack}
          className="text-gray-400 hover:text-white p-1"
          aria-label="Back"
        >
          <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 19l-7-7 7-7" />
          </svg>
        </button>
        <h1 className="text-xl font-bold text-white">Link Device</h1>
      </header>

      <div className="flex-1 flex flex-col items-center justify-center p-8">
        {isLoading ? (
          <div className="text-gray-500">Loading...</div>
        ) : error ? (
          <div className="text-center">
            <div className="text-red-400 mb-4">{error}</div>
            <Button onClick={onBack}>Go Back</Button>
          </div>
        ) : (
          <>
            <div className="text-center mb-6">
              <h2 className="text-xl text-white mb-2">Scan this code</h2>
              <p className="text-gray-400 text-sm">
                Open Whisper on your other device and scan this QR code to link
              </p>
            </div>

            {/* QR Code */}
            {qrData && (
              <div className="bg-white p-4 rounded-lg mb-6">
                <img
                  src={`data:image/svg+xml,${encodeURIComponent(qrData)}`}
                  alt="Link QR Code"
                  className="w-64 h-64"
                />
              </div>
            )}

            {/* Manual option */}
            <div className="text-center">
              <p className="text-gray-500 text-sm mb-2">Or share your Peer ID manually:</p>
              <div className="bg-gray-900 rounded-lg p-3 flex items-center gap-2">
                <code className="text-white text-xs font-mono break-all max-w-xs">
                  {peerId}
                </code>
                <button
                  onClick={handleCopyPeerId}
                  className="text-gray-400 hover:text-white p-1 shrink-0"
                  aria-label="Copy"
                >
                  {copied ? (
                    <svg className="w-5 h-5 text-green-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
                    </svg>
                  ) : (
                    <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 5H6a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2v-1M8 5a2 2 0 002 2h2a2 2 0 002-2M8 5a2 2 0 012-2h2a2 2 0 012 2m0 0h2a2 2 0 012 2v3m2 4H10m0 0l3-3m-3 3l3 3" />
                    </svg>
                  )}
                </button>
              </div>
            </div>

            <div className="mt-8 text-center">
              <Button variant="secondary" onClick={onBack}>
                Done
              </Button>
            </div>
          </>
        )}
      </div>
    </div>
  );
}
