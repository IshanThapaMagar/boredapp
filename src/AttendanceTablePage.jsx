import React, { useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Filter, RefreshCcw } from "lucide-react";
import "./AttendanceTablePage.css";

const formatDate = (date) => {
  const year = date.getFullYear();
  const month = `${date.getMonth() + 1}`.padStart(2, "0");
  const day = `${date.getDate()}`.padStart(2, "0");
  return `${year}-${month}-${day}`;
};

const getDefaultFromDate = () => {
  const now = new Date();
  return formatDate(new Date(now.getFullYear(), now.getMonth(), 1));
};

const getDefaultToDate = () => formatDate(new Date());

const getMonthRange = (baseDate) => {
  const year = baseDate.getFullYear();
  const month = baseDate.getMonth();
  const start = new Date(year, month, 1);
  const end = new Date(year, month + 1, 0);
  const today = new Date();

  // Clamp end date for current month so we don't request future dates.
  const clampedEnd =
    year === today.getFullYear() && month === today.getMonth() ? today : end;

  return {
    from: formatDate(start),
    to: formatDate(clampedEnd),
  };
};

const minutesToHoursText = (minutes) => {
  if (!minutes || minutes <= 0) return "0h 0m";
  return `${Math.floor(minutes / 60)}h ${minutes % 60}m`;
};

