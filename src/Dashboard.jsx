import React, { useState, useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";
import { LogOut } from "lucide-react";
import Clock from "./Clock";
import { AttendanceActions } from "./AttendanceActions";
import { HolidayForm } from "./HolidayForm";
import { Toaster } from "sonner";
import "./Dashboard.css";

import calendarData from "./lib/calendar_2080_2084.json";
import { ChevronLeft, ChevronRight, ChevronsLeft, ChevronsRight } from "lucide-react";

const nepaliMonths = [
  "Baishakh", "Jestha", "Ashadh", "Shrawan", "Bhadra", "Ashwin",
  "Kartik", "Mangshir", "Poush", "Magh", "Falgun", "Chaitra"
];

const nepaliMonthsUnicode = [
  "वैशाख", "जेठ", "असार", "साउन", "भदौ", "असोज",
  "कार्तिक", "मंसिर", "पुस", "माघ", "फागुन", "चैत"
];

const Calendar = () => {
  const [currentYear, setCurrentYear] = useState(2082);
  const [currentMonth, setCurrentMonth] = useState(11);

  useEffect(() => {
    const today = new Date();
    // Try multiple date formats to match JSON (some might be YYYY-M-D or YYYY-MM-DD)
    const adDateStr1 = `${today.getFullYear()}-${today.getMonth() + 1}-${today.getDate()}`;
    const adDateStr2 = `${today.getFullYear()}-${(today.getMonth() + 1).toString().padStart(2, '0')}-${today.getDate().toString().padStart(2, '0')}`;
    
    const todayEntry = calendarData.find(entry => entry.ad_date === adDateStr1 || entry.ad_date === adDateStr2);
    
    if (todayEntry) {
      const parts = todayEntry.bs_date.split('-');
      setCurrentYear(parseInt(parts[0]));
      setCurrentMonth(parseInt(parts[1]));
    }
  }, []);

  const monthStr = currentMonth.toString().padStart(2, '0');
  const monthPrefix = `${currentYear}-${monthStr}-`;
  const daysInMonth = calendarData.filter(d => d.bs_date.startsWith(monthPrefix));

  const years = [...new Set(calendarData.map(d => parseInt(d.bs_date.split('-')[0])))].sort();

  if (daysInMonth.length === 0) return <div>No data</div>;

  const startDayAD = daysInMonth[0].ad_date;
  const startWeekday = new Date(startDayAD).getDay();

  const adMonths = [...new Set(daysInMonth.map(d => {
    const date = new Date(d.ad_date);
    return date.toLocaleString('en-US', { month: 'short' });
  }))];
  const adYears = [...new Set(daysInMonth.map(d => new Date(d.ad_date).getFullYear()))];
  const adRangeStr = `${adMonths.join('/')} ${adYears.join('-')}`;

  const weekDays = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];

  const handlePrevMonth = () => {
    if (currentMonth === 1) {
      setCurrentMonth(12);
      setCurrentYear(prev => prev - 1);
    } else {
      setCurrentMonth(prev => prev - 1);
    }
  };

  const handleNextMonth = () => {
    if (currentMonth === 12) {
      setCurrentMonth(1);
      setCurrentYear(prev => prev + 1);
    } else {
      setCurrentMonth(prev => prev + 1);
    }
  };

  const handleGoToToday = () => {
    const today = new Date();
    const adDateStr = `${today.getFullYear()}-${today.getMonth() + 1}-${today.getDate()}`;
    const todayEntry = calendarData.find(entry => entry.ad_date === adDateStr);
    if (todayEntry) {
      const parts = todayEntry.bs_date.split('-');
      setCurrentYear(parseInt(parts[0]));
      setCurrentMonth(parseInt(parts[1]));
    }
  };

  return (
    <div className="calendar-card">
      <div className="calendar-header-nav">
        <div className="calendar-title">
          <span className="bs-title">{currentYear} {nepaliMonthsUnicode[currentMonth - 1]}</span>
          <span className="divider">|</span>
          <span className="ad-title">{adRangeStr}</span>
        </div>
        
        <div className="nav-controls">
          <div className="arrow-nav">
            <button onClick={() => setCurrentYear(prev => prev - 1)} title="Prev Year"><ChevronsLeft size={18} /></button>
            <button onClick={handlePrevMonth} title="Prev Month"><ChevronLeft size={18} /></button>
          </div>

          <div className="dropdown-nav">
            <select 
              value={currentMonth} 
              onChange={(e) => setCurrentMonth(parseInt(e.target.value))}
              className="calendar-select"
            >
              {nepaliMonths.map((m, i) => (
                <option key={m} value={i + 1}>{m}</option>
              ))}
            </select>
            <select 
              value={currentYear} 
              onChange={(e) => setCurrentYear(parseInt(e.target.value))}
              className="calendar-select"
            >
              {years.map(y => (
                <option key={y} value={y}>{y}</option>
              ))}
            </select>
          </div>

          <div className="arrow-nav">
            <button onClick={handleNextMonth} title="Next Month"><ChevronRight size={18} /></button>
            <button onClick={() => setCurrentYear(prev => prev + 1)} title="Next Year"><ChevronsRight size={18} /></button>
          </div>

          <button onClick={handleGoToToday} className="today-btn">Today</button>
        </div>
      </div>

      <div className="calendar-grid">
        {weekDays.map(d => (
          <div key={d} className="weekday-header">{d}</div>
        ))}
        {Array.from({ length: startWeekday }).map((_, i) => (
          <div key={`pad-${i}`} className="calendar-day padding"></div>
        ))}
        {daysInMonth.map(day => {
          const bsDay = day.bs_date.split('-').pop();
          const today = new Date();
          const isToday = day.ad_date === `${today.getFullYear()}-${today.getMonth() + 1}-${today.getDate()}`;
          
          return (
            <div key={day.bs_date} className={`calendar-day ${day.holiday ? 'holiday' : ''} ${isToday ? 'is-today' : ''}`}>
              <div className="day-top">
                <div className="day-bs-number">{bsDay}</div>
              </div>
              
              {day.event && day.event !== "--" && (
                <div className="day-event" title={day.event}>{day.event}</div>
              )}

              <div className="day-bottom">
                <div className="day-tithi">{day.tithi}</div>
                <div className="day-ad-info">{day.ad_date.split('-').pop()}</div>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
};

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
          <div className="left-side">
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

          <div className="right-side">
            <Calendar />
          </div>
        </div>
      </main>
    </div>
  );
};

export default Dashboard;
