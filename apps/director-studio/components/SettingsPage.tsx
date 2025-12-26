import React, { useState } from 'react';
import { Settings, Save, Server, Cpu, Database } from 'lucide-react';

export const SettingsPage: React.FC = () => {
    // This state would ideally come from a backend or local storage
    const [config, setConfig] = useState({
        provider: 'anthropic',
        model: 'claude-3-opus-20240229',
        poolSize: 1,
        contextStrategy: 'ast',
        radkitEndpoint: 'http://localhost:3000'
    });

    const handleChange = (e: React.ChangeEvent<HTMLInputElement | HTMLSelectElement>) => {
        const { name, value } = e.target;
        setConfig(prev => ({ ...prev, [name]: value }));
    };

    const handleSave = () => {
        // Save to backend or local storage
        console.log("Saving settings:", config);
        // We could post this to a new endpoint `/api/settings`
    };

    return (
        <div className="flex-1 bg-[#050505] p-8 overflow-y-auto">
             <div className="max-w-4xl mx-auto space-y-8">
                 <div className="flex items-center gap-3 mb-8">
                     <div className="p-3 bg-white/5 rounded-xl border border-white/10">
                        <Settings className="w-8 h-8 text-[#7000FF]" />
                     </div>
                     <div>
                         <h1 className="text-3xl font-bold text-white tracking-tight">Agent Settings</h1>
                         <p className="text-gray-400">Configure your autonomous Radkit workers</p>
                     </div>
                 </div>

                 <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                     {/* Model Configuration */}
                     <section className="bg-[#0a0a0a] border border-white/10 rounded-xl p-6 shadow-xl">
                        <div className="flex items-center gap-2 mb-6 text-white border-b border-white/5 pb-4">
                            <Cpu size={20} className="text-blue-400" />
                            <h2 className="font-semibold text-lg">Model Configuration</h2>
                        </div>

                        <div className="space-y-4">
                            <div>
                                <label className="block text-xs font-mono uppercase text-gray-500 mb-2">Provider</label>
                                <select
                                    name="provider"
                                    value={config.provider}
                                    onChange={handleChange}
                                    className="w-full bg-black border border-white/10 rounded p-3 text-sm text-white focus:border-blue-500 outline-none appearance-none"
                                >
                                    <option value="anthropic">Anthropic</option>
                                    <option value="openai">OpenAI</option>
                                    <option value="groq">Groq</option>
                                </select>
                            </div>
                            <div>
                                <label className="block text-xs font-mono uppercase text-gray-500 mb-2">Model</label>
                                <input
                                    type="text"
                                    name="model"
                                    value={config.model}
                                    onChange={handleChange}
                                    className="w-full bg-black border border-white/10 rounded p-3 text-sm text-white focus:border-blue-500 outline-none"
                                />
                            </div>
                        </div>
                     </section>

                     {/* Worker Configuration */}
                     <section className="bg-[#0a0a0a] border border-white/10 rounded-xl p-6 shadow-xl">
                        <div className="flex items-center gap-2 mb-6 text-white border-b border-white/5 pb-4">
                            <Server size={20} className="text-green-400" />
                            <h2 className="font-semibold text-lg">Worker Fleet</h2>
                        </div>

                        <div className="space-y-4">
                            <div>
                                <label className="block text-xs font-mono uppercase text-gray-500 mb-2">Pool Size (Concurrent Agents)</label>
                                <input
                                    type="number"
                                    name="poolSize"
                                    value={config.poolSize}
                                    onChange={handleChange}
                                    min={1}
                                    max={10}
                                    className="w-full bg-black border border-white/10 rounded p-3 text-sm text-white focus:border-green-500 outline-none"
                                />
                            </div>
                            <div>
                                <label className="block text-xs font-mono uppercase text-gray-500 mb-2">Context Strategy</label>
                                <select
                                    name="contextStrategy"
                                    value={config.contextStrategy}
                                    onChange={handleChange}
                                    className="w-full bg-black border border-white/10 rounded p-3 text-sm text-white focus:border-green-500 outline-none appearance-none"
                                >
                                    <option value="ast">AST (Smart Context)</option>
                                    <option value="regex">Regex (Legacy)</option>
                                    <option value="hybrid">Hybrid</option>
                                </select>
                            </div>
                        </div>
                     </section>

                     {/* Advanced */}
                     <section className="col-span-1 md:col-span-2 bg-[#0a0a0a] border border-white/10 rounded-xl p-6 shadow-xl">
                        <div className="flex items-center gap-2 mb-6 text-white border-b border-white/5 pb-4">
                            <Database size={20} className="text-purple-400" />
                            <h2 className="font-semibold text-lg">Radkit Endpoints</h2>
                        </div>
                         <div className="space-y-4">
                            <div>
                                <label className="block text-xs font-mono uppercase text-gray-500 mb-2">Director API URL</label>
                                <input
                                    type="text"
                                    name="radkitEndpoint"
                                    value={config.radkitEndpoint}
                                    onChange={handleChange}
                                    className="w-full bg-black border border-white/10 rounded p-3 text-sm text-white focus:border-purple-500 outline-none font-mono"
                                />
                            </div>
                         </div>
                     </section>
                 </div>

                 <div className="flex justify-end pt-4">
                     <button
                        onClick={handleSave}
                        className="flex items-center gap-2 px-8 py-3 bg-white text-black font-bold rounded-lg hover:bg-gray-200 transition-colors shadow-[0_0_20px_rgba(255,255,255,0.1)]"
                     >
                         <Save size={18} /> Save Settings
                     </button>
                 </div>
             </div>
        </div>
    );
};
