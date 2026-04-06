import React from "react";
import Clock from "../Clock";
import "./Navbar.css";

const Navbar = () => {
  return (
    <nav className="navbar">
      <div className="navbar-content">
        <Clock />
      </div>
    </nav>
  );
};

export default Navbar;
