import React, { useState, useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";
import {
  ChevronLeft,
  ChevronRight,
  ChevronsLeft,
  ChevronsRight,
  BarChart3,
  PieChart,
} from "lucide-react";
import Navbar from "./components/Navbar";
import Sidebar from "./components/Sidebar";
import { AttendanceActions } from "./AttendanceActions";
import AttendanceTablePage from "./AttendanceTablePage";
import SettingsPage from "./SettingsPage";
import { HolidayForm } from "./HolidayForm";
import { Toaster } from "sonner";
import "./Dashboard.css";

const nepaliMonths = [
  "Baishakh",
  "Jestha",
  "Ashadh",
  "Shrawan",
  "Bhadra",
  "Ashwin",
  "Kartik",
  "Mangshir",
  "Poush",
  "Magh",
  "Falgun",
  "Chaitra",
];

const nepaliMonthsUnicode = [
  "वैशाख",
  "जेठ",
  "असार",
  "साउन",
  "भदौ",
  "असोज",
  "कार्तिक",
  "मंसिर",
  "पुस",
  "माघ",
  "फागुन",
  "चैत",
];

const formatDateKey = (date) => {
  const year = date.getFullYear();
  const month = `${date.getMonth() + 1}`.padStart(2, "0");
  const day = `${date.getDate()}`.padStart(2, "0");
  return `${year}-${month}-${day}`;
};

const calculateHoursWorked = (checkIn, checkOut) => {
  if (!checkIn || !checkOut) return 0;

  const [inHours, inMinutes] = checkIn.split(":").map(Number);
  const [outHours, outMinutes] = checkOut.split(":").map(Number);
  const totalMinutes = outHours * 60 + outMinutes - (inHours * 60 + inMinutes);

  if (Number.isNaN(totalMinutes) || totalMinutes <= 0) return 0;

  return totalMinutes / 60;
};

const buildWeeklyAttendanceData = (attendanceRecords, leaveLogs) => {
  const today = new Date();
  const startDate = new Date(today);
  startDate.setDate(today.getDate() - 6);

  const attendanceMap = new Map(
    attendanceRecords.map((record) => [record.date, record]),
  );
  const leaveMap = new Map(leaveLogs.map((log) => [log.leave_date, log]));

  return Array.from({ length: 7 }, (_, index) => {
    const currentDate = new Date(startDate);
    currentDate.setDate(startDate.getDate() + index);
    const dateKey = formatDateKey(currentDate);
    const attendance = attendanceMap.get(dateKey);
    const leave = leaveMap.get(dateKey);
    const workedHours = attendance
      ? calculateHoursWorked(attendance.check_in, attendance.check_out)
      : 0;

    return {
      dateKey,
      label: currentDate.toLocaleDateString("en-US", { weekday: "short" }),
      day: currentDate.getDate(),
      workedHours,
      attendance,
      leave,
      status: leave
        ? leave.leave_type === "public_holiday"
          ? "Holiday"
          : "Leave"
        : attendance
          ? attendance.check_in && attendance.check_out
            ? "Present"
            : "Partial"
          : "Absent",
    };
  });
};

const buildLeaveDistribution = (leaveLogs) => {
  const distribution = leaveLogs.reduce((accumulator, log) => {
    const type = log.leave_type || "other";
    accumulator[type] = (accumulator[type] || 0) + 1;
    return accumulator;
  }, {});

  return [
    {
      key: "public_holiday",
      label: "Public Holiday",
      count: distribution.public_holiday || 0,
      color: "#2563eb",
    },
    {
      key: "absent",
      label: "Absent",
      count: distribution.absent || 0,
      color: "#ef4444",
    },
  ].filter((item) => item.count > 0);
};

const WeeklyAttendanceChart = ({ data }) => {
  const maxHours = Math.max(8, ...data.map((item) => item.workedHours));
  const totalHours = data.reduce((sum, item) => sum + item.workedHours, 0);
  const attendedDays = data.filter((item) => item.workedHours > 0).length;

  return (
    <section className="chart-card chart-card-attendance">
      <div className="chart-header">
        <div>
          <div className="chart-title-row">
            <BarChart3 size={18} />
            <h3>Weekly Attendance</h3>
          </div>
          <p>Last 7 days of recorded attendance</p>
        </div>
        <div className="chart-summary">
          <span>{attendedDays} active days</span>
          <strong>{totalHours.toFixed(1)} hrs</strong>
        </div>
      </div>

      <div className="attendance-chart">
        {data.map((item) => {
          const heightPercent = item.leave
            ? 100
            : Math.max(
                (item.workedHours / maxHours) * 100,
                item.workedHours > 0 ? 10 : 0,
              );

          return (
            <div
              key={item.dateKey}
              className="attendance-bar-group"
              title={`${item.label} ${item.day}: ${item.status}${item.workedHours > 0 ? ` - ${item.workedHours.toFixed(1)} hrs` : ""}`}
            >
              <div className="attendance-bar-track">
                <div
                  className={`attendance-bar-fill ${item.leave ? "leave" : item.workedHours > 0 ? "present" : "absent"}`}
                  style={{ height: `${heightPercent}%` }}
                />
              </div>
              <span className="attendance-bar-label">{item.label}</span>
              <span className="attendance-bar-meta">
                {item.leave ? item.status : `${item.workedHours.toFixed(1)}h`}
              </span>
            </div>
          );
        })}
      </div>

      <div className="chart-legend attendance-legend">
        <span>
          <i className="legend-swatch present" /> Present
        </span>
        <span>
          <i className="legend-swatch leave" /> Leave
        </span>
        <span>
          <i className="legend-swatch absent" /> No record
        </span>
      </div>
    </section>
  );
};

const LeaveDistributionChart = ({ data }) => {
  const totalLeaves = data.reduce((sum, item) => sum + item.count, 0);

  let runningTotal = 0;
  const segments = data.map((item) => {
    const start = runningTotal;
    runningTotal += item.count;
    return {
      ...item,
      start,
      percentage: totalLeaves ? (item.count / totalLeaves) * 100 : 0,
    };
  });

  const gradient = totalLeaves
    ? `conic-gradient(${segments
        .map((item) => {
          const start = (item.start / totalLeaves) * 100;
          const end = start + item.percentage;
          return `${item.color} ${start}% ${end}%`;
        })
        .join(", ")})`
    : "conic-gradient(#e5e7eb 0% 100%)";

  return (
    <section className="chart-card chart-card-leave">
      <div className="chart-header">
        <div>
          <div className="chart-title-row">
            <PieChart size={18} />
            <h3>Leave Distribution</h3>
          </div>
          <p>Saved leave logs grouped by type</p>
        </div>
        <div className="chart-summary">
          <span>Total logs</span>
          <strong>{totalLeaves}</strong>
        </div>
      </div>

      <div className="leave-chart-shell">
        <div className="donut-chart" style={{ background: gradient }}>
          <div className="donut-hole">
            <span>{totalLeaves}</span>
            <small>entries</small>
          </div>
        </div>

        <div className="distribution-list">
          {data.length > 0 ? (
            data.map((item) => (
              <div key={item.key} className="distribution-row">
                <span className="distribution-label">
                  <i
                    className="legend-swatch"
                    style={{ backgroundColor: item.color }}
                  />
                  {item.label}
                </span>
                <strong>{item.count}</strong>
              </div>
            ))
          ) : (
            <div className="empty-chart-state">No leave records yet.</div>
          )}
        </div>
      </div>
    </section>
  );
};

const Calendar = () => {
  const [currentYear, setCurrentYear] = useState(2082);
  const [currentMonth, setCurrentMonth] = useState(11);

  const [calendarData, setCalendarData] = useState([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const fetchCalendarData = async () => {
      try {
        const data = await invoke("get_calendar_data", {
          startDate: "2023-01-01",
          endDate: "2030-12-31",
        });
        setCalendarData(data);
      } catch (err) {
        console.error("Error fetching calendar data:", err);
        setCalendarData([]);
      } finally {
        setLoading(false);
      }
    };
    fetchCalendarData();
  }, []);

  const normalizeDate = (dateStr) => {
    if (!dateStr) return "";
    let normalized = dateStr.replace(/[\u0966-\u096F]/g, (d) => {
      return (d.charCodeAt(0) - 0x0966).toString();
    });

    // Ensure YYYY-MM-DD format
    const parts = normalized.split("-");
    if (parts.length !== 3) return normalized;
    return `${parts[0]}-${parts[1].padStart(2, "0")}-${parts[2].padStart(2, "0")}`;
  };

  useEffect(() => {
    if (calendarData.length === 0) return;

    const today = new Date();
    const adDateStr = `${today.getFullYear()}-${(today.getMonth() + 1).toString().padStart(2, "0")}-${today.getDate().toString().padStart(2, "0")}`;

    // Normalize both for comparison
    const todayEntry = calendarData.find(
      (entry) => normalizeDate(entry.ad_date) === adDateStr,
    );

    if (todayEntry) {
      // Use normalized version of bs_date for parsing
      const parts = normalizeDate(todayEntry.bs_date).split("-");
      setCurrentYear(parseInt(parts[0]));
      setCurrentMonth(parseInt(parts[1]));
    }
  }, [calendarData]);

  const monthStr = currentMonth.toString().padStart(2, "0");
  const monthPrefix = `${currentYear}-${monthStr}-`;
  const daysInMonth = calendarData.filter((d) =>
    normalizeDate(d.bs_date).startsWith(monthPrefix),
  );

  const years = [
    ...new Set(
      calendarData.map((d) => parseInt(normalizeDate(d.bs_date).split("-")[0])),
    ),
  ].sort();

  if (loading)
    return (
      <div className="calendar-card">
        <div className="p-4">Loading Calendar...</div>
      </div>
    );
  if (calendarData.length === 0)
    return (
      <div className="calendar-card">
        <div className="p-4">
          No calendar data found. (Check database/Backend)
        </div>
      </div>
    );
  if (daysInMonth.length === 0)
    return (
      <div className="calendar-card">
        <div className="p-4">
          No records found for {currentYear}-{currentMonth}. (Total Data:{" "}
          {calendarData.length})
        </div>
      </div>
    );

  const startDayAD = daysInMonth[0].ad_date;
  const startWeekday = new Date(startDayAD).getDay();

  const adMonths = [
    ...new Set(
      daysInMonth.map((d) => {
        const date = new Date(d.ad_date);
        return date.toLocaleString("en-US", { month: "short" });
      }),
    ),
  ];
  const adYears = [
    ...new Set(daysInMonth.map((d) => new Date(d.ad_date).getFullYear())),
  ];
  const adRangeStr = `${adMonths.join("/")} ${adYears.join("-")}`;

  const weekDays = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];

  const handlePrevMonth = () => {
    if (currentMonth === 1) {
      setCurrentMonth(12);
      setCurrentYear((prev) => prev - 1);
    } else {
      setCurrentMonth((prev) => prev - 1);
    }
  };

  const handleNextMonth = () => {
    if (currentMonth === 12) {
      setCurrentMonth(1);
      setCurrentYear((prev) => prev + 1);
    } else {
      setCurrentMonth((prev) => prev + 1);
    }
  };

  const handleGoToToday = () => {
    const today = new Date();
    const adDateStr = `${today.getFullYear()}-${(today.getMonth() + 1).toString().padStart(2, "0")}-${today.getDate().toString().padStart(2, "0")}`;

    const todayEntry = calendarData.find(
      (entry) => normalizeDate(entry.ad_date) === adDateStr,
    );
    if (todayEntry) {
      const parts = normalizeDate(todayEntry.bs_date).split("-");
      setCurrentYear(parseInt(parts[0]));
      setCurrentMonth(parseInt(parts[1]));
    }
  };

  return (
    <div className="calendar-card">
      <div className="p-4 border-b border-[#e5e7eb]">
        <div className="calendar-info-row">
          <div className="calendar-title">
            <span className="bs-title">
              {currentYear} {nepaliMonthsUnicode[currentMonth - 1]}
            </span>
            <span className="divider">|</span>
            <span className="ad-title">{adRangeStr}</span>
          </div>
        </div>

        <div className="nav-controls">
          <div className="nav-group">
            <button
              onClick={() => setCurrentYear((prev) => prev - 1)}
              className="nav-icon-btn"
              title="Prev Year"
            >
              <ChevronsLeft size={16} />
            </button>
            <button
              onClick={handlePrevMonth}
              className="nav-icon-btn"
              title="Prev Month"
            >
              <ChevronLeft size={16} />
            </button>

            <select
              value={currentMonth}
              onChange={(e) => setCurrentMonth(parseInt(e.target.value))}
              className="calendar-select"
            >
              {nepaliMonths.map((m, i) => (
                <option key={m} value={i + 1}>
                  {m}
                </option>
              ))}
            </select>

            <select
              value={currentYear}
              onChange={(e) => setCurrentYear(parseInt(e.target.value))}
              className="calendar-select"
            >
              {years.map((y) => (
                <option key={y} value={y}>
                  {y}
                </option>
              ))}
            </select>

            <button
              onClick={handleNextMonth}
              className="nav-icon-btn"
              title="Next Month"
            >
              <ChevronRight size={16} />
            </button>
            <button
              onClick={() => setCurrentYear((prev) => prev + 1)}
              className="nav-icon-btn"
              title="Next Year"
            >
              <ChevronsRight size={16} />
            </button>
          </div>

          <button onClick={handleGoToToday} className="today-btn">
            Today
          </button>
        </div>
      </div>

      <div className="calendar-grid">
        {weekDays.map((d) => (
          <div key={d} className="weekday-header">
            {d}
          </div>
        ))}
        {Array.from({ length: startWeekday }).map((_, i) => (
          <div key={`pad-${i}`} className="calendar-day padding"></div>
        ))}
        {daysInMonth.map((day) => {
          const bsDayRaw = day.bs_date.split("-").pop();
          const bsDay = normalizeDate(day.bs_date).split("-").pop();
          const today = new Date();
          const adDateStr = `${today.getFullYear()}-${(today.getMonth() + 1).toString().padStart(2, "0")}-${today.getDate().toString().padStart(2, "0")}`;
          const isToday = normalizeDate(day.ad_date) === adDateStr;

          return (
            <div
              key={day.bs_date}
              className={`calendar-day ${day.holiday ? "holiday" : ""} ${isToday ? "is-today" : ""}`}
            >
              <div className="day-top">
                <div className="day-bs-number">{bsDay}</div>
              </div>

              {day.event && day.event !== "--" && (
                <div className="day-event" title={day.event}>
                  {day.event}
                </div>
              )}

              <div className="day-bottom">
                <div className="day-tithi">{day.tithi}</div>
                <div className="day-ad-info">
                  {day.ad_date.split("-").pop()}
                </div>
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
  const [attendanceHistory, setAttendanceHistory] = useState([]);
  const [leaveLogs, setLeaveLogs] = useState([]);
  const [chartLoading, setChartLoading] = useState(true);
  const [chartRefreshToken, setChartRefreshToken] = useState(0);
  const [isSidebarOpen, setIsSidebarOpen] = useState(true);
  const [activeSection, setActiveSection] = useState("dashboard");

  useEffect(() => {
    const storedUser =
      localStorage.getItem("user") || sessionStorage.getItem("user");

    if (!storedUser) {
      navigate("/");
      return;
    }

    const getDateString = (date) => {
      const d = date || new Date();
      return (
        d.getFullYear() +
        "-" +
        (d.getMonth() + 1).toString().padStart(2, "0") +
        "-" +
        d.getDate().toString().padStart(2, "0")
      );
    };

    const fetchTodayRecord = async (userId) => {
      try {
        const dateStr = getDateString();
        const record = await invoke("get_attendance_record", {
          userId,
          date: dateStr,
        });
        if (record) {
          setTodayRecord(record);
        }
        const leaveRecord = await invoke("get_today_leave", {
          userId,
          date: dateStr,
        });
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

  useEffect(() => {
    if (!user?.id) return;

    const loadChartData = async () => {
      setChartLoading(true);

      try {
        const today = new Date();
        const start = new Date(today);
        start.setDate(today.getDate() - 6);

        const startDate = formatDateKey(start);
        const endDate = formatDateKey(today);

        const [attendance, leaves] = await Promise.all([
          invoke("get_attendance_records", {
            userId: user.id,
            startDate,
            endDate,
          }),
          invoke("get_leave_logs", { userId: user.id }),
        ]);

        setAttendanceHistory(Array.isArray(attendance) ? attendance : []);
        setLeaveLogs(Array.isArray(leaves) ? leaves : []);
      } catch (err) {
        console.error("Error loading dashboard charts:", err);
        setAttendanceHistory([]);
        setLeaveLogs([]);
      } finally {
        setChartLoading(false);
      }
    };

    loadChartData();
  }, [user, chartRefreshToken]);

  const saveRecord = async (record) => {
    setTodayRecord(record);
    if (user) {
      try {
        await invoke("save_attendance_record", { record });
        setChartRefreshToken((value) => value + 1);
      } catch (err) {
        console.error("Error saving attendance:", err);
      }
    }
  };

  const handleCheckIn = (manualTime, manualDate) => {
    const now = new Date();
    const timeStr =
      manualTime ||
      now.getHours().toString().padStart(2, "0") +
        ":" +
        now.getMinutes().toString().padStart(2, "0");

    const dateStr =
      manualDate ||
      now.getFullYear() +
        "-" +
        (now.getMonth() + 1).toString().padStart(2, "0") +
        "-" +
        now.getDate().toString().padStart(2, "0");

    const newRecord = {
      user_id: user.id,
      date: dateStr,
      check_in: timeStr,
      check_out: null,
      status: "checked-in",
      overtime: 0,
      is_manual: !!manualTime,
    };
    saveRecord(newRecord);
  };

  const calculateOvertime = (checkIn, checkOut) => {
    const [inH, inM] = checkIn.split(":").map(Number);
    const [outH, outM] = checkOut.split(":").map(Number);

    const durationMinutes = outH * 60 + outM - (inH * 60 + inM);
    const overtime = Math.max(0, durationMinutes - 480);
    return overtime;
  };

  const handleCheckOut = (manualTime, manualDate) => {
    if (!todayRecord && !manualDate) return;

    const now = new Date();
    const timeStr =
      manualTime ||
      now.getHours().toString().padStart(2, "0") +
        ":" +
        now.getMinutes().toString().padStart(2, "0");

    const dateStr =
      manualDate ||
      now.getFullYear() +
        "-" +
        (now.getMonth() + 1).toString().padStart(2, "0") +
        "-" +
        now.getDate().toString().padStart(2, "0");

    const updatedRecord = {
      user_id: user.id,
      date: dateStr,
      check_in: todayRecord?.check_in || null,
      check_out: timeStr,
      status: "checked-out",
      overtime: calculateOvertime(todayRecord?.check_in || manualTime, timeStr),
      is_manual: todayRecord?.is_manual || !!manualTime,
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
      status: "checked-out",
      overtime: calculateOvertime(checkIn, checkOut),
      is_manual: true,
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

      {/* Navbar */}
      <Navbar />

      {/* Sidebar */}
      <Sidebar
        onLogout={handleLogout}
        user={user}
        isOpen={isSidebarOpen}
        onToggle={() => setIsSidebarOpen(!isSidebarOpen)}
        activeItem={activeSection}
        onSelectItem={setActiveSection}
      />

      <main
        className={`main-content ${!isSidebarOpen ? "sidebar-closed" : ""} ${activeSection === "settings" ? "is-settings" : ""}`}
      >
        {activeSection === "attendance" ? (
          <AttendanceTablePage userId={user.id} />
        ) : activeSection === "settings" ? (
          <SettingsPage user={user} />
        ) : (
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
                <HolidayForm
                  userId={user.id}
                  onSaved={() => setChartRefreshToken((value) => value + 1)}
                />
              </div>
            </div>

            <div className="right-side">
              <div className="dashboard-insights">
                {chartLoading ? (
                  <>
                    <div className="chart-card chart-loading">
                      Loading attendance chart...
                    </div>
                    <div className="chart-card chart-loading">
                      Loading leave chart...
                    </div>
                  </>
                ) : (
                  <>
                    <WeeklyAttendanceChart
                      data={buildWeeklyAttendanceData(
                        attendanceHistory,
                        leaveLogs,
                      )}
                    />
                    <LeaveDistributionChart
                      data={buildLeaveDistribution(leaveLogs)}
                    />
                  </>
                )}
              </div>
              <Calendar />
            </div>
          </div>
        )}
      </main>
    </div>
  );
};

export default Dashboard;
