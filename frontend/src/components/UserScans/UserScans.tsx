import React, { useState, useEffect } from "react";
import {
  Tent,
  Calendar,
  MapPin,
  Clock,
  Trash2,
  Play,
  Pause,
  CheckCircle,
  XCircle,
  RefreshCw,
} from "lucide-react";
import {
  ScanData,
  getUserScans,
  updateScan,
  deleteScan,
  formatDateForDisplay,
  getStatusColor,
} from "../CreateScan/ScanUtils";
import "./UserScans.css";

interface UserScansProps {
  refreshTrigger?: number; // Optional prop to trigger refresh from parent
}

const UserScans: React.FC<UserScansProps> = ({ refreshTrigger }) => {
  const [scans, setScans] = useState<ScanData[]>([]);
  const [isLoading, setIsLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);
  const [updatingScans, setUpdatingScans] = useState<Set<string>>(new Set());

  const loadScans = async () => {
    try {
      setIsLoading(true);
      setError(null);
      const response = await getUserScans();
      setScans(response.scans);
    } catch (err) {
      console.error("Error loading scans:", err);
      setError(err instanceof Error ? err.message : "Failed to load scans");
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    loadScans();
  }, [refreshTrigger]);

  const handleStatusUpdate = async (
    scanId: string,
    newStatus: "active" | "paused" | "completed" | "cancelled",
  ) => {
    try {
      setUpdatingScans((prev) => new Set(prev).add(scanId));
      await updateScan(scanId, { status: newStatus });

      // Update the scan in our local state
      setScans((prevScans) =>
        prevScans.map((scan) =>
          scan.id === scanId ? { ...scan, status: newStatus } : scan,
        ),
      );
    } catch (err) {
      console.error("Error updating scan:", err);
      alert(err instanceof Error ? err.message : "Failed to update scan");
    } finally {
      setUpdatingScans((prev) => {
        const newSet = new Set(prev);
        newSet.delete(scanId);
        return newSet;
      });
    }
  };

  const handleDeleteScan = async (scanId: string, campgroundName: string) => {
    if (
      !window.confirm(
        `Are you sure you want to delete the scan for ${campgroundName}?`,
      )
    ) {
      return;
    }

    try {
      setUpdatingScans((prev) => new Set(prev).add(scanId));
      await deleteScan(scanId);

      // Remove the scan from our local state
      setScans((prevScans) => prevScans.filter((scan) => scan.id !== scanId));
    } catch (err) {
      console.error("Error deleting scan:", err);
      alert(err instanceof Error ? err.message : "Failed to delete scan");
    } finally {
      setUpdatingScans((prev) => {
        const newSet = new Set(prev);
        newSet.delete(scanId);
        return newSet;
      });
    }
  };

  const getStatusIcon = (status: string) => {
    switch (status) {
      case "active":
        return <Play size={16} />;
      case "paused":
        return <Pause size={16} />;
      case "completed":
        return <CheckCircle size={16} />;
      case "cancelled":
        return <XCircle size={16} />;
      default:
        return <Clock size={16} />;
    }
  };

  const getActionButtons = (scan: ScanData) => {
    const isUpdating = updatingScans.has(scan.id);

    return (
      <div className="scan-actions">
        {scan.status === "active" && (
          <button
            className="action-btn pause-btn"
            onClick={() => handleStatusUpdate(scan.id, "paused")}
            disabled={isUpdating}
            title="Pause scan"
          >
            <Pause size={14} />
          </button>
        )}

        {scan.status === "paused" && (
          <button
            className="action-btn resume-btn"
            onClick={() => handleStatusUpdate(scan.id, "active")}
            disabled={isUpdating}
            title="Resume scan"
          >
            <Play size={14} />
          </button>
        )}

        {(scan.status === "active" || scan.status === "paused") && (
          <button
            className="action-btn complete-btn"
            onClick={() => handleStatusUpdate(scan.id, "completed")}
            disabled={isUpdating}
            title="Mark as completed"
          >
            <CheckCircle size={14} />
          </button>
        )}

        <button
          className="action-btn delete-btn"
          onClick={() => handleDeleteScan(scan.id, scan.campground_name)}
          disabled={isUpdating}
          title="Delete scan"
        >
          <Trash2 size={14} />
        </button>
      </div>
    );
  };

  if (isLoading) {
    return (
      <div className="user-scans loading">
        <div className="loading-spinner">
          <RefreshCw className="spinning" size={24} />
          <p>Loading your scans...</p>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="user-scans error">
        <div className="error-message">
          <XCircle size={24} />
          <p>{error}</p>
          <button onClick={loadScans} className="retry-btn">
            Try Again
          </button>
        </div>
      </div>
    );
  }

  if (scans.length === 0) {
    return (
      <div className="user-scans empty">
        <div className="empty-state">
          <Tent size={48} />
          <h3>No scans yet</h3>
          <p>
            Create your first scan to start monitoring campground availability!
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="user-scans">
      <div className="scans-header">
        <h3>Your Campground Scans</h3>
        <span className="scan-count">{scans.length} total</span>
      </div>

      <div className="scans-list">
        {scans.map((scan) => (
          <div key={scan.id} className={`scan-card ${scan.status}`}>
            <div className="scan-info">
              <div className="user-scan-header">
                <div className="campground-info">
                  <MapPin size={16} />
                  <h4 className="campground-name">{scan.campground_name}</h4>
                </div>
                <div
                  className="status-badge"
                  style={{ backgroundColor: getStatusColor(scan.status) }}
                >
                  {getStatusIcon(scan.status)}
                  <span>{scan.status}</span>
                </div>
              </div>

              <div className="scan-details">
                <div className="date-range">
                  <Calendar size={14} />
                  <span>
                    {formatDateForDisplay(scan.check_in_date)} -{" "}
                    {formatDateForDisplay(scan.check_out_date)}
                  </span>
                  <span className="nights">({scan.nights} nights)</span>
                </div>

                <div className="scan-meta">
                  <div className="created-date">
                    <Clock size={14} />
                    <span>Created {formatDateForDisplay(scan.created_at)}</span>
                  </div>

                  {scan.notification_sent && (
                    <div className="notification-status">
                      <CheckCircle size={14} />
                      <span>Notification sent</span>
                    </div>
                  )}
                </div>
              </div>
            </div>

            {getActionButtons(scan)}
          </div>
        ))}
      </div>
    </div>
  );
};

export default UserScans;
