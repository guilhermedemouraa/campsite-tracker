import React, { useState } from "react";
import { User, MapPin } from "lucide-react";
import SignUpModal from "../Auth/SignUpModal";
import "./Header.css";

const Header: React.FC = () => {
  const [showSignUpModal, setShowSignUpModal] = useState(false);

  const handleSignIn = () => {
    // TODO: Implement sign-in functionality
    console.log("Sign in clicked");
  };

  const handleSignUp = () => {
    setShowSignUpModal(true);
  };

  const handleSignUpSuccess = () => {
    console.log("User signed up successfully!");
    // TODO: Handle successful signup (maybe auto-login or redirect)
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
            <button className="nav-button" onClick={handleSignIn}>
              <User size={20} />
              Sign In
            </button>
            <button className="nav-button primary" onClick={handleSignUp}>
              Sign Up
            </button>
          </nav>
        </div>
      </header>

      <SignUpModal
        isOpen={showSignUpModal}
        onClose={() => setShowSignUpModal(false)}
        onSuccess={handleSignUpSuccess}
      />
    </>
  );
};

export default Header;
