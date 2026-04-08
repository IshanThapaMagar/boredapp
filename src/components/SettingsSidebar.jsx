import React from "react";
import { User as UserIcon, Calendar as CalendarIcon } from "lucide-react";
import "./SettingsSidebar.css";

const SettingsSidebar = ({ activeSection, onSelect }) => {
  const navItems = [
    { id: "profile", label: "Profile", icon: UserIcon },
    { id: "calendar", label: "Calendar Setting", icon: CalendarIcon },
  ];

  return (
    <aside className="settings-sidebar">
      <div className="sidebar-header">
        <h3>Settings</h3>
      </div>
      <nav className="sidebar-nav">
        {navItems.map((item) => {
          const Icon = item.icon;
          return (
            <button
              key={item.id}
              className={`sidebar-nav-item ${activeSection === item.id ? "active" : ""}`}
              onClick={() => onSelect(item.id)}
            >
              <Icon size={18} />
              <span>{item.label}</span>
            </button>
          );
        })}
      </nav>
    </aside>
  );
};

export default SettingsSidebar;
