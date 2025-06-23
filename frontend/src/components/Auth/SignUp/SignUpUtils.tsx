export interface SignUpFormData {
  name: string;
  email: string;
  phone: string;
  password: string;
  confirmPassword: string;
  notifications: {
    email: boolean;
    sms: boolean;
  };
}

export interface SignUpRequest {
  name: string;
  email: string;
  phone: string;
  password: string;
  notification_preferences: {
    email: boolean;
    sms: boolean;
  };
}

export interface AuthResponse {
  access_token: string;
  refresh_token: string;
  user: {
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
  };
}

export const formatPhoneNumber = (phone: string): string => {
  // Remove all non-digit characters
  const digits = phone.replace(/\D/g, "");

  // Add +1 if it's a 10-digit US number
  if (digits.length === 10) {
    return `+1${digits}`;
  } else if (digits.length === 11 && digits.startsWith("1")) {
    return `+${digits}`;
  }

  return `+${digits}`;
};

export const validateForm = (
  formData: SignUpFormData,
): Partial<SignUpFormData> => {
  const errors: Partial<SignUpFormData> = {};

  // Name validation
  if (!formData.name.trim()) {
    errors.name = "Name is required";
  }

  // Email validation
  const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
  if (!formData.email) {
    errors.email = "Email is required";
  } else if (!emailRegex.test(formData.email)) {
    errors.email = "Please enter a valid email";
  }

  // Phone validation (basic US format)
  const phoneRegex =
    /^\+?1?[-.\s]?\(?([0-9]{3})\)?[-.\s]?([0-9]{3})[-.\s]?([0-9]{4})$/;
  if (!formData.phone) {
    errors.phone = "Phone number is required";
  } else if (!phoneRegex.test(formData.phone)) {
    errors.phone = "Please enter a valid US phone number";
  }

  // Password validation
  if (!formData.password) {
    errors.password = "Password is required";
  } else if (formData.password.length < 8) {
    errors.password = "Password must be at least 8 characters";
  }

  // Confirm password validation
  if (formData.password !== formData.confirmPassword) {
    errors.confirmPassword = "Passwords do not match";
  }

  return errors;
};

export const signUpUser = async (
  formData: SignUpFormData,
): Promise<AuthResponse> => {
  const signUpData: SignUpRequest = {
    name: formData.name.trim(),
    email: formData.email.toLowerCase().trim(),
    phone: formatPhoneNumber(formData.phone),
    password: formData.password,
    notification_preferences: formData.notifications,
  };

  console.log("Signing up user:", {
    ...signUpData,
    password: "[REDACTED]",
  });

  const response = await fetch("/api/auth/signup", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(signUpData),
  });

  if (!response.ok) {
    const errorData = await response.json();
    throw new Error(errorData.message || "Sign up failed");
  }

  const result = await response.json();
  console.log("Sign up successful:", result);

  return result;
};
