// ScanUtils.tsx - API utilities for scan operations

export interface ScanData {
  id: string;
  campground_id: string;
  campground_name: string;
  check_in_date: string; // ISO date string (YYYY-MM-DD)
  check_out_date: string; // ISO date string (YYYY-MM-DD)
  nights: number;
  status: "active" | "paused" | "completed" | "cancelled";
  notification_sent: boolean;
  created_at: string; // ISO datetime string
  updated_at: string; // ISO datetime string
  expires_at?: string; // ISO datetime string, optional
}

export interface CreateScanRequest {
  campground_id: string;
  campground_name: string;
  check_in_date: string; // ISO date string (YYYY-MM-DD)
  check_out_date: string; // ISO date string (YYYY-MM-DD)
}

export interface CreateScanResponse {
  id: string;
  campground_id: string;
  campground_name: string;
  check_in_date: string;
  check_out_date: string;
  nights: number;
  status: string;
  notification_sent: boolean;
  created_at: string;
}

export interface ListScansResponse {
  scans: ScanData[];
  total: number;
}

export interface UpdateScanRequest {
  status: "active" | "paused" | "completed" | "cancelled";
}

// API utility functions
const getAuthHeaders = () => {
  const token = localStorage.getItem("access_token");
  return {
    Authorization: `Bearer ${token}`,
    "Content-Type": "application/json",
  };
};

export const createScan = async (
  scanData: CreateScanRequest,
): Promise<CreateScanResponse> => {
  const response = await fetch("/api/scans", {
    method: "POST",
    headers: getAuthHeaders(),
    body: JSON.stringify(scanData),
  });

  if (!response.ok) {
    const errorData = await response.json();
    throw new Error(errorData.message || "Failed to create scan");
  }

  return response.json();
};

export const getUserScans = async (): Promise<ListScansResponse> => {
  const response = await fetch("/api/scans", {
    method: "GET",
    headers: getAuthHeaders(),
  });

  if (!response.ok) {
    const errorData = await response.json();
    throw new Error(errorData.message || "Failed to fetch scans");
  }

  return response.json();
};

export const getActiveScans = async (): Promise<ListScansResponse> => {
  const response = await fetch("/api/scans/active", {
    method: "GET",
    headers: getAuthHeaders(),
  });

  if (!response.ok) {
    const errorData = await response.json();
    throw new Error(errorData.message || "Failed to fetch active scans");
  }

  return response.json();
};

export const getScan = async (scanId: string): Promise<ScanData> => {
  const response = await fetch(`/api/scans/${scanId}`, {
    method: "GET",
    headers: getAuthHeaders(),
  });

  if (!response.ok) {
    const errorData = await response.json();
    throw new Error(errorData.message || "Failed to fetch scan");
  }

  return response.json();
};

export const updateScan = async (
  scanId: string,
  updateData: UpdateScanRequest,
): Promise<ScanData> => {
  const response = await fetch(`/api/scans/${scanId}`, {
    method: "PUT",
    headers: getAuthHeaders(),
    body: JSON.stringify(updateData),
  });

  if (!response.ok) {
    const errorData = await response.json();
    throw new Error(errorData.message || "Failed to update scan");
  }

  return response.json();
};

export const deleteScan = async (scanId: string): Promise<void> => {
  const response = await fetch(`/api/scans/${scanId}`, {
    method: "DELETE",
    headers: getAuthHeaders(),
  });

  if (!response.ok) {
    const errorData = await response.json();
    throw new Error(errorData.message || "Failed to delete scan");
  }
};

// Helper function to format dates for API
export const formatDateForApi = (dateString: string): string => {
  return new Date(dateString).toISOString().split("T")[0];
};

// Helper function to format dates for display
export const formatDateForDisplay = (dateString: string): string => {
  return new Date(dateString).toLocaleDateString();
};

// Helper function to get status badge color
export const getStatusColor = (status: string): string => {
  switch (status) {
    case "active":
      return "#22c55e"; // green
    case "paused":
      return "#f59e0b"; // yellow
    case "completed":
      return "#3b82f6"; // blue
    case "cancelled":
      return "#ef4444"; // red
    default:
      return "#6b7280"; // gray
  }
};
