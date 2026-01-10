import React, { useState, useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";
import { LogOut } from "lucide-react";
import Clock from "./Clock"; // Import the Clock component
import TodayStatus from "./TodayStatus"; // Import the TodayStatus component
import "./Dashboard.css";

const Dashboard = () => {
  const navigate = useNavigate();
  const [user, setUser] = useState(null);
  const [sidebarOpen, setSidebarOpen] = useState(true);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const storedUser =
      localStorage.getItem("user") || sessionStorage.getItem("user");

    if (!storedUser) {
      navigate("/");
      return;
    }

    try {
      const userData = JSON.parse(storedUser);
      setUser(userData);
    } catch (err) {
      console.error("Error parsing user data:", err);
      navigate("/");
    } finally {
      setLoading(false);
    }
  }, [navigate]);

  const handleLogout = async () => {
    try {
      await invoke("logout");
      localStorage.removeItem("user");
      localStorage.removeItem("rememberMe");
      sessionStorage.removeItem("user");
      navigate("/");
    } catch (err) {
      console.error("Logout error:", err);
    }
  };

  if (loading) {
    return (
      <div className="loading-container">
        <div className="spinner"></div>
        <p>Loading...</p>
      </div>
    );
  }

  if (!user) return null;

  return (
    <div className="dashboard-container">
      <main className="main-content">
        <Clock />
        {/* <TodayStatus /> */}
        {/* <button onClick={handleLogout} className="logout-btn">
          <LogOut size={20} />
          {sidebarOpen && <span>Logout</span>}
        </button> */}
      </main>
    </div>
  );
};

export default Dashboard;
