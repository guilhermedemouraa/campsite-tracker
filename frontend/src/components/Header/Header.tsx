import React, { useState, useRef, useEffect } from "react";
import { User, MapPin, LogOut, Settings, ChevronDown } from "lucide-react";
import SignUpModal from "../Auth/SignUp/SignUpModal";
import LoginModal from "../Auth/Login/LoginModal";
import "./Header.css";

interface HeaderProps {
  user: any | null;
  onLogin: (user: any, tokens: any) => void;
  onLogout: () => void;
  onShowProfile: () => void;
}

const Header: React.FC<HeaderProps> = ({
  user,
  onLogin,
  onLogout,
  onShowProfile,
}) => {
  const [showSignUpModal, setShowSignUpModal] = useState(false);
  const [showLoginModal, setShowLoginModal] = useState(false);
  const [showUserMenu, setShowUserMenu] = useState(false);
  const dropdownRef = useRef<HTMLDivElement>(null);

  // Close dropdown when clicking outside
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (
        dropdownRef.current &&
        !dropdownRef.current.contains(event.target as Node)
      ) {
        setShowUserMenu(false);
      }
    };

    document.addEventListener("mousedown", handleClickOutside);
    return () => {
      document.removeEventListener("mousedown", handleClickOutside);
    };
  }, []);

  const handleSignIn = () => {
    setShowLoginModal(true);
  };

  const handleSignUp = () => {
    setShowSignUpModal(true);
  };

  const handleSignUpSuccess = (user: any, tokens: any) => {
    console.log("User signed up successfully!");
    onLogin(user, tokens);
    setShowSignUpModal(false);
  };

  const handleLoginSuccess = (user: any, tokens: any) => {
    console.log("User logged in successfully!");
    onLogin(user, tokens);
    setShowLoginModal(false);
  };

  const handleSwitchToSignUp = () => {
    setShowLoginModal(false);
    setShowSignUpModal(true);
  };

  const handleSwitchToLogin = () => {
    setShowSignUpModal(false);
    setShowLoginModal(true);
  };

  const handleLogout = () => {
    localStorage.removeItem("access_token");
    localStorage.removeItem("refresh_token");
    onLogout();
    setShowUserMenu(false);
  };

  const getInitials = (name: string) => {
    return name
      .split(" ")
      .map((n) => n[0])
      .join("")
      .toUpperCase();
  };

  return (
    <>
      <header className="header">
        <div className="header-content">
          <div className="logo">
            <MapPin className="logo-icon" />
            <span className="logo-text">CampTracker</span>
          </div>
          <nav className="nav">
            {user ? (
              // Authenticated user menu
              <div className="user-menu" ref={dropdownRef}>
                <button
                  className="user-button"
                  onClick={() => setShowUserMenu(!showUserMenu)}
                >
                  <div className="user-avatar">{getInitials(user.name)}</div>
                  <ChevronDown
                    size={16}
                    className={`chevron ${showUserMenu ? "rotated" : ""}`}
                  />
                </button>

                {showUserMenu && (
                  <div className="user-dropdown">
                    <div className="dropdown-menu">
                      <button
                        className="dropdown-item"
                        onClick={() => {
                          onShowProfile();
                          setShowUserMenu(false);
                        }}
                      >
                        <Settings size={16} />
                        Profile Settings
                      </button>
                      <button
                        className="dropdown-item logout"
                        onClick={handleLogout}
                      >
                        <LogOut size={16} />
                        Sign Out
                      </button>
                    </div>
                  </div>
                )}
              </div>
            ) : (
              // Guest user buttons
              <>
                <button className="nav-button" onClick={handleSignIn}>
                  <User size={20} />
                  Sign In
                </button>
                <button className="nav-button primary" onClick={handleSignUp}>
                  Sign Up
                </button>
              </>
            )}
          </nav>
        </div>
      </header>

      <SignUpModal
        isOpen={showSignUpModal}
        onClose={() => setShowSignUpModal(false)}
        onSuccess={handleSignUpSuccess}
        onSwitchToLogin={handleSwitchToLogin}
      />

      <LoginModal
        isOpen={showLoginModal}
        onClose={() => setShowLoginModal(false)}
        onSuccess={handleLoginSuccess}
        onSwitchToSignUp={handleSwitchToSignUp}
      />
    </>
  );
};

export default Header;
