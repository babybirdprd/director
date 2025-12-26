import React, { useState } from 'react';
import { X, Check, Activity, Terminal, Clock, ShieldAlert, Play, MessageSquare } from 'lucide-react';
import { Ticket } from '../types';
import { ImageComparator } from './ImageComparator';
import { LineChart, Line, XAxis, YAxis, Tooltip, ResponsiveContainer } from 'recharts';

interface TicketDetailModalProps {
  ticket: Ticket;
  onClose: () => void;
  onVerify: (id: string) => void;
}

// Mock perf history data
const PERF_DATA = [
  { frame: 1, ms: 16.1 },
  { frame: 2, ms: 16.3 },
  { frame: 3, ms: 16.4 },
  { frame: 4, ms: 15.9 },
  { frame: 5, ms: 16.2 },
  { frame: 6, ms: 16.4 }, // Current
];

export const TicketDetailModal: React.FC<TicketDetailModalProps> = ({ ticket, onClose, onVerify }) => {
  const [isVerifying, setIsVerifying] = useState(false);
  const [rejectionMode, setRejectionMode] = useState(false);
  const [feedback, setFeedback] = useState('');

  const handleVerify = () => {
    setIsVerifying(true);
    // Simulate verification delay
    setTimeout(() => {
        setIsVerifying(false);
        onVerify(ticket.id);
    }, 2000);
  };

  const handleReject = () => {
      console.log(`Rejected ${ticket.id} with feedback: ${feedback}`);
      onClose();
  }

  const handleApprove = () => {
      console.log(`Approved ${ticket.id}`);
      onClose();
  }

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/90 backdrop-blur-md animate-in fade-in duration-200">
      <div className="bg-[#0a0a0a] w-[95vw] h-[90vh] max-w-[1600px] rounded-xl border border-white/10 shadow-2xl flex overflow-hidden ring-1 ring-white/5">
        
        {/* Left Column: Context */}
        <div className="w-[35%] border-r border-white/10 flex flex-col bg-[#0f0f0f]">
          {/* Header */}
          <div className="p-6 border-b border-white/10 bg-[#141414]">
            <div className="flex items-center gap-2 mb-3">
                <span className={`px-2 py-0.5 text-[10px] uppercase font-bold tracking-wider rounded border ${
                    ticket.priority === 'high' 
                    ? 'bg-[#FF0055]/10 border-[#FF0055]/20 text-[#FF0055]' 
                    : 'bg-gray-800/50 border-gray-700 text-gray-400'
                }`}>
                    {ticket.priority} Priority
                </span>
                <span className="text-gray-500 font-mono text-xs tracking-wide">{ticket.id}</span>
            </div>
            <h2 className="text-xl font-bold text-white mb-2 leading-tight">{ticket.title}</h2>
            <div className="flex items-center gap-2 text-xs text-gray-400 font-medium">
                <span className="w-2 h-2 rounded-full bg-[#7000FF] shadow-[0_0_8px_#7000FF]"></span>
                <span>Owner: <span className="text-gray-300">{ticket.owner}</span></span>
            </div>
          </div>

          {/* Scrollable Content */}
          <div className="flex-1 overflow-y-auto p-6 space-y-8 custom-scrollbar">
            {/* Description */}
            <section>
                <h3 className="text-[10px] font-mono uppercase text-gray-500 mb-3 tracking-widest">Description</h3>
                <p className="text-gray-300 text-sm leading-relaxed whitespace-pre-line">{ticket.description}</p>
            </section>

            {/* Specs */}
            {ticket.specs && (
                <section>
                    <h3 className="text-[10px] font-mono uppercase text-gray-500 mb-3 tracking-widest">Specifications</h3>
                    <div className="bg-[#080808] p-4 rounded border border-white/5 text-sm font-mono text-gray-300 whitespace-pre-line leading-relaxed shadow-inner">
                        {ticket.specs}
                    </div>
                </section>
            )}

            {/* Logs */}
            {ticket.logs && (
                <section>
                    <h3 className="text-[10px] font-mono uppercase text-gray-500 mb-3 flex items-center gap-2 tracking-widest">
                        <Terminal size={12} /> Agent Logs
                    </h3>
                    <div className="bg-black p-4 rounded border border-white/5 font-mono text-xs space-y-2 shadow-inner h-48 overflow-y-auto custom-scrollbar">
                        {ticket.logs.map((log, i) => (
                            <div key={i} className={`flex gap-2 ${log.includes('[AGENT]') ? 'text-[#7000FF]' : 'text-[#00FF94]'}`}>
                                <span className="opacity-30 select-none">{i+1}</span>
                                <span>{log}</span>
                            </div>
                        ))}
                         {isVerifying && (
                             <div className="flex gap-2 text-gray-500 animate-pulse">
                                 <span className="opacity-30">&gt;</span>
                                 <span>Running verification suite...</span>
                             </div>
                         )}
                    </div>
                </section>
            )}
          </div>

          {/* Footer Actions (Context side) */}
           <div className="p-4 border-t border-white/10 bg-[#141414]">
              {!rejectionMode ? (
                  <div className="flex gap-3">
                    <button 
                        onClick={() => setRejectionMode(true)}
                        className="flex-1 py-3 px-4 rounded bg-[#2a0000] border border-[#FF0055]/30 text-[#FF0055] hover:bg-[#FF0055]/10 font-medium text-sm transition-colors flex items-center justify-center gap-2"
                    >
                        <ShieldAlert size={16} /> Reject
                    </button>
                    <button 
                        onClick={handleApprove}
                        className="flex-1 py-3 px-4 rounded bg-[#002a18] border border-[#00FF94]/30 text-[#00FF94] hover:bg-[#00FF94]/10 font-medium text-sm transition-colors flex items-center justify-center gap-2"
                    >
                        <Check size={16} /> Approve & Merge
                    </button>
                  </div>
              ) : (
                  <div className="space-y-3">
                      <div className="relative">
                        <MessageSquare size={14} className="absolute top-3 left-3 text-gray-500"/>
                        <textarea 
                            autoFocus
                            value={feedback}
                            onChange={(e) => setFeedback(e.target.value)}
                            placeholder="Provide feedback to the Agent..."
                            className="w-full bg-black border border-white/10 rounded p-2 pl-9 text-sm text-white focus:border-[#7000FF] outline-none min-h-[80px]"
                        />
                      </div>
                      <div className="flex gap-2">
                          <button onClick={() => setRejectionMode(false)} className="flex-1 py-2 text-xs text-gray-400 hover:text-white">Cancel</button>
                          <button onClick={handleReject} className="flex-1 py-2 bg-[#FF0055] text-white rounded text-xs font-bold hover:bg-[#D40046] shadow-[0_0_10px_#FF0055]">Confirm Reject</button>
                      </div>
                  </div>
              )}
           </div>
        </div>

        {/* Right Column: Visuals */}
        <div className="w-[65%] flex flex-col bg-[#050505]">
            {/* Toolbar */}
            <div className="h-16 border-b border-white/10 flex items-center justify-between px-6 bg-[#141414]">
                <div className="flex items-center gap-4">
                    <h2 className="font-semibold text-white tracking-tight">Verification Suite</h2>
                    {ticket.metrics && (
                        <div className={`flex items-center gap-2 px-3 py-1 rounded-full border text-xs font-mono ${
                            ticket.metrics.render_time_ms > 30 
                            ? 'bg-[#FF0055]/10 border-[#FF0055]/30 text-[#FF0055]' 
                            : 'bg-[#00FF94]/10 border-[#00FF94]/30 text-[#00FF94]'
                        }`}>
                            <Clock size={12} />
                            {ticket.metrics.render_time_ms}ms ({ticket.metrics.render_time_diff})
                        </div>
                    )}
                </div>
                <div className="flex items-center gap-2">
                     <button onClick={handleVerify} disabled={isVerifying} className="flex items-center gap-2 px-3 py-1.5 bg-white/5 border border-white/10 text-gray-300 text-xs font-medium hover:bg-white/10 hover:text-white rounded transition-colors disabled:opacity-50">
                        <Play size={14} fill="currentColor" /> Rerun Tests
                    </button>
                    <div className="w-px h-4 bg-white/10 mx-2"></div>
                    <button onClick={onClose} className="p-2 text-gray-400 hover:text-white hover:bg-white/10 rounded-full transition-colors">
                        <X size={20} />
                    </button>
                </div>
            </div>

            {/* Main Visual Area */}
            <div className="flex-1 p-6 flex flex-col gap-6 overflow-y-auto bg-[url('https://grainy-gradients.vercel.app/noise.svg')]">
                {/* Visual Comparator */}
                {ticket.artifacts ? (
                    <div className="flex-1 min-h-[400px]">
                        <ImageComparator 
                            beforeUrl={ticket.artifacts.before_image}
                            afterUrl={ticket.artifacts.after_image}
                            diffUrl={ticket.artifacts.diff_image}
                        />
                    </div>
                ) : (
                    <div className="flex-1 flex flex-col items-center justify-center border border-dashed border-white/10 rounded-lg text-gray-600 bg-white/5">
                        <Activity size={32} className="mb-2 opacity-50" />
                        <span>No visual artifacts generated yet.</span>
                    </div>
                )}

                {/* Performance Graph */}
                <div className="h-48 bg-[#0a0a0a] rounded-lg border border-white/10 p-4 shadow-lg">
                    <div className="flex items-center justify-between mb-2">
                         <h4 className="text-[10px] font-mono uppercase text-gray-500 flex items-center gap-2 tracking-wider">
                            <Activity size={12} /> Render Time (Last 6 Runs)
                        </h4>
                    </div>
                    <div className="h-32 w-full">
                        <ResponsiveContainer width="100%" height="100%">
                            <LineChart data={PERF_DATA}>
                                <XAxis dataKey="frame" hide />
                                <YAxis domain={[10, 20]} hide />
                                <Tooltip 
                                    contentStyle={{ backgroundColor: '#000', border: '1px solid #333', borderRadius: '4px' }}
                                    itemStyle={{ color: '#00FF94', fontFamily: 'monospace', fontSize: '12px' }}
                                    labelStyle={{ display: 'none' }}
                                />
                                <Line 
                                    type="monotone" 
                                    dataKey="ms" 
                                    stroke="#00FF94" 
                                    strokeWidth={2} 
                                    dot={{ fill: '#000', stroke: '#00FF94', r: 3, strokeWidth: 2 }}
                                    activeDot={{ r: 5, fill: '#00FF94' }}
                                />
                            </LineChart>
                        </ResponsiveContainer>
                    </div>
                </div>
            </div>
        </div>
      </div>
    </div>
  );
};