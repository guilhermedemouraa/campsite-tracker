import React from "react";
import "./MountainBackground.css";

const MountainBackground: React.FC = () => {
  return (
    <svg
      className="mountain-bg"
      viewBox="0 0 1200 800"
      xmlns="http://www.w3.org/2000/svg"
    >
      <defs>
        <linearGradient id="skyGradient" x1="0%" y1="0%" x2="0%" y2="100%">
          <stop offset="0%" stopColor="#2C3E50" />
          <stop offset="50%" stopColor="#34495E" />
          <stop offset="100%" stopColor="#4A6741" />
        </linearGradient>
        <linearGradient
          id="mountainGradient1"
          x1="0%"
          y1="0%"
          x2="0%"
          y2="100%"
        >
          <stop offset="0%" stopColor="#2C5530" />
          <stop offset="100%" stopColor="#1B3B36" />
        </linearGradient>
        <linearGradient
          id="mountainGradient2"
          x1="0%"
          y1="0%"
          x2="0%"
          y2="100%"
        >
          <stop offset="0%" stopColor="#4A6741" />
          <stop offset="100%" stopColor="#2C5530" />
        </linearGradient>
      </defs>

      {/* Sky */}
      <rect width="1200" height="800" fill="url(#skyGradient)" />

      {/* Mountains - Back Layer */}
      <path
        d="M0,600 L200,300 L400,450 L600,200 L800,350 L1000,250 L1200,400 L1200,800 L0,800 Z"
        fill="url(#mountainGradient1)"
        opacity="0.7"
      />

      {/* Mountains - Front Layer */}
      <path
        d="M0,700 L150,400 L300,550 L500,300 L700,450 L900,350 L1100,500 L1200,450 L1200,800 L0,800 Z"
        fill="url(#mountainGradient2)"
      />

      {/* Trees */}
      <circle cx="100" cy="650" r="25" fill="#1B3B36" opacity="0.8" />
      <circle cx="150" cy="670" r="20" fill="#2C5530" opacity="0.8" />
      <circle cx="250" cy="640" r="30" fill="#1B3B36" opacity="0.8" />
      <circle cx="900" cy="680" r="25" fill="#2C5530" opacity="0.8" />
      <circle cx="1050" cy="660" r="20" fill="#1B3B36" opacity="0.8" />
    </svg>
  );
};

export default MountainBackground;
