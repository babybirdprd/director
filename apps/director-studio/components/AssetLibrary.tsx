import React, { useState, useEffect } from 'react';
import { Asset } from '../types';
import { api } from '../services/api';
import { Type, Image, FileJson, Copy, Upload } from 'lucide-react';

export const AssetLibrary: React.FC = () => {
  const [assets, setAssets] = useState<Asset[]>([]);
  const [loading, setLoading] = useState(true);
  const [dragActive, setDragActive] = useState(false);
  const [copiedId, setCopiedId] = useState<string | null>(null);

  useEffect(() => {
    loadAssets();
  }, []);

  const loadAssets = async () => {
    const data = await api.getAssets();
    setAssets(data);
    setLoading(false);
  };

  const handleDrag = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    if (e.type === "dragenter" || e.type === "dragover") {
      setDragActive(true);
    } else if (e.type === "dragleave") {
      setDragActive(false);
    }
  };

  const handleDrop = async (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setDragActive(false);
    
    if (e.dataTransfer.files && e.dataTransfer.files[0]) {
      const file = e.dataTransfer.files[0];
      const newAsset = await api.uploadAsset(file);
      setAssets(prev => [...prev, newAsset]);
    }
  };

  const copySnippet = (asset: Asset) => {
    const snippet = `assets.load("${asset.path}")`;
    navigator.clipboard.writeText(snippet);
    setCopiedId(asset.id);
    setTimeout(() => setCopiedId(null), 2000);
  };

  const getIcon = (type: Asset['type']) => {
    switch(type) {
        case 'font': return <Type size={32} className="text-gray-500"/>;
        case 'image': return <Image size={32} className="text-gray-500"/>;
        case 'lottie': return <FileJson size={32} className="text-gray-500"/>;
    }
  };

  return (
    <div className="p-8 h-full flex flex-col" onDragEnter={handleDrag}>
      <header className="mb-8 flex justify-between items-end">
        <div>
            <h1 className="text-3xl font-bold text-white mb-2">Asset Library</h1>
            <p className="text-gray-400">Drag files anywhere to ingest into the engine.</p>
        </div>
        <div className="text-xs font-mono text-gray-600 bg-[#1a1a1a] px-3 py-1 rounded border border-[#333]">
            {assets.length} items loaded
        </div>
      </header>

      {/* Grid */}
      <div className="grid grid-cols-2 md:grid-cols-4 lg:grid-cols-5 gap-4 relative">
        {assets.map(asset => (
            <div 
                key={asset.id}
                onClick={() => copySnippet(asset)}
                className="group relative aspect-square bg-[#1a1a1a] border border-[#333] rounded-lg overflow-hidden hover:border-[#7000FF] cursor-pointer transition-colors"
            >
                {/* Content */}
                <div className="absolute inset-0 flex items-center justify-center p-4">
                    {asset.type === 'image' && asset.preview_url ? (
                        <img src={asset.preview_url} alt={asset.name} className="max-w-full max-h-full object-contain" />
                    ) : (
                        getIcon(asset.type)
                    )}
                </div>

                {/* Overlay Info */}
                <div className="absolute inset-x-0 bottom-0 bg-black/90 p-3 translate-y-full group-hover:translate-y-0 transition-transform">
                    <p className="text-xs font-bold text-white truncate">{asset.name}</p>
                    <p className="text-[10px] font-mono text-gray-500 mt-1 truncate">{asset.rust_id}</p>
                    {copiedId === asset.id ? (
                        <div className="absolute top-2 right-2 text-[#00FF94] text-xs flex items-center gap-1">
                            Copied!
                        </div>
                    ) : (
                        <div className="absolute top-2 right-2 text-gray-500 opacity-0 group-hover:opacity-100 transition-opacity">
                            <Copy size={12} />
                        </div>
                    )}
                </div>
            </div>
        ))}

        {/* Upload Placeholder */}
        <div className="aspect-square border-2 border-dashed border-[#333] rounded-lg flex flex-col items-center justify-center text-gray-600 hover:text-gray-400 hover:border-gray-500 transition-colors">
            <Upload size={24} className="mb-2" />
            <span className="text-xs font-medium">Drop to Upload</span>
        </div>
      </div>

      {/* Drag Overlay */}
      {dragActive && (
        <div 
            className="absolute inset-0 bg-[#7000FF]/20 backdrop-blur-sm z-50 flex items-center justify-center border-4 border-[#7000FF] rounded-lg m-4"
            onDragEnter={handleDrag}
            onDragLeave={handleDrag}
            onDragOver={handleDrag}
            onDrop={handleDrop}
        >
            <div className="text-[#7000FF] text-2xl font-bold flex flex-col items-center">
                <Upload size={48} className="mb-4" />
                Release to Ingest
            </div>
        </div>
      )}
    </div>
  );
};