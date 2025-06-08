import React, { useState } from "react";
import {
  User,
  Mail,
  Phone,
  Bell,
  Save,
  ArrowLeft,
  CheckCircle,
  XCircle,
} from "lucide-react";
import "./UserProfile.css";

interface UserData {
  id: string;
  name: string;
  email: string;
  phone: string;
  email_verified: boolean;
  phone_verified: boolean;
  notification_preferences: {
    email: boolean;
    sms: boolean;
  };
}

interface UserProfileProps {
  user: UserData;
  onBack: () => void;
  onUserUpdate: (updatedUser: UserData) => void;
}

const UserProfile: React.FC<UserProfileProps> = ({
  user,
  onBack,
  onUserUpdate,
}) => {
  const [formData, setFormData] = useState({
    name: user.name,
    email: user.email,
    phone: user.phone,
    notification_preferences: { ...user.notification_preferences },
  });

  const [isLoading, setIsLoading] = useState(false);
  const [hasChanges, setHasChanges] = useState(false);

  const handleInputChange = (field: string, value: string | boolean) => {
    if (field.startsWith("notifications.")) {
      const notificationType = field.split(".")[1] as "email" | "sms";
      setFormData((prev) => ({
        ...prev,
        notification_preferences: {
          ...prev.notification_preferences,
          [notificationType]: value as boolean,
        },
      }));
    } else {
      setFormData((prev) => ({
        ...prev,
        [field]: value,
      }));
    }

    setHasChanges(true);
  };

  const handleSave = async () => {
    setIsLoading(true);
    try {
      const token = localStorage.getItem("access_token");
      const response = await fetch("/api/user/profile/update", {
        method: "PUT",
        headers: {
          "Content-Type": "application/json",
          Authorization: `Bearer ${token}`,
        },
        body: JSON.stringify(formData),
      });

      if (!response.ok) {
        throw new Error("Failed to update profile");
      }

      const updatedUser = await response.json();
      onUserUpdate(updatedUser);
      setHasChanges(false);
      alert("Profile updated successfully!");
    } catch (error) {
      console.error("Update error:", error);
      alert("Failed to update profile. Please try again.");
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="user-profile">
      <div className="profile-header">
        <button className="back-button" onClick={onBack}>
          <ArrowLeft size={20} />
          Back to Dashboard
        </button>
        <h1 className="profile-title">My Profile</h1>
      </div>

      <div className="profile-content">
        {/* User Info Section */}
        <div className="profile-section">
          <h2 className="section-title">
            <User size={20} />
            Personal Information
          </h2>

          <div className="form-grid">
            <div className="form-group">
              <label className="form-label">Full Name</label>
              <input
                type="text"
                value={formData.name}
                onChange={(e) => handleInputChange("name", e.target.value)}
                className="form-input"
                placeholder="Enter your full name"
              />
            </div>

            <div className="form-group">
              <label className="form-label">Email Address</label>
              <div className="input-with-status">
                <input
                  type="email"
                  value={formData.email}
                  onChange={(e) => handleInputChange("email", e.target.value)}
                  className="form-input"
                  placeholder="Enter your email"
                />
                <div className="verification-status">
                  {user.email_verified ? (
                    <span className="verified">
                      <CheckCircle size={16} />
                      Verified
                    </span>
                  ) : (
                    <span className="unverified">
                      <XCircle size={16} />
                      Unverified
                    </span>
                  )}
                </div>
              </div>
            </div>

            <div className="form-group">
              <label className="form-label">Phone Number</label>
              <div className="input-with-status">
                <input
                  type="tel"
                  value={formData.phone}
                  onChange={(e) => handleInputChange("phone", e.target.value)}
                  className="form-input"
                  placeholder="(555) 123-4567"
                />
                <div className="verification-status">
                  {user.phone_verified ? (
                    <span className="verified">
                      <CheckCircle size={16} />
                      Verified
                    </span>
                  ) : (
                    <span className="unverified">
                      <XCircle size={16} />
                      Unverified
                    </span>
                  )}
                </div>
              </div>
            </div>
          </div>
        </div>

        {/* Notification Preferences */}
        <div className="profile-section">
          <h2 className="section-title">
            <Bell size={20} />
            Notification Preferences
          </h2>

          <div className="notification-options">
            <label className="checkbox-label">
              <input
                type="checkbox"
                checked={formData.notification_preferences.email}
                onChange={(e) =>
                  handleInputChange("notifications.email", e.target.checked)
                }
              />
              <span className="checkbox-text">
                <Mail size={16} />
                Email notifications for campsite availability
              </span>
            </label>

            <label className="checkbox-label">
              <input
                type="checkbox"
                checked={formData.notification_preferences.sms}
                onChange={(e) =>
                  handleInputChange("notifications.sms", e.target.checked)
                }
              />
              <span className="checkbox-text">
                <Phone size={16} />
                SMS notifications for campsite availability
              </span>
            </label>
          </div>
        </div>

        {/* Account Stats */}
        <div className="profile-section">
          <h2 className="section-title">Account Status</h2>
          <div className="stats-grid">
            <div className="stat-card">
              <div className="stat-label">Email Status</div>
              <div
                className={`stat-value ${user.email_verified ? "verified" : "unverified"}`}
              >
                {user.email_verified ? "Verified" : "Pending Verification"}
              </div>
            </div>
            <div className="stat-card">
              <div className="stat-label">Phone Status</div>
              <div
                className={`stat-value ${user.phone_verified ? "verified" : "unverified"}`}
              >
                {user.phone_verified ? "Verified" : "Pending Verification"}
              </div>
            </div>
          </div>
        </div>

        {/* Save Button */}
        {hasChanges && (
          <div className="save-section">
            <button
              onClick={handleSave}
              disabled={isLoading}
              className={`save-button ${isLoading ? "loading" : ""}`}
            >
              <Save size={20} />
              {isLoading ? "Saving..." : "Save Changes"}
            </button>
          </div>
        )}
      </div>
    </div>
  );
};

export default UserProfile;
