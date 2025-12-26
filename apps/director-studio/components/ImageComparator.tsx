import React, { useState, useRef, useEffect, useCallback } from 'react';
import { ChevronLeft, ChevronRight, Eye, Layers, ScanEye } from 'lucide-react';

interface ImageComparatorProps {
  beforeUrl: string;
  afterUrl: string;
  diffUrl?: string; // Optional overlay
}

export const ImageComparator: React.FC<ImageComparatorProps> = ({ beforeUrl, afterUrl, diffUrl }) => {
  const [sliderPosition, setSliderPosition] = useState(50);
  const [isResizing, setIsResizing] = useState(false);
  const [mode, setMode] = useState<'slider' | 'diff'>('slider');
  const containerRef = useRef<HTMLDivElement>(null);

  const handleMouseDown = useCallback(() => setIsResizing(true), []);
  const handleMouseUp = useCallback(() => setIsResizing(false), []);
  const handleMouseMove = useCallback((e: MouseEvent) => {
    if (!isResizing || !containerRef.current) return;
    const rect = containerRef.current.getBoundingClientRect();
    const x = Math.max(0, Math.min(e.clientX - rect.left, rect.width));
    const percentage = (x / rect.width) * 100;
    setSliderPosition(Math.max(0, Math.min(100, percentage)));
  }, [isResizing]);

  useEffect(() => {
    if (isResizing) {
      window.addEventListener('mousemove', handleMouseMove);
      window.addEventListener('mouseup', handleMouseUp);
    }
    return () => {
      window.removeEventListener('mousemove', handleMouseMove);
      window.removeEventListener('mouseup', handleMouseUp);
    };
  }, [isResizing, handleMouseMove, handleMouseUp]);

  return (
    <div className="flex flex-col h-full bg-[#0a0a0a] rounded-lg overflow-hidden border border-white/10 shadow-2xl">
      {/* Toolbar */}
      <div className="flex items-center justify-between px-3 py-2 border-b border-white/10 bg-[#141414]">
        <div className="flex items-center gap-2">
            <ScanEye size={14} className="text-[#00FF94]" />
            <span className="text-xs font-mono text-gray-400 uppercase tracking-wider">Visual Verification</span>
        </div>
        <div className="flex space-x-1 bg-black/50 rounded p-0.5 border border-white/5">
          <button
            onClick={() => setMode('slider')}
            className={`px-3 py-1 rounded text-[10px] font-medium flex items-center gap-1.5 transition-colors ${mode === 'slider' ? 'bg-[#333] text-white shadow-sm' : 'text-gray-500 hover:text-gray-300'}`}
          >
            <Layers size={12} /> Slider
          </button>
          {diffUrl && (
            <button
              onClick={() => setMode('diff')}
              className={`px-3 py-1 rounded text-[10px] font-medium flex items-center gap-1.5 transition-colors ${mode === 'diff' ? 'bg-[#333] text-white shadow-sm' : 'text-gray-500 hover:text-gray-300'}`}
            >
              <Eye size={12} /> Diff Overlay
            </button>
          )}
        </div>
      </div>

      {/* Viewport */}
      <div 
        ref={containerRef}
        className="relative flex-1 w-full overflow-hidden select-none group cursor-col-resize"
        onMouseDown={mode === 'slider' ? handleMouseDown : undefined}
      >
        {mode === 'slider' ? (
          <>
            {/* Background (After / Agent) */}
            <div className="absolute inset-0">
                <img 
                  src={afterUrl} 
                  alt="After" 
                  className="w-full h-full object-cover" 
                  draggable={false}
                />
            </div>
            
            {/* Foreground (Before / Golden) - Clipped */}
            <div 
              className="absolute inset-y-0 left-0 overflow-hidden z-10"
              style={{ width: `${sliderPosition}%` }}
            >
              <img 
                src={beforeUrl} 
                alt="Before" 
                className="absolute inset-y-0 left-0 max-w-none h-full object-cover"
                style={{ width: containerRef.current?.offsetWidth || '100%' }}
                draggable={false}
              />
              {/* Border line on the clipping edge */}
              <div className="absolute inset-y-0 right-0 w-[2px] bg-[#00FF94] shadow-[0_0_10px_#00FF94]"></div>
            </div>

            {/* Handle */}
            <div 
              className="absolute inset-y-0 -ml-4 w-8 z-20 flex items-center justify-center pointer-events-none"
              style={{ left: `${sliderPosition}%` }}
            >
              <div className="w-8 h-8 rounded-full bg-[#00FF94] shadow-[0_0_15px_rgba(0,0,0,0.5)] flex items-center justify-center text-black transform transition-transform hover:scale-110">
                <div className="flex -space-x-1">
                  <ChevronLeft size={12} strokeWidth={3} />
                  <ChevronRight size={12} strokeWidth={3} />
                </div>
              </div>
            </div>

            {/* Labels */}
            <div className="absolute bottom-4 left-4 bg-black/60 backdrop-blur-sm px-2 py-1 rounded-sm text-[10px] font-mono text-[#00FF94] border border-[#00FF94]/30 pointer-events-none z-30">
              GOLDEN MASTER
            </div>
            <div className="absolute bottom-4 right-4 bg-black/60 backdrop-blur-sm px-2 py-1 rounded-sm text-[10px] font-mono text-white border border-white/20 pointer-events-none z-30">
              AGENT OUTPUT
            </div>
          </>
        ) : (
          <div className="relative w-full h-full cursor-default">
             <img 
              src={diffUrl} 
              alt="Diff" 
              className="w-full h-full object-cover" 
              draggable={false}
            />
             {/* Red overlay tint for effect */}
             <div className="absolute inset-0 pointer-events-none mix-blend-overlay bg-[#FF0055]/10"></div>
             
             <div className="absolute bottom-4 right-4 bg-black/80 backdrop-blur-sm px-2 py-1 rounded-sm text-[10px] font-mono text-[#FF0055] border border-[#FF0055]/30 pointer-events-none">
              PIXEL DIFF
            </div>
          </div>
        )}
      </div>
    </div>
  );
};