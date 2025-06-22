import React, { useState, useEffect } from "react";
import {
  User,
  Mail,
  Phone,
  Bell,
  Save,
  ArrowLeft,
  CheckCircle,
  XCircle,
  Send,
  Shield,
} from "lucide-react";
import {
  UserData,
  UpdateProfileData,
  updateProfile,
  sendEmailVerification,
  verifyEmail,
  sendSmsVerification,
  verifySms,
} from "./UserProfileUtils";
import "./UserProfile.css";

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
  const [formData, setFormData] = useState<UpdateProfileData>({
    name: user.name,
    email: user.email,
    phone: user.phone,
    notification_preferences: { ...user.notification_preferences },
  });

  const [isLoading, setIsLoading] = useState(false);
  const [hasChanges, setHasChanges] = useState(false);
  const [verificationLoading, setVerificationLoading] = useState({
    email: false,
    sms: false,
  });
  const [showVerificationInput, setShowVerificationInput] = useState({
    email: false,
    sms: false,
  });
  const [verificationCodes, setVerificationCodes] = useState({
    email: "",
    sms: "",
  });

  // Track if email/phone changed to reset verification status
  const [emailChanged, setEmailChanged] = useState(false);
  const [phoneChanged, setPhoneChanged] = useState(false);

  useEffect(() => {
    setEmailChanged(formData.email !== user.email);
    setPhoneChanged(formData.phone !== user.phone);
  }, [formData.email, formData.phone, user.email, user.phone]);

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
      const updatedUser = await updateProfile(formData);
      onUserUpdate(updatedUser);
      setHasChanges(false);
      // Reset change tracking after successful save
      setEmailChanged(false);
      setPhoneChanged(false);
      alert("Profile updated successfully!");
    } catch (error) {
      console.error("Update error:", error);
      alert(
        error instanceof Error ? error.message : "Failed to update profile",
      );
    } finally {
      setIsLoading(false);
    }
  };

  const handleSendEmailVerification = async () => {
    setVerificationLoading((prev) => ({ ...prev, email: true }));
    try {
      await sendEmailVerification();
      setShowVerificationInput((prev) => ({ ...prev, email: true }));
      alert(
        "Verification email sent! Check your inbox and enter the code below.",
      );
    } catch (error) {
      console.error("Email verification error:", error);
      alert(
        error instanceof Error
          ? error.message
          : "Failed to send verification email",
      );
    } finally {
      setVerificationLoading((prev) => ({ ...prev, email: false }));
    }
  };

  const handleSendSmsVerification = async () => {
    setVerificationLoading((prev) => ({ ...prev, sms: true }));
    try {
      await sendSmsVerification();
      setShowVerificationInput((prev) => ({ ...prev, sms: true }));
      alert("Verification code sent to your phone! Enter the code below.");
    } catch (error) {
      console.error("SMS verification error:", error);
      alert(
        error instanceof Error
          ? error.message
          : "Failed to send verification SMS",
      );
    } finally {
      setVerificationLoading((prev) => ({ ...prev, sms: false }));
    }
  };

  const handleVerifyEmail = async () => {
    if (!verificationCodes.email || verificationCodes.email.length !== 6) {
      alert("Please enter a valid 6-digit verification code");
      return;
    }

    try {
      await verifyEmail(verificationCodes.email);
      const updatedUser = { ...user, email_verified: true };
      onUserUpdate(updatedUser);
      setShowVerificationInput((prev) => ({ ...prev, email: false }));
      setVerificationCodes((prev) => ({ ...prev, email: "" }));
      alert("Email verified successfully! 🎉");
    } catch (error) {
      console.error("Email verification error:", error);
      alert(error instanceof Error ? error.message : "Failed to verify email");
    }
  };

  const handleVerifySms = async () => {
    if (!verificationCodes.sms || verificationCodes.sms.length !== 6) {
      alert("Please enter a valid 6-digit verification code");
      return;
    }

    try {
      await verifySms(verificationCodes.sms);
      const updatedUser = { ...user, phone_verified: true };
      onUserUpdate(updatedUser);
      setShowVerificationInput((prev) => ({ ...prev, sms: false }));
      setVerificationCodes((prev) => ({ ...prev, sms: "" }));
      alert("Phone number verified successfully! 🎉");
    } catch (error) {
      console.error("SMS verification error:", error);
      alert(
        error instanceof Error
          ? error.message
          : "Failed to verify phone number",
      );
    }
  };

  // Determine verification status considering changes
  const getEmailVerificationStatus = () => {
    if (emailChanged)
      return { verified: false, needsVerification: true, showAsChanged: true };
    return {
      verified: user.email_verified,
      needsVerification: !user.email_verified,
      showAsChanged: false,
    };
  };

  const getPhoneVerificationStatus = () => {
    if (phoneChanged)
      return { verified: false, needsVerification: true, showAsChanged: true };
    return {
      verified: user.phone_verified,
      needsVerification: !user.phone_verified,
      showAsChanged: false,
    };
  };

  const emailStatus = getEmailVerificationStatus();
  const phoneStatus = getPhoneVerificationStatus();

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
                  {emailStatus.verified ? (
                    <span className="verified">
                      <CheckCircle size={16} />
                      Verified
                    </span>
                  ) : (
                    <span className="unverified">
                      <XCircle size={16} />
                      {emailStatus.showAsChanged ? "Changed" : "Unverified"}
                    </span>
                  )}
                </div>
              </div>

              {/* Inline verification for email */}
              {emailStatus.needsVerification && (
                <div className="verification-inline">
                  <button
                    onClick={handleSendEmailVerification}
                    disabled={verificationLoading.email || emailChanged}
                    className="verification-button-small"
                    title={
                      emailChanged
                        ? "Save changes first"
                        : "Send verification code"
                    }
                  >
                    <Send size={14} />
                    {verificationLoading.email
                      ? "Sending..."
                      : "Send Verification"}
                  </button>

                  {emailChanged && (
                    <span className="change-note">
                      Save changes first to verify
                    </span>
                  )}
                </div>
              )}

              {showVerificationInput.email && (
                <div className="verification-input-inline">
                  <input
                    type="text"
                    value={verificationCodes.email}
                    onChange={(e) =>
                      setVerificationCodes((prev) => ({
                        ...prev,
                        email: e.target.value,
                      }))
                    }
                    placeholder="Enter 6-digit code"
                    maxLength={6}
                    className="verification-code-input-small"
                  />
                  <button
                    onClick={handleVerifyEmail}
                    className="verify-button-small"
                  >
                    Verify
                  </button>
                </div>
              )}
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
                  {phoneStatus.verified ? (
                    <span className="verified">
                      <CheckCircle size={16} />
                      Verified
                    </span>
                  ) : (
                    <span className="unverified">
                      <XCircle size={16} />
                      {phoneStatus.showAsChanged ? "Changed" : "Unverified"}
                    </span>
                  )}
                </div>
              </div>

              {/* Inline verification for phone */}
              {phoneStatus.needsVerification && (
                <div className="verification-inline">
                  <button
                    onClick={handleSendSmsVerification}
                    disabled={verificationLoading.sms || phoneChanged}
                    className="verification-button-small"
                    title={
                      phoneChanged
                        ? "Save changes first"
                        : "Send verification code"
                    }
                  >
                    <Send size={14} />
                    {verificationLoading.sms
                      ? "Sending..."
                      : "Send Verification"}
                  </button>

                  {phoneChanged && (
                    <span className="change-note">
                      Save changes first to verify
                    </span>
                  )}
                </div>
              )}

              {showVerificationInput.sms && (
                <div className="verification-input-inline">
                  <input
                    type="text"
                    value={verificationCodes.sms}
                    onChange={(e) =>
                      setVerificationCodes((prev) => ({
                        ...prev,
                        sms: e.target.value,
                      }))
                    }
                    placeholder="Enter 6-digit code"
                    maxLength={6}
                    className="verification-code-input-small"
                  />
                  <button
                    onClick={handleVerifySms}
                    className="verify-button-small"
                  >
                    Verify
                  </button>
                </div>
              )}
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
                {!emailStatus.verified && (
                  <span className="verification-note">
                    (Email verification required)
                  </span>
                )}
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
                {!phoneStatus.verified && (
                  <span className="verification-note">
                    (Phone verification required)
                  </span>
                )}
              </span>
            </label>
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
