// UserProfileUtils.tsx - API utilities for user profile operations

export interface UserData {
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

export interface UpdateProfileData {
  name: string;
  email: string;
  phone: string;
  notification_preferences: {
    email: boolean;
    sms: boolean;
  };
}

// API utility functions
const getAuthHeaders = () => {
  const token = localStorage.getItem("access_token");
  return {
    Authorization: `Bearer ${token}`,
    "Content-Type": "application/json",
  };
};

export const updateProfile = async (
  profileData: UpdateProfileData,
): Promise<UserData> => {
  const response = await fetch("/api/user/profile/update", {
    method: "PUT",
    headers: getAuthHeaders(),
    body: JSON.stringify(profileData),
  });

  if (!response.ok) {
    const errorData = await response.json();
    throw new Error(errorData.message || "Failed to update profile");
  }

  return response.json();
};

export const sendEmailVerification = async (): Promise<{ message: string }> => {
  const response = await fetch("/api/user/verify/email/send", {
    method: "POST",
    headers: getAuthHeaders(),
  });

  if (!response.ok) {
    const errorData = await response.json();
    throw new Error(errorData.message || "Failed to send verification email");
  }

  return response.json();
};

export const sendSmsVerification = async (): Promise<{ message: string }> => {
  const response = await fetch("/api/user/verify/sms/send", {
    method: "POST",
    headers: getAuthHeaders(),
  });

  if (!response.ok) {
    const errorData = await response.json();
    throw new Error(errorData.message || "Failed to send verification SMS");
  }

  return response.json();
};

export const verifySms = async (code: string): Promise<{ message: string }> => {
  const response = await fetch("/api/user/verify/sms", {
    method: "POST",
    headers: getAuthHeaders(),
    body: JSON.stringify({ code }),
  });

  if (!response.ok) {
    const errorData = await response.json();
    throw new Error(errorData.message || "Invalid verification code");
  }

  return response.json();
};
