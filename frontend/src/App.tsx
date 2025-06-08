import React, { useState, useEffect } from "react";
import Header from "./components/Header/Header";
import MountainBackground from "./components/MountainBackground/MountainBackground";
import CreateScan from "./components/CreateScan/CreateScan";
import UserProfile from "./components/UserProfile/UserProfile";
import "./App.css";

interface User {
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

function App() {
  const [user, setUser] = useState<User | null>(null);
  const [currentView, setCurrentView] = useState<"dashboard" | "profile">(
    "dashboard",
  );
  const [isLoading, setIsLoading] = useState(true);

  // Check for existing session on app load
  useEffect(() => {
    const checkExistingSession = async () => {
      const token = localStorage.getItem("access_token");
      if (token) {
        try {
          const response = await fetch("/api/user/profile", {
            headers: {
              Authorization: `Bearer ${token}`,
            },
          });

          if (response.ok) {
            const userData = await response.json();
            setUser(userData);
          } else {
            // Token is invalid, remove it
            localStorage.removeItem("access_token");
            localStorage.removeItem("refresh_token");
          }
        } catch (error) {
          console.error("Session check failed:", error);
          localStorage.removeItem("access_token");
          localStorage.removeItem("refresh_token");
        }
      }
      setIsLoading(false);
    };

    checkExistingSession();
  }, []);

  const handleLogin = (userData: User, tokens: any) => {
    setUser(userData);
    setCurrentView("dashboard");
    console.log("User logged in:", userData);
  };

  const handleLogout = () => {
    setUser(null);
    setCurrentView("dashboard");
    console.log("User logged out");
  };

  const handleShowProfile = () => {
    setCurrentView("profile");
  };

  const handleBackToDashboard = () => {
    setCurrentView("dashboard");
  };

  const handleUserUpdate = (updatedUser: User) => {
    setUser(updatedUser);
  };

  if (isLoading) {
    // Loading screen
    return (
      <div className="app">
        <MountainBackground />
        <div className="loading-screen">
          <div className="loading-spinner">Loading...</div>
        </div>
      </div>
    );
  }

  return (
    <div className="app">
      {currentView === "profile" ? (
        // User Profile View
        <UserProfile
          user={user!}
          onBack={handleBackToDashboard}
          onUserUpdate={handleUserUpdate}
        />
      ) : (
        // Main Dashboard View
        <>
          <MountainBackground />
          <div className="app-content">
            <Header
              user={user}
              onLogin={handleLogin}
              onLogout={handleLogout}
              onShowProfile={handleShowProfile}
            />
            <main className="main">
              <div className="hero">
                <h1 className="hero-title">
                  {user
                    ? `Welcome back, ${user.name.split(" ")[0]}!`
                    : "Never Miss a Campsite"}
                </h1>
                <p className="hero-subtitle">
                  {user
                    ? "Manage your campsite scans and get notified when availability opens up"
                    : "Get notified instantly when your dream campsite becomes available"}
                </p>
              </div>
              <CreateScan />
            </main>
          </div>
        </>
      )}
    </div>
  );
}

export default App;
