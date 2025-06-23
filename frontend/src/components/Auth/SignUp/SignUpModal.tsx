import React, { useState } from "react";
import { X, User, Mail, Phone, Lock, Eye, EyeOff } from "lucide-react";
import {
  SignUpFormData,
  validateForm,
  signUpUser,
  AuthResponse,
} from "./SignUpUtils";
import styles from "./SignUpModal.module.css";

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
    <div className={styles.modalOverlay} onClick={onClose}>
      <div className={styles.modalContent} onClick={(e) => e.stopPropagation()}>
        <div className={styles.modalHeader}>
          <h2 className={styles.modalTitle}>Create Your Account</h2>
          <button className={styles.modalClose} onClick={onClose}>
            <X size={24} />
          </button>
        </div>

        <form onSubmit={handleSubmit} className={styles.modalForm}>
          {/* Name Field */}
          <div className={styles.formGroup}>
            <label className={styles.formLabel}>Full Name</label>
            <div className={styles.inputWrapper}>
              <User className={styles.inputIcon} size={20} />
              <input
                type="text"
                value={formData.name}
                onChange={(e) => handleInputChange("name", e.target.value)}
                placeholder="Enter your full name"
                className={`${styles.formInput} ${errors.name ? styles.error : ""}`}
                disabled={isLoading}
              />
            </div>
            {errors.name && (
              <span className={styles.errorMessage}>{errors.name}</span>
            )}
          </div>

          {/* Email Field */}
          <div className={styles.formGroup}>
            <label className={styles.formLabel}>Email Address</label>
            <div className={styles.inputWrapper}>
              <Mail className={styles.inputIcon} size={20} />
              <input
                type="email"
                value={formData.email}
                onChange={(e) => handleInputChange("email", e.target.value)}
                placeholder="Enter your email"
                className={`${styles.formInput} ${errors.email ? styles.error : ""}`}
                disabled={isLoading}
              />
            </div>
            {errors.email && (
              <span className={styles.errorMessage}>{errors.email}</span>
            )}
          </div>

          {/* Phone Field */}
          <div className={styles.formGroup}>
            <label className={styles.formLabel}>Phone Number</label>
            <div className={styles.inputWrapper}>
              <Phone className={styles.inputIcon} size={20} />
              <input
                type="tel"
                value={formData.phone}
                onChange={(e) => handleInputChange("phone", e.target.value)}
                placeholder="(555) 123-4567"
                className={`${styles.formInput} ${errors.phone ? styles.error : ""}`}
                disabled={isLoading}
              />
            </div>
            {errors.phone && (
              <span className={styles.errorMessage}>{errors.phone}</span>
            )}
            <div className={styles.fieldHelp}>
              We'll use this to send you campsite alerts
            </div>
          </div>

          {/* Password Field */}
          <div className={styles.formGroup}>
            <label className={styles.formLabel}>Password</label>
            <div className={styles.inputWrapper}>
              <Lock className={styles.inputIcon} size={20} />
              <input
                type={showPassword ? "text" : "password"}
                value={formData.password}
                onChange={(e) => handleInputChange("password", e.target.value)}
                placeholder="Create a password"
                className={`${styles.formInput} ${errors.password ? styles.error : ""}`}
                disabled={isLoading}
              />
              <button
                type="button"
                className={styles.passwordToggle}
                onClick={() => setShowPassword(!showPassword)}
              >
                {showPassword ? <EyeOff size={20} /> : <Eye size={20} />}
              </button>
            </div>
            {errors.password && (
              <span className={styles.errorMessage}>{errors.password}</span>
            )}
          </div>

          {/* Confirm Password Field */}
          <div className={styles.formGroup}>
            <label className={styles.formLabel}>Confirm Password</label>
            <div className={styles.inputWrapper}>
              <Lock className={styles.inputIcon} size={20} />
              <input
                type={showConfirmPassword ? "text" : "password"}
                value={formData.confirmPassword}
                onChange={(e) =>
                  handleInputChange("confirmPassword", e.target.value)
                }
                placeholder="Confirm your password"
                className={`${styles.formInput} ${errors.confirmPassword ? styles.error : ""}`}
                disabled={isLoading}
              />
              <button
                type="button"
                className={styles.passwordToggle}
                onClick={() => setShowConfirmPassword(!showConfirmPassword)}
              >
                {showConfirmPassword ? <EyeOff size={20} /> : <Eye size={20} />}
              </button>
            </div>
            {errors.confirmPassword && (
              <span className={styles.errorMessage}>
                {errors.confirmPassword}
              </span>
            )}
          </div>

          {/* Notification Preferences */}
          <div className={styles.formGroup}>
            <label className={styles.formLabel}>Notification Preferences</label>
            <div className={styles.checkboxGroup}>
              <label className={styles.checkboxLabel}>
                <input
                  type="checkbox"
                  checked={formData.notifications.email}
                  onChange={(e) =>
                    handleNotificationChange("email", e.target.checked)
                  }
                  disabled={isLoading}
                />
                <span className={styles.checkboxText}>Email notifications</span>
              </label>
              <label className={styles.checkboxLabel}>
                <input
                  type="checkbox"
                  checked={formData.notifications.sms}
                  onChange={(e) =>
                    handleNotificationChange("sms", e.target.checked)
                  }
                  disabled={isLoading}
                />
                <span className={styles.checkboxText}>SMS notifications</span>
              </label>
            </div>
          </div>

          {/* Submit Button */}
          <button
            type="submit"
            className={`${styles.submitButton} ${isLoading ? styles.loading : ""}`}
            disabled={isLoading}
          >
            {isLoading ? "Creating Account..." : "Create Account"}
          </button>
        </form>

        <div className={styles.modalFooter}>
          <p>
            Already have an account?{" "}
            <button
              className={styles.linkButton}
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
