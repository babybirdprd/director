export interface Metrics {
  render_time_ms: number;
  render_time_diff: string; // e.g. "+0.2"
}

export interface Artifacts {
  before_image: string;
  after_image: string;
  diff_image?: string;
}

export type TicketStatus = 'todo' | 'in_progress' | 'review' | 'done' | 'archived';

export interface Ticket {
  id: string;
  title: string;
  description: string;
  status: TicketStatus;
  priority: 'low' | 'medium' | 'high' | 'critical';
  owner: string;
  verification_status: 'pending' | 'success' | 'failure';
  metrics?: Metrics;
  artifacts?: Artifacts;
  logs?: string[];
  specs?: string;
}

export interface Asset {
  id: string;
  name: string;
  type: 'font' | 'image' | 'lottie';
  path: string;
  rust_id: string;
  preview_url?: string;
}

export interface Column {
  id: TicketStatus;
  label: string;
}