export default function AttendanceTablePage({ userId }) {
  const [records, setRecords] = useState([]);
  const [isLoading, setIsLoading] = useState(true);
  const [loadError, setLoadError] = useState("");

  const [fromDate, setFromDate] = useState(getDefaultFromDate);
  const [toDate, setToDate] = useState(getDefaultToDate);
  const [statusFilter, setStatusFilter] = useState("all");
  const [entryFilter, setEntryFilter] = useState("all");
  const [selectedMonth, setSelectedMonth] = useState(() => new Date());

  const getDateRange = (start, end) => {
    const list = [];
    const current = new Date(`${start}T00:00:00`);
    const last = new Date(`${end}T00:00:00`);

    while (current <= last) {
      list.push(formatDate(current));
      current.setDate(current.getDate() + 1);
    }

    return list;
  };

  const getStatusInfo = (record) => {
    if (record.status === "holiday")
      return { label: "Holiday", tone: "holiday" };
    if (record.status === "off-day")
      return { label: "Off Day", tone: "offday" };
    if (record.check_in && record.check_out)
      return { label: "Present", tone: "present" };
    if (record.check_in && !record.check_out)
      return { label: "Checked In", tone: "checkedin" };
    return { label: "Absent", tone: "absent" };
  };

  const fetchRecords = async (customFromDate, customToDate) => {
    if (!userId) return;

    const startDate = customFromDate || fromDate;
    const endDate = customToDate || toDate;

    setIsLoading(true);
    setLoadError("");

    try {
      const [attendanceResponse, leaveResponse, officeHoursResponse] =
        await Promise.all([
          invoke("get_attendance_records", {
            userId,
            startDate,
            endDate,
          }),
          invoke("get_leave_logs", { userId }),
          invoke("get_office_hours"),
        ]);

      const attendance = Array.isArray(attendanceResponse)
        ? attendanceResponse
        : [];
      const leaveLogs = Array.isArray(leaveResponse) ? leaveResponse : [];
      const officeHours = Array.isArray(officeHoursResponse)
        ? officeHoursResponse
        : [];

      const attendanceMap = new Map(
        attendance.map((item) => [item.date, item]),
      );
      const leaveMap = new Map(
        leaveLogs
          .filter(
            (log) => log.leave_date >= startDate && log.leave_date <= endDate,
          )
          .map((log) => [log.leave_date, log]),
      );
      const officeMap = new Map(
        officeHours.map((hour) => [Number(hour.day_of_week), hour]),
      );

      const rows = getDateRange(startDate, endDate).map((date) => {
        const attendanceItem = attendanceMap.get(date);
        const leaveItem = leaveMap.get(date);
        const dayOfWeek = new Date(`${date}T00:00:00`).getDay();
        const officeForDay = officeMap.get(dayOfWeek);

        if (attendanceItem) {
          return attendanceItem;
        }

        if (leaveItem?.leave_type === "public_holiday") {
          return {
            user_id: userId,
            date,
            check_in: null,
            check_out: null,
            status: "holiday",
            overtime: 0,
            is_manual: false,
          };
        }

        if (officeForDay?.is_off_day) {
          return {
            user_id: userId,
            date,
            check_in: null,
            check_out: null,
            status: "off-day",
            overtime: 0,
            is_manual: false,
          };
        }

        return {
          user_id: userId,
          date,
          check_in: null,
          check_out: null,
          status: "absent",
          overtime: 0,
          is_manual: false,
        };
      });

      setRecords(rows);
    } catch (err) {
      console.error("Failed to load attendance records:", err);
      setLoadError("Could not load attendance records.");
      setRecords([]);
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    fetchRecords();
  }, [userId]);

  const applyMonthRange = (baseDate) => {
    const nextMonth = new Date(baseDate.getFullYear(), baseDate.getMonth(), 1);
    const { from, to } = getMonthRange(nextMonth);

    setSelectedMonth(nextMonth);
    setFromDate(from);
    setToDate(to);
    fetchRecords(from, to);
  };

  const filteredRecords = useMemo(() => {
    return records.filter((record) => {
      if (statusFilter !== "all") {
        const computedStatus = getStatusInfo(record).label.toLowerCase();
        if (computedStatus !== statusFilter) {
          return false;
        }
      }

      if (entryFilter !== "all") {
        const isManual = !!record.is_manual;
        if (entryFilter === "manual" && !isManual) {
          return false;
        }
        if (entryFilter === "system" && isManual) {
          return false;
        }
      }

      return true;
    });
  }, [records, statusFilter, entryFilter]);

  const summary = useMemo(() => {
    const total = filteredRecords.length;
    const present = filteredRecords.filter(
      (item) => item.check_in && item.check_out,
    ).length;
    const holidays = filteredRecords.filter(
      (item) => getStatusInfo(item).label === "Holiday",
    ).length;
    const absent = filteredRecords.filter(
      (item) => getStatusInfo(item).label === "Absent",
    ).length;
    const manual = filteredRecords.filter((item) => item.is_manual).length;

    return {
      total,
      present,
      holidays,
      absent,
      manual,
    };
  }, [filteredRecords]);

  return (
    <div className="attendance-page">
      <div className="attendance-summary-section">
        <div className="attendance-summary-card">
          <p className="attendance-summary-label">Total Records</p>
          <h3>{summary.total}</h3>
        </div>
        <div className="attendance-summary-card">
          <p className="attendance-summary-label">Present Days</p>
          <h3>{summary.present}</h3>
        </div>
        <div className="attendance-summary-card">
          <p className="attendance-summary-label">Holidays</p>
          <h3>{summary.holidays}</h3>
        </div>
        <div className="attendance-summary-card">
          <p className="attendance-summary-label">Absent Days</p>
          <h3>{summary.absent}</h3>
        </div>
        <div className="attendance-summary-card">
          <p className="attendance-summary-label">Manual Entries</p>
          <h3>{summary.manual}</h3>
        </div>
      </div>

      <div className="attendance-filter-section">
        <div className="attendance-filter-title">
          <Filter size={18} />
          <h3>Filter Attendance</h3>
        </div>

        <div className="attendance-month-controls">
          <button
            type="button"
            className="attendance-btn secondary"
            onClick={() =>
              applyMonthRange(
                new Date(
                  selectedMonth.getFullYear(),
                  selectedMonth.getMonth() - 1,
                  1,
                ),
              )
            }
          >
            Prev Month
          </button>
          <div className="attendance-month-label">
            {selectedMonth.toLocaleDateString("en-US", {
              month: "long",
              year: "numeric",
            })}
          </div>
          <button
            type="button"
            className="attendance-btn secondary"
            onClick={() =>
              applyMonthRange(
                new Date(
                  selectedMonth.getFullYear(),
                  selectedMonth.getMonth() + 1,
                  1,
                ),
              )
            }
          >
            Next Month
          </button>
          <button
            type="button"
            className="attendance-btn"
            onClick={() => applyMonthRange(new Date())}
          >
            Current Month
          </button>
        </div>

        <div className="attendance-filter-grid">
          <label className="attendance-filter-item">
            From
            <input
              type="date"
              value={fromDate}
              onChange={(e) => setFromDate(e.target.value)}
            />
          </label>

          <label className="attendance-filter-item">
            To
            <input
              type="date"
              value={toDate}
              onChange={(e) => setToDate(e.target.value)}
            />
          </label>

          <label className="attendance-filter-item">
            Status
            <select
              value={statusFilter}
              onChange={(e) => setStatusFilter(e.target.value)}
            >
              <option value="all">All</option>
              <option value="present">Present</option>
              <option value="checked in">Checked In</option>
              <option value="holiday">Holiday</option>
              <option value="absent">Absent</option>
              <option value="off day">Off Day</option>
            </select>
          </label>

          <label className="attendance-filter-item">
            Entry Type
            <select
              value={entryFilter}
              onChange={(e) => setEntryFilter(e.target.value)}
            >
              <option value="all">All</option>
              <option value="system">System</option>
              <option value="manual">Manual</option>
            </select>
          </label>

          <div className="attendance-filter-actions">
            <button
              type="button"
              className="attendance-btn"
              onClick={fetchRecords}
            >
              <RefreshCcw size={16} />
              Apply
            </button>
            <button
              type="button"
              className="attendance-btn secondary"
              onClick={() => {
                const now = new Date();
                setSelectedMonth(now);
                setFromDate(getDefaultFromDate());
                setToDate(getDefaultToDate());
                setStatusFilter("all");
                setEntryFilter("all");
              }}
            >
              Reset
            </button>
          </div>
        </div>
      </div>

      <div className="attendance-table-section">
        <div className="attendance-table-header">
          <h3>Attendance Table</h3>
          <span>{filteredRecords.length} row(s)</span>
        </div>

        {isLoading ? (
          <div className="attendance-empty">Loading attendance...</div>
        ) : loadError ? (
          <div className="attendance-empty error">{loadError}</div>
        ) : filteredRecords.length === 0 ? (
          <div className="attendance-empty">No attendance records found.</div>
        ) : (
          <div className="attendance-table-wrap">
            <table className="attendance-table">
              <thead>
                <tr>
                  <th>Date</th>
                  <th>Check In</th>
                  <th>Check Out</th>
                  <th>Status</th>
                  <th>Overtime</th>
                  <th>Entry</th>
                </tr>
              </thead>
              <tbody>
                {filteredRecords.map((record) => (
                  <tr key={`${record.date}-${record.user_id}`}>
                    <td>{record.date}</td>
                    <td>{record.check_in || "-"}</td>
                    <td>{record.check_out || "-"}</td>
                    <td>
                      {(() => {
                        const status = getStatusInfo(record);
                        return (
                          <span className={`status-pill ${status.tone}`}>
                            {status.label}
                          </span>
                        );
                      })()}
                    </td>
                    <td>{minutesToHoursText(record.overtime)}</td>
                    <td>
                      {record.status === "holiday" ||
                      record.status === "off-day"
                        ? "Leave"
                        : record.status === "absent"
                          ? "Auto"
                          : record.is_manual
                            ? "Manual"
                            : "System"}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>
    </div>
  );
}
