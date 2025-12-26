import React, { useState } from 'react';
import { Ticket, Column, TicketStatus } from '../types';
import { COLUMNS } from '../constants';
import { KanbanCard } from './KanbanCard';

interface KanbanBoardProps {
  tickets: Ticket[];
  onTicketClick: (ticket: Ticket) => void;
  onTicketMove: (ticketId: string, newStatus: TicketStatus) => void;
}

export const KanbanBoard: React.FC<KanbanBoardProps> = ({ tickets, onTicketClick, onTicketMove }) => {
  const [draggedTicketId, setDraggedTicketId] = useState<string | null>(null);
  const [dragOverCol, setDragOverCol] = useState<string | null>(null);

  const getTicketsForColumn = (colId: string) => tickets.filter(t => t.status === colId);

  const handleDragStart = (e: React.DragEvent, ticketId: string) => {
    setDraggedTicketId(ticketId);
    e.dataTransfer.effectAllowed = "move";
    e.dataTransfer.setData("text/plain", ticketId);
    
    // Create a custom drag image if possible, but default is usually fine if card is styled.
    // We can just let the browser snapshot the element.
  };

  const handleDragEnd = () => {
    setDraggedTicketId(null);
    setDragOverCol(null);
  };

  const handleDragOver = (e: React.DragEvent, colId: string) => {
    e.preventDefault(); // Necessary to allow dropping
    if (dragOverCol !== colId) {
        setDragOverCol(colId);
    }
  };

  const handleDragLeave = (e: React.DragEvent) => {
    // Optional: refine this to avoid flickering when hovering over children
    // For now, we rely on handleDragOver to keep it set, and handleDrop/End to clear.
  };

  const handleDrop = (e: React.DragEvent, colId: string) => {
    e.preventDefault();
    const ticketId = e.dataTransfer.getData("text/plain");
    
    setDragOverCol(null);
    setDraggedTicketId(null);

    if (ticketId) {
        onTicketMove(ticketId, colId as TicketStatus);
    }
  };

  return (
    <div className="h-full p-8 flex gap-6 overflow-x-auto">
      {COLUMNS.map(col => {
        const isDragOver = dragOverCol === col.id;
        const colTickets = getTicketsForColumn(col.id);
        
        return (
          <div 
            key={col.id} 
            className={`w-80 flex-shrink-0 flex flex-col rounded-xl transition-colors duration-200 ${
                isDragOver ? 'bg-[#1a1a1a]/40 ring-2 ring-[#7000FF] ring-opacity-50' : ''
            }`}
            onDragOver={(e) => handleDragOver(e, col.id)}
            onDragLeave={handleDragLeave}
            onDrop={(e) => handleDrop(e, col.id)}
          >
            <header className="flex items-center justify-between mb-4 px-3 mt-2">
              <h3 className={`font-medium text-sm transition-colors ${isDragOver ? 'text-[#7000FF]' : 'text-gray-400'}`}>
                {col.label}
              </h3>
              <span className={`text-xs font-mono px-2 py-0.5 rounded transition-colors ${
                  isDragOver ? 'bg-[#7000FF] text-white' : 'text-gray-600 bg-[#1a1a1a]'
              }`}>
                {colTickets.length}
              </span>
            </header>
            
            <div className="flex-1 space-y-3 min-h-[200px] p-2">
              {colTickets.map(ticket => (
                <KanbanCard 
                  key={ticket.id} 
                  ticket={ticket} 
                  onClick={onTicketClick}
                  onDragStart={handleDragStart}
                  onDragEnd={handleDragEnd}
                  isDragging={draggedTicketId === ticket.id}
                />
              ))}
              
              {/* Drop Zone Visual Hint */}
              {isDragOver && (
                 <div className="h-24 rounded-lg border-2 border-dashed border-[#7000FF]/30 bg-[#7000FF]/5 flex items-center justify-center text-[#7000FF] text-xs font-medium uppercase tracking-widest animate-pulse">
                    Drop Here
                </div>
              )}
              
              {/* Default Empty State */}
              {!isDragOver && colTickets.length === 0 && (
                <div className="h-24 rounded-lg border border-dashed border-[#222] flex items-center justify-center text-[#222] text-xs font-medium uppercase tracking-widest select-none">
                    Empty
                </div>
              )}
            </div>
          </div>
        );
      })}
    </div>
  );
};