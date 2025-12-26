import React, { useMemo } from 'react';
import { Ticket } from '../types';
import { Bot, AlertCircle, CheckCircle } from 'lucide-react';

interface KanbanCardProps {
  ticket: Ticket;
  onClick: (ticket: Ticket) => void;
  onDragStart?: (e: React.DragEvent, ticketId: string) => void;
  onDragEnd?: () => void;
  isDragging?: boolean;
}

export const KanbanCard: React.FC<KanbanCardProps> = ({ 
    ticket, 
    onClick, 
    onDragStart, 
    onDragEnd, 
    isDragging 
}) => {
  const isHighPriority = ticket.priority === 'high';
  // Threshold for "Slow" is 16ms (60fps budget)
  const isSlow = ticket.metrics && ticket.metrics.render_time_ms > 16.6; 
  
  // Generate fake sparkline data based on the current render time
  const sparklinePoints = useMemo(() => {
    if (!ticket.metrics) return "";
    const base = ticket.metrics.render_time_ms;
    const points = [];
    for (let i = 0; i < 10; i++) {
        // Random variance +/- 2ms
        const val = base + (Math.random() * 4 - 2); 
        // Scale to 20px height. If base is 16, typically it's in middle. 
        // Let's assume range 0-40ms. 
        // y = 20 - (val / 40 * 20)
        const y = 20 - Math.min(20, Math.max(0, (val / 40) * 20));
        points.push(`${i * 5},${y}`); 
    }
    return points.join(" ");
  }, [ticket.metrics]);

  return (
    <div 
      draggable={!!onDragStart}
      onDragStart={(e) => onDragStart && onDragStart(e, ticket.id)}
      onDragEnd={onDragEnd}
      onClick={() => onClick(ticket)}
      className={`
        relative overflow-hidden rounded-lg p-3 transition-all duration-200 group
        ${isDragging 
            ? 'opacity-30 scale-95 grayscale border-2 border-dashed border-[#555] bg-transparent shadow-none cursor-grabbing' 
            : 'bg-[#1a1a1a] border border-white/10 cursor-grab hover:border-white/20 hover:bg-[#1f1f1f] hover:shadow-2xl hover:-translate-y-1'
        }
      `}
    >
      <div className="flex justify-between items-start mb-2">
        <span className="text-[10px] font-mono text-gray-500 tracking-wide">{ticket.id}</span>
        {ticket.verification_status === 'success' && (
            <CheckCircle size={14} className="text-[#00FF94]" />
        )}
      </div>

      <h4 className="text-sm font-medium text-gray-200 mb-3 line-clamp-2 leading-snug group-hover:text-white font-sans select-none">
        {ticket.title}
      </h4>

      {/* Visual Pip */}
      {ticket.artifacts && (
        <div className="mb-3 relative h-24 w-full overflow-hidden rounded border border-white/10 bg-black pointer-events-none">
             <img src={ticket.artifacts.after_image} alt="preview" className="w-full h-full object-cover opacity-80 group-hover:opacity-100 transition-opacity" />
             {isSlow && (
                 <div className="absolute top-1 right-1 bg-[#FF0055]/90 backdrop-blur-sm text-white text-[9px] font-mono px-1.5 py-0.5 rounded-sm shadow-sm border border-[#FF0055]/50">
                     SLOW
                 </div>
             )}
        </div>
      )}

      {/* Footer */}
      <div className="flex items-center justify-between text-xs text-gray-500 mt-2 h-5 select-none">
        <div className="flex items-center gap-1.5">
           {ticket.owner.startsWith('agent') ? <Bot size={12} className="text-[#7000FF]"/> : <div className="w-3 h-3 rounded-full bg-gray-600"/>}
           <span className={`text-[10px] ${ticket.owner.startsWith('agent') ? 'text-[#7000FF] font-medium' : ''}`}>{ticket.owner.replace('agent-', '').replace('human-', '')}</span>
        </div>
        
        {ticket.metrics && !ticket.artifacts && (
             <div className="flex items-center gap-2">
                {/* Sparkline */}
                <svg width="45" height="20" className="opacity-70">
                    <polyline 
                        points={sparklinePoints} 
                        fill="none" 
                        stroke={isSlow ? '#FF0055' : '#00FF94'} 
                        strokeWidth="1.5" 
                        strokeLinecap="round"
                        strokeLinejoin="round"
                    />
                </svg>
                <div className={`flex items-center gap-1 font-mono text-[10px] ${isSlow ? 'text-[#FF0055]' : 'text-[#00FF94]'}`}>
                    {ticket.metrics.render_time_ms.toFixed(1)}ms
                </div>
             </div>
        )}

        {isHighPriority && !ticket.metrics && !ticket.artifacts && (
            <div className="flex items-center gap-1 text-[#FF0055] bg-[#FF0055]/10 px-1.5 py-0.5 rounded border border-[#FF0055]/20">
                <AlertCircle size={10} />
                <span className="text-[9px] font-bold uppercase">High</span>
            </div>
        )}
      </div>
    </div>
  );
};