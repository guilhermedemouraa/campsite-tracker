import React, { useState } from "react";
import { X, User, Mail, Phone, Lock, Eye, EyeOff } from "lucide-react";
import {
  SignUpFormData,
  validateForm,
  signUpUser,
  AuthResponse,
} from "./SignUpUtils";
import "./SignUpModal.css";

interface SignUpModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSuccess: (user: any, tokens: any) => void;
  onSwitchToLogin?: () => void;
}

const SignUpModal: React.FC<SignUpModalProps> = ({
  isOpen,
  onClose,
  onSuccess,
  onSwitchToLogin,
}) => {
  const [formData, setFormData] = useState<SignUpFormData>({
    name: "",
    email: "",
    phone: "",
    password: "",
    confirmPassword: "",
    notifications: {
      email: true,
      sms: true,
    },
  });

  const [showPassword, setShowPassword] = useState(false);
  const [showConfirmPassword, setShowConfirmPassword] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [errors, setErrors] = useState<Partial<SignUpFormData>>({});

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    const formErrors = validateForm(formData);
    setErrors(formErrors);

    if (Object.keys(formErrors).length > 0) {
      return;
    }

    setIsLoading(true);

    try {
      const result: AuthResponse = await signUpUser(formData);

      // Store tokens in localStorage
      localStorage.setItem("access_token", result.access_token);
      localStorage.setItem("refresh_token", result.refresh_token);

      // Call onSuccess with user data and tokens
      onSuccess(result.user, {
        access_token: result.access_token,
        refresh_token: result.refresh_token,
      });

      onClose();

      // Show success message with verification prompt
      if (!result.user.email_verified) {
        alert(
          "Account created successfully! 🎉\n\n" +
            "A verification email has been sent to your email address. " +
            "Please check your inbox and verify your email to enable notifications.",
        );
      } else {
        alert("Account created successfully! Welcome to CampTracker!");
      }
    } catch (error) {
      console.error("Sign up error:", error);
      alert(
        error instanceof Error
          ? error.message
          : "Sign up failed. Please try again.",
      );
    } finally {
      setIsLoading(false);
    }
  };

  const handleInputChange = (
    field: keyof SignUpFormData,
    value: string | boolean,
  ) => {
    if (field === "notifications") {
      return;
    }

    setFormData((prev) => ({
      ...prev,
      [field]: value,
    }));

    // Clear error when user starts typing
    if (errors[field]) {
      setErrors((prev) => ({
        ...prev,
        [field]: undefined,
      }));
    }
  };

  const handleNotificationChange = (type: "email" | "sms", value: boolean) => {
    setFormData((prev) => ({
      ...prev,
      notifications: {
        ...prev.notifications,
        [type]: value,
      },
    }));
  };

  if (!isOpen) return null;

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal-content" onClick={(e) => e.stopPropagation()}>
        <div className="modal-header">
          <h2 className="modal-title">Create Your Account</h2>
          <button className="modal-close" onClick={onClose}>
            <X size={24} />
          </button>
        </div>

        <form onSubmit={handleSubmit} className="modal-form">
          {/* Name Field */}
          <div className="form-group">
            <label className="form-label">Full Name</label>
            <div className="input-wrapper">
              <User className="input-icon" size={20} />
              <input
                type="text"
                value={formData.name}
                onChange={(e) => handleInputChange("name", e.target.value)}
                placeholder="Enter your full name"
                className={`form-input ${errors.name ? "error" : ""}`}
                disabled={isLoading}
              />
            </div>
            {errors.name && (
              <span className="error-message">{errors.name}</span>
            )}
          </div>

          {/* Email Field */}
          <div className="form-group">
            <label className="form-label">Email Address</label>
            <div className="input-wrapper">
              <Mail className="input-icon" size={20} />
              <input
                type="email"
                value={formData.email}
                onChange={(e) => handleInputChange("email", e.target.value)}
                placeholder="Enter your email"
                className={`form-input ${errors.email ? "error" : ""}`}
                disabled={isLoading}
              />
            </div>
            {errors.email && (
              <span className="error-message">{errors.email}</span>
            )}
          </div>

          {/* Phone Field */}
          <div className="form-group">
            <label className="form-label">Phone Number</label>
            <div className="input-wrapper">
              <Phone className="input-icon" size={20} />
              <input
                type="tel"
                value={formData.phone}
                onChange={(e) => handleInputChange("phone", e.target.value)}
                placeholder="(555) 123-4567"
                className={`form-input ${errors.phone ? "error" : ""}`}
                disabled={isLoading}
              />
            </div>
            {errors.phone && (
              <span className="error-message">{errors.phone}</span>
            )}
            <div className="field-help">
              We'll use this to send you campsite alerts
            </div>
          </div>

          {/* Password Field */}
          <div className="form-group">
            <label className="form-label">Password</label>
            <div className="input-wrapper">
              <Lock className="input-icon" size={20} />
              <input
                type={showPassword ? "text" : "password"}
                value={formData.password}
                onChange={(e) => handleInputChange("password", e.target.value)}
                placeholder="Create a password"
                className={`form-input ${errors.password ? "error" : ""}`}
                disabled={isLoading}
              />
              <button
                type="button"
                className="password-toggle"
                onClick={() => setShowPassword(!showPassword)}
              >
                {showPassword ? <EyeOff size={20} /> : <Eye size={20} />}
              </button>
            </div>
            {errors.password && (
              <span className="error-message">{errors.password}</span>
            )}
          </div>

          {/* Confirm Password Field */}
          <div className="form-group">
            <label className="form-label">Confirm Password</label>
            <div className="input-wrapper">
              <Lock className="input-icon" size={20} />
              <input
                type={showConfirmPassword ? "text" : "password"}
                value={formData.confirmPassword}
                onChange={(e) =>
                  handleInputChange("confirmPassword", e.target.value)
                }
                placeholder="Confirm your password"
                className={`form-input ${errors.confirmPassword ? "error" : ""}`}
                disabled={isLoading}
              />
              <button
                type="button"
                className="password-toggle"
                onClick={() => setShowConfirmPassword(!showConfirmPassword)}
              >
                {showConfirmPassword ? <EyeOff size={20} /> : <Eye size={20} />}
              </button>
            </div>
            {errors.confirmPassword && (
              <span className="error-message">{errors.confirmPassword}</span>
            )}
          </div>

          {/* Notification Preferences */}
          <div className="form-group">
            <label className="form-label">Notification Preferences</label>
            <div className="checkbox-group">
              <label className="checkbox-label">
                <input
                  type="checkbox"
                  checked={formData.notifications.email}
                  onChange={(e) =>
                    handleNotificationChange("email", e.target.checked)
                  }
                  disabled={isLoading}
                />
                <span className="checkbox-text">Email notifications</span>
              </label>
              <label className="checkbox-label">
                <input
                  type="checkbox"
                  checked={formData.notifications.sms}
                  onChange={(e) =>
                    handleNotificationChange("sms", e.target.checked)
                  }
                  disabled={isLoading}
                />
                <span className="checkbox-text">SMS notifications</span>
              </label>
            </div>
          </div>

          {/* Submit Button */}
          <button
            type="submit"
            className={`submit-button ${isLoading ? "loading" : ""}`}
            disabled={isLoading}
          >
            {isLoading ? "Creating Account..." : "Create Account"}
          </button>
        </form>

        <div className="modal-footer">
          <p>
            Already have an account?{" "}
            <button
              className="link-button"
              onClick={onSwitchToLogin || onClose}
            >
              Sign in instead
            </button>
          </p>
        </div>
      </div>
    </div>
  );
};

export default SignUpModal;
