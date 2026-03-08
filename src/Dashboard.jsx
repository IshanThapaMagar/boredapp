import React, { useState, useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";
import { LogOut } from "lucide-react";
import Clock from "./Clock";
import { AttendanceActions } from "./AttendanceActions";
import { HolidayForm } from "./HolidayForm";
import { Toaster } from "sonner";
import "./Dashboard.css";

const Dashboard = () => {
  const navigate = useNavigate();
  const [user, setUser] = useState(null);
  const [loading, setLoading] = useState(true);
  const [todayRecord, setTodayRecord] = useState(null);
  const [todayLeave, setTodayLeave] = useState(null);

  useEffect(() => {
    const storedUser =
      localStorage.getItem("user") || sessionStorage.getItem("user");

    if (!storedUser) {
      navigate("/");
      return;
    }

    const getDateString = (date) => {
      const d = date || new Date();
      return d.getFullYear() + '-' +
        (d.getMonth() + 1).toString().padStart(2, '0') + '-' +
        d.getDate().toString().padStart(2, '0');
    };

    const fetchTodayRecord = async (userId) => {
      try {
        const dateStr = getDateString();
        const record = await invoke("get_attendance_record", { userId, date: dateStr });
        if (record) {
          setTodayRecord(record);
        }
        const leaveRecord = await invoke("get_today_leave", { userId, date: dateStr });
        if (leaveRecord) {
          setTodayLeave(leaveRecord);
        }
      } catch (err) {
        console.error("Error fetching attendance/leave:", err);
      }
    };

    try {
      const userData = JSON.parse(storedUser);
      setUser(userData);
      fetchTodayRecord(userData.id);
    } catch (err) {
      console.error("Error parsing user data:", err);
      navigate("/");
    } finally {
      setLoading(false);
    }
  }, [navigate]);

  const saveRecord = async (record) => {
    setTodayRecord(record);
    if (user) {
      try {
        await invoke("save_attendance_record", { record });
      } catch (err) {
        console.error("Error saving attendance:", err);
      }
    }
  };

  const handleCheckIn = (manualTime, manualDate) => {
    const now = new Date();
    const timeStr = manualTime || now.getHours().toString().padStart(2, '0') + ':' +
      now.getMinutes().toString().padStart(2, '0');

    const dateStr = manualDate || now.getFullYear() + '-' +
      (now.getMonth() + 1).toString().padStart(2, '0') + '-' +
      now.getDate().toString().padStart(2, '0');

    const newRecord = {
      user_id: user.id,
      date: dateStr,
      check_in: timeStr,
      check_out: null,
      status: 'checked-in',
      overtime: 0,
      is_manual: !!manualTime
    };
    saveRecord(newRecord);
  };

  const calculateOvertime = (checkIn, checkOut) => {
    const [inH, inM] = checkIn.split(':').map(Number);
    const [outH, outM] = checkOut.split(':').map(Number);

    const durationMinutes = (outH * 60 + outM) - (inH * 60 + inM);
    const overtime = Math.max(0, durationMinutes - 480);
    return overtime;
  };

  const handleCheckOut = (manualTime, manualDate) => {
    if (!todayRecord && !manualDate) return;

    const now = new Date();
    const timeStr = manualTime || now.getHours().toString().padStart(2, '0') + ':' +
      now.getMinutes().toString().padStart(2, '0');

    const dateStr = manualDate || now.getFullYear() + '-' +
      (now.getMonth() + 1).toString().padStart(2, '0') + '-' +
      now.getDate().toString().padStart(2, '0');

    const updatedRecord = {
      user_id: user.id,
      date: dateStr,
      check_in: todayRecord?.check_in || null,
      check_out: timeStr,
      status: 'checked-out',
      overtime: calculateOvertime(todayRecord?.check_in || manualTime, timeStr),
      is_manual: todayRecord?.is_manual || !!manualTime
    };
    saveRecord(updatedRecord);
  };

  const handleManualLog = (manualData) => {
    const { date, checkIn, checkOut } = manualData;
    const record = {
      user_id: user.id,
      date: date,
      check_in: checkIn,
      check_out: checkOut,
      status: 'checked-out',
      overtime: calculateOvertime(checkIn, checkOut),
      is_manual: true
    };
    saveRecord(record);
  };

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
    <div className="dashboard-container relative">
      <Toaster position="bottom-right" theme="dark" />
      <main className="main-content">
        <div className="dashboard-header-section p-8">
          <Clock />
        </div>

        <div className="dashboard-content">
          <div className="attendance-container">
            <AttendanceActions
              todayRecord={todayRecord}
              todayLeave={todayLeave}
              onCheckIn={handleCheckIn}
              onCheckOut={handleCheckOut}
              onManualLog={handleManualLog}
            />
          </div>

          <div className="holiday-form-container">
            <HolidayForm userId={user.id} />
          </div>
        </div>
      </main>
    </div>
  );
};

export default Dashboard;
