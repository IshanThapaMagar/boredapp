import React, { useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Filter, RefreshCcw } from "lucide-react";
import {
  CALENDAR_TYPES,
  adToBs,
  bsToAd,
  NEPALI_MONTHS,
  getBsMonthRange,
} from "./lib/calendar";
import { NepaliDatePicker } from "nepali-datepicker-reactjs";
import "nepali-datepicker-reactjs/dist/index.css";
import NepaliDate from "nepali-date-converter";
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

  const clampedEnd =
    year === today.getFullYear() && month === today.getMonth() ? today : end;

  return {
    from: formatDate(start),
    to: formatDate(clampedEnd),
  };
};

const getClampedBsMonthRange = (year, month) => {
  const range = getBsMonthRange(year, month);
  const todayAd = formatDate(new Date());

  return {
    from: range.from,
    to: range.to > todayAd ? todayAd : range.to,
  };
};

const minutesToHoursText = (minutes) => {
  if (!minutes || minutes <= 0) return "0h 0m";
  return `${Math.floor(minutes / 60)}h ${minutes % 60}m`;
};

export default function AttendanceTablePage({ userId, calendarPreference }) {
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
    if (record.status === "absent")
      return { label: "Absent", tone: "absent" };
    if (record.status === "half_day")
      return { label: "Half Day Leave", tone: "absent" };
    return { label: "Not checked in", tone: "notcheckedin" };
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
          invoke("get_office_hours", { userId }),
        ]);

      const attendance = Array.isArray(attendanceResponse)
        ? attendanceResponse
        : [];
      const leaveLogs = Array.isArray(leaveResponse) ? leaveResponse : [];
      const officeHours = Array.isArray(officeHoursResponse)
        ? officeHoursResponse
        : [];

      const attendanceMap = new Map(
        attendance.map((item) => [item.attendance_date_ad || item.date, item]),
      );
      const leaveMap = new Map(
        leaveLogs
          .filter(
            (log) =>
              (log.leave_date_ad || log.leave_date) >= startDate &&
              (log.leave_date_ad || log.leave_date) <= endDate,
          )
          .map((log) => [log.leave_date_ad || log.leave_date, log]),
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
            attendance_date_ad: date,
            attendance_date_bs: adToBs(date),
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
            attendance_date_ad: date,
            attendance_date_bs: adToBs(date),
            check_in: null,
            check_out: null,
            status: "off-day",
            overtime: 0,
            is_manual: false,
          };
        }

        if (leaveItem?.leave_type === "absent") {
          return {
            user_id: userId,
            date,
            attendance_date_ad: date,
            attendance_date_bs: adToBs(date),
            check_in: null,
            check_out: null,
            status: "absent",
            overtime: 0,
            is_manual: false,
          };
        }

        if (leaveItem?.leave_type === "half_day") {
          return {
            user_id: userId,
            date,
            attendance_date_ad: date,
            attendance_date_bs: adToBs(date),
            check_in: null,
            check_out: null,
            status: "half_day",
            overtime: 0,
            is_manual: false,
          };
        }

        return {
          user_id: userId,
          date,
          attendance_date_ad: date,
          attendance_date_bs: adToBs(date),
          check_in: null,
          check_out: null,
          status: "notcheckedin",
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
    if (calendarPreference === CALENDAR_TYPES.BS) {
      const now = new NepaliDate();
      const { from, to } = getClampedBsMonthRange(
        now.getYear(),
        now.getMonth() + 1,
      );
      setFromDate(from);
      setToDate(to);
      setSelectedMonth({ year: now.getYear(), month: now.getMonth() + 1 });
      fetchRecords(from, to);
    } else {
      fetchRecords();
    }
  }, [userId, calendarPreference]);

  const applyMonthRange = (baseDate, bsMonthData = null) => {
    if (calendarPreference === CALENDAR_TYPES.BS && bsMonthData) {
      const { year, month } = bsMonthData;
      const { from, to } = getClampedBsMonthRange(year, month);
      setSelectedMonth(bsMonthData);
      setFromDate(from);
      setToDate(to);
      fetchRecords(from, to);
    } else {
      const nextMonth = new Date(
        baseDate.getFullYear(),
        baseDate.getMonth(),
        1,
      );
      const { from, to } = getMonthRange(nextMonth);

      setSelectedMonth(nextMonth);
      setFromDate(from);
      setToDate(to);
      fetchRecords(from, to);
    }
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
            onClick={() => {
              if (calendarPreference === CALENDAR_TYPES.BS) {
                let { year, month } =
                  selectedMonth instanceof NepaliDate
                    ? {
                        year: selectedMonth.getYear(),
                        month: selectedMonth.getMonth() + 1,
                      }
                    : new NepaliDate(selectedMonth).getYear()
                      ? {
                          year: new NepaliDate(selectedMonth).getYear(),
                          month: new NepaliDate(selectedMonth).getMonth() + 1,
                        }
                      : { year: 2081, month: 1 };
                // If selectedMonth is Date (AD), convert it first
                if (
                  !(selectedMonth instanceof NepaliDate) &&
                  selectedMonth.getFullYear
                ) {
                  const nd = new NepaliDate(selectedMonth);
                  year = nd.getYear();
                  month = nd.getMonth() + 1;
                } else if (
                  typeof selectedMonth === "object" &&
                  selectedMonth.year
                ) {
                  year = selectedMonth.year;
                  month = selectedMonth.month;
                }

                month--;
                if (month < 1) {
                  month = 12;
                  year--;
                }
                applyMonthRange(null, { year, month });
              } else {
                applyMonthRange(
                  new Date(
                    selectedMonth.getFullYear(),
                    selectedMonth.getMonth() - 1,
                    1,
                  ),
                );
              }
            }}
          >
            Prev Month
          </button>
          <div className="attendance-month-label">
            {calendarPreference === CALENDAR_TYPES.BS
              ? (() => {
                  let year, month;
                  if (typeof selectedMonth === "object" && selectedMonth.year) {
                    year = selectedMonth.year;
                    month = selectedMonth.month;
                  } else {
                    const nd = new NepaliDate(
                      selectedMonth instanceof Date
                        ? selectedMonth
                        : new Date(),
                    );
                    year = nd.getYear();
                    month = nd.getMonth() + 1;
                  }
                  return `${NEPALI_MONTHS[month - 1]} ${year}`;
                })()
              : selectedMonth.toLocaleDateString("en-US", {
                  month: "long",
                  year: "numeric",
                })}
          </div>
          <button
            type="button"
            className="attendance-btn secondary"
            onClick={() => {
              if (calendarPreference === CALENDAR_TYPES.BS) {
                let year, month;
                if (typeof selectedMonth === "object" && selectedMonth.year) {
                  year = selectedMonth.year;
                  month = selectedMonth.month;
                } else {
                  const nd = new NepaliDate(
                    selectedMonth instanceof Date ? selectedMonth : new Date(),
                  );
                  year = nd.getYear();
                  month = nd.getMonth() + 1;
                }

                month++;
                if (month > 12) {
                  month = 1;
                  year++;
                }
                applyMonthRange(null, { year, month });
              } else {
                applyMonthRange(
                  new Date(
                    selectedMonth.getFullYear(),
                    selectedMonth.getMonth() + 1,
                    1,
                  ),
                );
              }
            }}
          >
            Next Month
          </button>
          <button
            type="button"
            className="attendance-btn"
            onClick={() => {
              if (calendarPreference === CALENDAR_TYPES.BS) {
                const now = new NepaliDate();
                applyMonthRange(null, {
                  year: now.getYear(),
                  month: now.getMonth() + 1,
                });
              } else {
                applyMonthRange(new Date());
              }
            }}
          >
            Current Month
          </button>
        </div>

        <div className="attendance-filter-grid">
          <label className="attendance-filter-item">
            From
            {calendarPreference === CALENDAR_TYPES.BS ? (
              <NepaliDatePicker
                inputClassName="attendance-date-input"
                value={adToBs(fromDate)}
                onChange={(value) => setFromDate(bsToAd(value))}
                options={{ calenderLocale: "ne", valueLocale: "en" }}
              />
            ) : (
              <input
                type="date"
                value={fromDate}
                onChange={(e) => setFromDate(e.target.value)}
              />
            )}
          </label>

          <label className="attendance-filter-item">
            To
            {calendarPreference === CALENDAR_TYPES.BS ? (
              <NepaliDatePicker
                inputClassName="attendance-date-input"
                value={adToBs(toDate)}
                onChange={(value) => setToDate(bsToAd(value))}
                options={{ calenderLocale: "ne", valueLocale: "en" }}
              />
            ) : (
              <input
                type="date"
                value={toDate}
                onChange={(e) => setToDate(e.target.value)}
              />
            )}
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
                if (calendarPreference === CALENDAR_TYPES.BS) {
                  const now = new NepaliDate();
                  const { from, to } = getClampedBsMonthRange(
                    now.getYear(),
                    now.getMonth() + 1,
                  );
                  setFromDate(from);
                  setToDate(to);
                  setSelectedMonth({
                    year: now.getYear(),
                    month: now.getMonth() + 1,
                  });
                  setStatusFilter("all");
                  setEntryFilter("all");
                  fetchRecords(from, to);
                } else {
                  const now = new Date();
                  setSelectedMonth(now);
                  setFromDate(getDefaultFromDate());
                  setToDate(getDefaultToDate());
                  setStatusFilter("all");
                  setEntryFilter("all");
                  fetchRecords(getDefaultFromDate(), getDefaultToDate());
                }
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
                  <tr
                    key={`${record.attendance_date_ad || record.date}-${record.user_id}`}
                  >
                    <td>
                      {calendarPreference === CALENDAR_TYPES.BS
                        ? record.attendance_date_bs ||
                          adToBs(record.attendance_date_ad || record.date)
                        : record.attendance_date_ad || record.date}
                    </td>
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
                        : record.status === "absent" || record.status === "half_day"
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
