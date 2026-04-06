import React, { useState } from "react";
import {
  LayoutDashboard,
  User,
  Settings,
  Menu,
  LogOut,
  ChevronLeft,
  BarChart3,
} from "lucide-react";
import "./Sidebar.css";

const Sidebar = ({ onLogout, user, isOpen, onToggle }) => {
  const [activeItem, setActiveItem] = useState("dashboard");

  const navItems = [
    { id: "dashboard", label: "Dashboard", icon: LayoutDashboard },
    { id: "profile", label: "Profile", icon: User },
    { id: "reports", label: "Reports", icon: BarChart3 },
    { id: "settings", label: "Settings", icon: Settings },
  ];

  return (
    <aside className={`sidebar ${!isOpen ? "closed" : ""}`}>
      {/* Toggle Button */}
      <button
        onClick={onToggle}
        className="sidebar-toggle"
        title={isOpen ? "Collapse" : "Expand"}
        aria-label="Toggle sidebar"
      >
        {isOpen ? <ChevronLeft size={20} /> : <Menu size={20} />}
      </button>

      {/* Sidebar Navigation */}
      <nav className="sidebar-nav">
        {navItems.map((item) => {
          const Icon = item.icon;
          return (
            <div
              key={item.id}
              className={`nav-item ${activeItem === item.id ? "active" : ""}`}
              onClick={() => setActiveItem(item.id)}
              role="button"
              tabIndex={0}
              onKeyDown={(e) => {
                if (e.key === "Enter" || e.key === " ") {
                  setActiveItem(item.id);
                }
              }}
            >
              <Icon size={20} className="nav-icon" />
              {isOpen && <span className="nav-label">{item.label}</span>}
            </div>
          );
        })}
      </nav>

      {/* Sidebar Footer */}
      <div className="sidebar-footer">
        {isOpen && user && (
          <div className="user-info">
            <span className="user-name">
              {user.first_name || user.email}
            </span>
          </div>
        )}
        <button onClick={onLogout} className="logout-btn" title="Logout">
          <LogOut size={20} />
          {isOpen && <span>Logout</span>}
        </button>
      </div>
    </aside>
  );
};

export default Sidebar;
