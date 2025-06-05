import React from "react";
import { render, screen } from "@testing-library/react";
import App from "./App";

test("renders campsite tracker app", () => {
  render(<App />);
  const heroTitle = screen.getByText(/Never Miss a Campsite/i);
  expect(heroTitle).toBeInTheDocument();
});

test("renders sign up button", () => {
  render(<App />);
  const signUpButton = screen.getByText(/Sign Up/i);
  expect(signUpButton).toBeInTheDocument();
});

test("renders create scan form", () => {
  render(<App />);
  const createScanTitle = screen.getByText(/Create a New Scan/i);
  expect(createScanTitle).toBeInTheDocument();
});
