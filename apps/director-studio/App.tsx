import React, { useState, useEffect } from 'react';
import { Layout } from './components/Layout';
import { KanbanBoard } from './components/KanbanBoard';
import { TicketDetailModal } from './components/TicketDetailModal';
import { AssetLibrary } from './components/AssetLibrary';
import { SettingsPage } from './components/SettingsPage';
import { api } from './services/api';
import { Ticket, TicketStatus } from './types';

const App: React.FC = () => {
  const [currentRoute, setCurrentRoute] = useState('/');
  const [tickets, setTickets] = useState<Ticket[]>([]);
  const [selectedTicket, setSelectedTicket] = useState<Ticket | null>(null);

  useEffect(() => {
    if (currentRoute === '/') {
       loadData();
    }
  }, [currentRoute]);

  const loadData = async () => {
    const data = await api.getTickets();
    setTickets(data);
  };

  const handleVerifyTicket = async (id: string) => {
    await api.verifyTicket(id);
    console.log("Verified");
    loadData(); // Reload to get updated status/artifacts
  };

  const handleTicketMove = async (ticketId: string, newStatus: TicketStatus) => {
    // Optimistic UI update
    setTickets(prev => prev.map(t => 
      t.id === ticketId ? { ...t, status: newStatus } : t
    ));
    
    // API call
    await api.updateTicketStatus(ticketId, newStatus);
  };

  const renderContent = () => {
    switch (currentRoute) {
      case '/':
        return (
          <KanbanBoard 
            tickets={tickets} 
            onTicketClick={setSelectedTicket} 
            onTicketMove={handleTicketMove}
          />
        );
      case '/assets':
        return <AssetLibrary />;
      case '/settings':
        return <SettingsPage />;
      default:
        return <div className="p-8 text-white">404 Not Found</div>;
    }
  };

  return (
    <Layout currentRoute={currentRoute} onNavigate={setCurrentRoute}>
      {renderContent()}
      
      {selectedTicket && (
        <TicketDetailModal 
          ticket={selectedTicket} 
          onClose={() => setSelectedTicket(null)}
          onVerify={handleVerifyTicket}
        />
      )}
    </Layout>
  );
};

export default App;