/**
 * Main application component - handles routing based on identity state.
 */

import { useEffect, useState } from 'react';
import { useIdentityStore, useMessagesStore } from './stores';
import { Sidebar, TitleBar } from './components';
import {
  UnlockPage,
  OnboardingPage,
  ConversationsPage,
  ChatPage,
  ContactsPage,
  SettingsPage,
  LinkDevicePage,
} from './pages';

type MainView = 'conversations' | 'contacts' | 'settings';
type ActivePage = 'main' | 'chat' | 'link-device';

function App() {
  const { state: identityState, isLoading, checkIdentity } = useIdentityStore();
  const { setupListeners, loadConversations, refreshUnreadCount } = useMessagesStore();

  const [mainView, setMainView] = useState<MainView>('conversations');
  const [activePage, setActivePage] = useState<ActivePage>('main');
  const [selectedPeerId, setSelectedPeerId] = useState<string | null>(null);

  // Check identity on mount
  useEffect(() => {
    checkIdentity();
  }, [checkIdentity]);

  // Setup message listeners when unlocked
  useEffect(() => {
    if (identityState.status === 'unlocked') {
      loadConversations();
      refreshUnreadCount();
      
      let cleanup: (() => void) | undefined;
      setupListeners().then((unlisten) => {
        cleanup = unlisten;
      });
      
      return () => {
        cleanup?.();
      };
    }
  }, [identityState.status, setupListeners, loadConversations, refreshUnreadCount]);

  // Loading state
  if (isLoading && identityState.status === 'locked') {
    return (
      <div className="min-h-screen bg-gray-950 flex items-center justify-center">
        <div className="text-gray-500">Loading...</div>
      </div>
    );
  }

  // No identity - show onboarding
  if (identityState.status === 'none') {
    return <OnboardingPage />;
  }

  // Identity locked - show unlock
  if (identityState.status === 'locked') {
    return <UnlockPage />;
  }

  // Handle conversation selection
  const handleSelectConversation = (peerId: string) => {
    setSelectedPeerId(peerId);
    setActivePage('chat');
  };

  // Handle new message (opens contacts to select recipient)
  const handleNewMessage = () => {
    setMainView('contacts');
  };

  // Handle contact selection (opens chat)
  const handleSelectContact = (peerId: string) => {
    setSelectedPeerId(peerId);
    setActivePage('chat');
    setMainView('conversations'); // Switch back to conversations view
  };

  // Handle back from chat
  const handleBackFromChat = () => {
    setSelectedPeerId(null);
    setActivePage('main');
  };

  // Handle link device
  const handleLinkDevice = () => {
    setActivePage('link-device');
  };

  // Handle back from link device
  const handleBackFromLinkDevice = () => {
    setActivePage('main');
    setMainView('settings');
  };

  // Identity unlocked - show main app
  return (
    <div className="h-screen flex flex-col bg-gray-950">
      <TitleBar />
      
      <div className="flex-1 flex overflow-hidden">
        {/* Show sidebar on main page */}
        {activePage === 'main' && (
          <Sidebar
            currentView={mainView}
            onViewChange={setMainView}
            onConversationSelect={handleSelectConversation}
            selectedPeerId={selectedPeerId}
          />
        )}
        
        {/* Main content area */}
        <main className="flex-1 overflow-hidden">
          {activePage === 'chat' && selectedPeerId && (
            <ChatPage peerId={selectedPeerId} onBack={handleBackFromChat} />
          )}
          
          {activePage === 'link-device' && (
            <LinkDevicePage onBack={handleBackFromLinkDevice} />
          )}
          
          {activePage === 'main' && mainView === 'conversations' && (
            <ConversationsPage
              onSelectConversation={handleSelectConversation}
              onNewMessage={handleNewMessage}
            />
          )}
          
          {activePage === 'main' && mainView === 'contacts' && (
            <ContactsPage onSelectContact={handleSelectContact} />
          )}
          
          {activePage === 'main' && mainView === 'settings' && (
            <SettingsPage onLinkDevice={handleLinkDevice} />
          )}
        </main>
      </div>
    </div>
  );
}

export default App;
