/**
 * Contacts page - view and manage contacts.
 */

import { useEffect, useState } from 'react';
import { useContactsStore } from '../stores';
import { Button, Input } from '../components';
import type { Contact, TrustLevel } from '../types';

interface ContactsPageProps {
  onSelectContact: (peerId: string) => void;
}

export function ContactsPage({ onSelectContact }: ContactsPageProps) {
  const { contacts, isLoading, error, loadContacts, addContact, updateAlias, deleteContact, clearError } = useContactsStore();
  const [showAddForm, setShowAddForm] = useState(false);
  const [newPeerId, setNewPeerId] = useState('');
  const [newAlias, setNewAlias] = useState('');
  const [editingContact, setEditingContact] = useState<string | null>(null);
  const [editAlias, setEditAlias] = useState('');

  useEffect(() => {
    loadContacts();
  }, [loadContacts]);

  const handleAddContact = async () => {
    if (!newPeerId.trim()) return;
    try {
      await addContact(newPeerId.trim(), newAlias.trim() || undefined);
      setNewPeerId('');
      setNewAlias('');
      setShowAddForm(false);
    } catch {
      // Error handled by store
    }
  };

  const handleUpdateAlias = async (peerId: string) => {
    await updateAlias(peerId, editAlias);
    setEditingContact(null);
  };

  const handleDeleteContact = async (contact: Contact) => {
    if (confirm(`Delete contact ${contact.alias || contact.peerId.slice(0, 8)}?`)) {
      await deleteContact(contact.peerId);
    }
  };

  const getTrustBadge = (level: TrustLevel) => {
    switch (level) {
      case 'verified':
        return <span className="text-green-400 text-xs">Verified</span>;
      case 'unverified':
        return <span className="text-yellow-400 text-xs">Unverified</span>;
      default:
        return <span className="text-gray-500 text-xs">Unknown</span>;
    }
  };

  return (
    <div className="flex flex-col h-full bg-gray-950">
      {/* Header */}
      <header className="flex items-center justify-between p-4 border-b border-gray-800">
        <h1 className="text-xl font-bold text-white">Contacts</h1>
        <Button
          variant="primary"
          size="sm"
          onClick={() => setShowAddForm(!showAddForm)}
        >
          {showAddForm ? 'Cancel' : 'Add Contact'}
        </Button>
      </header>

      {/* Add contact form */}
      {showAddForm && (
        <div className="p-4 border-b border-gray-800 bg-gray-900">
          <div className="space-y-3">
            <Input
              label="Peer ID"
              placeholder="Enter contact's peer ID"
              value={newPeerId}
              onChange={(e) => {
                setNewPeerId(e.target.value);
                if (error) clearError();
              }}
              error={error || undefined}
            />
            <Input
              label="Alias (optional)"
              placeholder="Give them a nickname"
              value={newAlias}
              onChange={(e) => setNewAlias(e.target.value)}
            />
            <Button
              variant="primary"
              onClick={handleAddContact}
              disabled={!newPeerId.trim()}
            >
              Add Contact
            </Button>
          </div>
        </div>
      )}

      {/* Contact list */}
      <div className="flex-1 overflow-y-auto">
        {isLoading ? (
          <div className="flex items-center justify-center h-full">
            <div className="text-gray-500">Loading contacts...</div>
          </div>
        ) : contacts.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-full text-center p-8">
            <div className="text-gray-500 mb-4">No contacts yet</div>
            <button
              onClick={() => setShowAddForm(true)}
              className="text-blue-400 hover:text-blue-300"
            >
              Add your first contact
            </button>
          </div>
        ) : (
          <ul>
            {contacts.map((contact) => (
              <li key={contact.peerId} className="border-b border-gray-800">
                <div className="p-4 hover:bg-gray-900">
                  {editingContact === contact.peerId ? (
                    <div className="flex items-center gap-2">
                      <Input
                        value={editAlias}
                        onChange={(e) => setEditAlias(e.target.value)}
                        placeholder="Enter alias"
                        className="flex-1"
                      />
                      <Button size="sm" onClick={() => handleUpdateAlias(contact.peerId)}>
                        Save
                      </Button>
                      <Button size="sm" variant="secondary" onClick={() => setEditingContact(null)}>
                        Cancel
                      </Button>
                    </div>
                  ) : (
                    <div className="flex items-center justify-between">
                      <button
                        onClick={() => onSelectContact(contact.peerId)}
                        className="flex-1 text-left"
                      >
                        <div className="flex items-center gap-2">
                          <span className="text-white font-medium">
                            {contact.alias || contact.peerId.slice(0, 8)}
                          </span>
                          {getTrustBadge(contact.trustLevel)}
                        </div>
                        <div className="text-gray-500 text-xs font-mono mt-1">
                          {contact.peerId}
                        </div>
                      </button>
                      <div className="flex items-center gap-2">
                        <button
                          onClick={() => {
                            setEditingContact(contact.peerId);
                            setEditAlias(contact.alias);
                          }}
                          className="text-gray-400 hover:text-white p-2"
                          aria-label="Edit"
                        >
                          <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z" />
                          </svg>
                        </button>
                        <button
                          onClick={() => handleDeleteContact(contact)}
                          className="text-gray-400 hover:text-red-400 p-2"
                          aria-label="Delete"
                        >
                          <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                          </svg>
                        </button>
                      </div>
                    </div>
                  )}
                </div>
              </li>
            ))}
          </ul>
        )}
      </div>
    </div>
  );
}
