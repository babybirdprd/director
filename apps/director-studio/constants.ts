import { Ticket, Asset, Column } from './types';

// Theme Colors
export const COLORS = {
  background: '#000000',
  surface: '#1a1a1a',
  surfaceHighlight: '#2a2a2a',
  success: '#00FF94',
  error: '#FF0055',
  agent: '#7000FF',
  text: '#e5e5e5',
  textDim: '#a1a1a1',
};

export const COLUMNS: Column[] = [
  { id: 'todo', label: 'To Do' },
  { id: 'in_progress', label: 'Active' },
  { id: 'review', label: 'Review' },
  { id: 'done', label: 'Done' },
];

export const MOCK_TICKETS: Ticket[] = [
  {
    id: 'T-001',
    title: 'Implement Text Shadows',
    description: 'The engine needs to support soft drop shadows for text nodes to improve legibility against complex backgrounds.',
    specs: '- Shadow Color: RGBA support\n- Blur Radius: 0-20px\n- Offset: X/Y independent',
    status: 'review',
    priority: 'high',
    owner: 'agent-claude',
    verification_status: 'success',
    metrics: {
      render_time_ms: 16.4,
      render_time_diff: '+0.2',
    },
    artifacts: {
      before_image: 'https://picsum.photos/id/11/800/600', // Landscape
      after_image: 'https://picsum.photos/id/10/800/600',  // Forest
      diff_image: 'https://picsum.photos/id/12/800/600',   // Beach (Simulating diff)
    },
    logs: [
      '[AGENT] Analyzing render pipeline...',
      '[AGENT] Identified shadow pass injection point in `renderer.rs`.',
      '[AGENT] Compiling shaders...',
      '[SYSTEM] Build successful in 4.2s.',
      '[AGENT] Generated verification artifacts.'
    ]
  },
  {
    id: 'T-002',
    title: 'Optimize Particle System',
    description: 'Particle rendering is causing frame drops in heavy scenes.',
    status: 'in_progress',
    priority: 'high',
    owner: 'agent-devin',
    verification_status: 'pending',
    metrics: {
      render_time_ms: 34.2, // Slow!
      render_time_diff: '+12.0',
    },
    logs: ['[AGENT] Profiling GPU buffers...']
  },
  {
    id: 'T-003',
    title: 'Fix Asset Loading Race Condition',
    description: 'Occasional crash when loading Lottie files concurrently.',
    status: 'todo',
    priority: 'medium',
    owner: 'human-lead',
    verification_status: 'pending',
  },
  {
    id: 'T-004',
    title: 'Release v1.2.0',
    description: 'Finalize changelog and tag release.',
    status: 'done',
    priority: 'low',
    owner: 'human-pm',
    verification_status: 'success',
  }
];

export const MOCK_ASSETS: Asset[] = [
  {
    id: 'A-1',
    name: 'Inter-Regular.ttf',
    type: 'font',
    path: 'assets/fonts/Inter-Regular.ttf',
    rust_id: 'FONT_INTER_REG',
  },
  {
    id: 'A-2',
    name: 'Hero_Background.png',
    type: 'image',
    path: 'assets/images/hero_bg.png',
    rust_id: 'IMG_HERO_BG',
    preview_url: 'https://picsum.photos/id/20/200/200',
  },
  {
    id: 'A-3',
    name: 'Loader_Spinner.json',
    type: 'lottie',
    path: 'assets/lottie/spinner.json',
    rust_id: 'LOTTIE_SPINNER',
  },
];
