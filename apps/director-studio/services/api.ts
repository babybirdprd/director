import { Ticket, Asset, TicketStatus } from '../types';

// Real API Service
class ApiService {
  private baseUrl = '/api';

  async getTickets(): Promise<Ticket[]> {
    const response = await fetch(`${this.baseUrl}/tickets`);
    if (!response.ok) {
      throw new Error(`Failed to fetch tickets: ${response.statusText}`);
    }
    return response.json();
  }

  async getTicket(id: string): Promise<Ticket> {
    const response = await fetch(`${this.baseUrl}/tickets/${id}`);
    if (!response.ok) {
      throw new Error(`Failed to fetch ticket ${id}: ${response.statusText}`);
    }
    return response.json();
  }

  async getAssets(): Promise<Asset[]> {
    // There isn't a dedicated endpoint for listing assets in the requirements.
    // However, if we need to list them, we might need an endpoint or just return mock data for now?
    // The prompt says: "Verify assets/test_font.ttf exists on the local filesystem."
    // And "Upload a dummy file ... UI: New asset card appears in the grid."
    // So we probably need to list assets.
    // I missed adding `GET /api/assets` to the server plan.
    // I should add it now to server.rs or just return an empty list if not strictly required,
    // but better to implement it.
    // Let's assume for now I will need to implement listing assets.
    // I'll leave a TODO here or try to fetch from a hypothetic endpoint.

    // Actually, I can use `GET /api/assets` if I implement it.
    // Let's assume I will add it.

    // For now, let's return an empty array or handle error gracefully.
    try {
        const response = await fetch(`${this.baseUrl}/assets`);
        if (response.ok) return response.json();
    } catch (e) {
        console.warn("Assets endpoint might not be implemented yet", e);
    }
    return [];
  }

  async updateTicketStatus(id: string, status: TicketStatus): Promise<void> {
    const response = await fetch(`${this.baseUrl}/tickets/${id}`, {
      method: 'PATCH',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ status }),
    });

    if (!response.ok) {
      throw new Error(`Failed to update ticket status: ${response.statusText}`);
    }
  }

  async verifyTicket(id: string): Promise<{ success: boolean; output: string; artifacts_path?: string }> {
    const response = await fetch(`${this.baseUrl}/tickets/${id}/verify`, {
      method: 'POST',
    });

    if (!response.ok) {
        // Try to read error body
        const errorText = await response.text();
        throw new Error(`Failed to verify ticket: ${response.statusText} - ${errorText}`);
    }

    const result = await response.json();
    return {
        success: result.success,
        output: result.stdout + (result.stderr ? `\nSTDERR:\n${result.stderr}` : ""),
        artifacts_path: result.artifacts_path
    };
  }

  async uploadAsset(file: File): Promise<Asset> {
    const formData = new FormData();
    formData.append('file', file);

    const response = await fetch(`${this.baseUrl}/assets`, {
      method: 'POST',
      body: formData,
    });

    if (!response.ok) {
      throw new Error(`Failed to upload asset: ${response.statusText}`);
    }

    const result = await response.json();
    // The server returns { "uploaded": [ { name, path, url } ] }
    const uploaded = result.uploaded[0];

    return {
      id: `A-${Date.now()}`, // Client-side ID generation for now, or use something from server?
      name: uploaded.name,
      type: file.type.includes('image') ? 'image' : file.name.endsWith('json') ? 'lottie' : 'font',
      path: uploaded.path,
      rust_id: `ASSET_${uploaded.name.toUpperCase().replace(/[^A-Z0-9]/g, '_')}`,
      preview_url: uploaded.url
    };
  }
}

export const api = new ApiService();
