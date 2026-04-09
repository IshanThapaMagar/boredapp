import React, { useEffect, useState } from "react";
import { LogIn, LogOut, Clock, Edit2 } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { NepaliDatePicker } from "nepali-datepicker-reactjs";
import "nepali-datepicker-reactjs/dist/index.css";
import { CALENDAR_TYPES, getTodayByPreference } from "./lib/calendar";
import "./AttendanceActions.css";

export function AttendanceActions({
  userId,
  todayRecord,
  todayLeave,
  calendarPreference,
  onCheckIn,
  onCheckOut,
  onManualLog,
}) {
  const [manualDate, setManualDate] = useState("");
  const [manualCheckIn, setManualCheckIn] = useState("");
  const [manualCheckOut, setManualCheckOut] = useState("");
  const [isManualOpen, setIsManualOpen] = useState(false);
  const [manualAction, setManualAction] = useState("in"); // 'in', 'out', or 'log'
  const [todayStartTime, setTodayStartTime] = useState(null);

  const isCheckedIn = todayRecord?.status === "checked-in";
  const isCheckedOut = todayRecord?.status === "checked-out";

  useEffect(() => {
    const loadOfficeHours = async () => {
      if (!userId) return;
      try {
        const hours = await invoke("get_office_hours", { userId });
        if (!Array.isArray(hours)) {
          setTodayStartTime(null);
          return;
        }

        const dayOfWeek = new Date().getDay();
        const todayOfficeHour = hours.find(
          (hour) => hour.day_of_week === dayOfWeek,
        );

        if (todayOfficeHour?.is_off_day || !todayOfficeHour?.start_time) {
          setTodayStartTime(null);
          return;
        }

        setTodayStartTime(todayOfficeHour.start_time);
      } catch (err) {
        console.error("Failed to fetch office hours:", err);
        setTodayStartTime(null);
      }
    };

    loadOfficeHours();
  }, [userId]);

  const toMinutes = (timeValue) => {
    if (!timeValue) return null;
    const [hours, minutes] = timeValue.split(":");
    const parsedHours = Number(hours);
    const parsedMinutes = Number(minutes);

    if (Number.isNaN(parsedHours) || Number.isNaN(parsedMinutes)) {
      return null;
    }

    return parsedHours * 60 + parsedMinutes;
  };

  const isLate =
    todayRecord?.check_in &&
    toMinutes(todayRecord.check_in) !== null &&
    toMinutes(todayStartTime) !== null &&
    toMinutes(todayRecord.check_in) > toMinutes(todayStartTime);

  const formatTime = (date) => {
    return (
      date.getHours().toString().padStart(2, "0") +
      ":" +
      date.getMinutes().toString().padStart(2, "0")
    );
  };

  const formatDate = (date) => {
    return (
      date.getFullYear() +
      "-" +
      (date.getMonth() + 1).toString().padStart(2, "0") +
      "-" +
      date.getDate().toString().padStart(2, "0")
    );
  };

  const handleManualSubmit = (e) => {
    e.preventDefault();
    if (manualAction === "log") {
      if (manualDate && manualCheckIn && manualCheckOut) {
        onManualLog({
          date: manualDate,
          checkIn: manualCheckIn,
          checkOut: manualCheckOut,
        });
        setIsManualOpen(false);
      }
    } else if (manualAction === "in") {
      if (manualCheckIn) {
        onCheckIn(manualCheckIn, manualDate);
        setIsManualOpen(false);
      }
    } else if (manualAction === "out") {
      if (manualCheckOut) {
        onCheckOut(manualCheckOut, manualDate);
        setIsManualOpen(false);
      }
    }
  };

  const openManualDialog = (action) => {
    setManualAction(action);
    const now = new Date();
    setManualDate(
      getTodayByPreference(calendarPreference || CALENDAR_TYPES.AD),
    );
    if (action === "in" || action === "log") {
      setManualCheckIn(
        action === "log" && todayRecord?.check_in
          ? todayRecord.check_in
          : formatTime(now),
      );
    }
    if (action === "out" || action === "log") {
      setManualCheckOut(
        action === "log" && todayRecord?.check_out
          ? todayRecord.check_out
          : formatTime(now),
      );
    }
    setIsManualOpen(true);
  };

  const isFullDayLeave =
    todayLeave &&
    (todayLeave.leave_type === "public_holiday" ||
      todayLeave.leave_type === "absent");
  const disabled = isFullDayLeave || false;

  return (
    <div className="attendance-actions">
      {todayLeave ? (
        <div className="leave-notice">
          <h3 className="leave-notice-title">
            {todayLeave.leave_type === "public_holiday"
              ? "Today is a public holiday"
              : todayLeave.leave_type === "half_day"
                ? "You're on Half-Day leave today"
                : "You're on leave today"}
          </h3>
          {todayLeave.leave_type === "half_day" && (
            <p className="leave-notice-notes mt-1">
              You may only log hours for the remainder of your shift.
            </p>
          )}
          {todayLeave.notes && (
            <p className="leave-notice-notes">{todayLeave.notes}</p>
          )}
        </div>
      ) : null}      <div className="main-actions">
        <button
          onClick={() => onCheckIn()}
          disabled={disabled || isCheckedIn || isCheckedOut}
          className="action-btn btn-primary"
        >
          <LogIn className="w-5 h-5" />
          Check In
        </button>
        <button
          onClick={() => onCheckOut()}
          disabled={disabled || !isCheckedIn}
          className="action-btn btn-secondary"
        >
          <LogOut className="w-5 h-5" />
          Check Out
        </button>
      </div>

      <div className="secondary-actions">
        <button
          className="manual-btn"
          onClick={() => openManualDialog("in")}
          disabled={disabled || isCheckedIn || isCheckedOut}
        >
          <Edit2 className="w-6 h-4" />
          Manual Check In
        </button>
        <button
          className="manual-btn"
          onClick={() => openManualDialog("out")}
          disabled={disabled || !isCheckedIn}
        >
          <Edit2 className="w-4 h-4" />
          Manual Out
        </button>
        <button
          onClick={() => openManualDialog("log")}
          className="manual-btn btn-outline"
          disabled={disabled}
        >
          <Clock className="w-4 h-4" />
          Full Log
        </button>
      </div>

      {!todayLeave && (
        <div className="status-card">
          <h4 className="status-title">Today's Status</h4>
          {todayRecord && todayRecord.status !== "absent" ? (
            <div className="status-grid">
              <div className="time-entries">
                {todayRecord.check_in && (
                  <div className="time-entry">
                    <span className="label">Check In</span>
                    <span className={`time in ${isLate ? "late" : ""}`}>
                      {todayRecord.check_in}
                    </span>
                    {isLate && (
                      <span className="late-warning">Checked in late</span>
                    )}
                  </div>
                )}
                {todayRecord.check_out && (
                  <div className="time-entry">
                    <span className="label">Check Out</span>
                    <span className="time">{todayRecord.check_out}</span>
                  </div>
                )}
              </div>
              {todayRecord.overtime > 0 && (
                <div className="overtime">
                  <span className="label">Overtime</span>
                  <span className="overtime-value">
                    +{Math.floor(todayRecord.overtime / 60)}h{" "}
                    {todayRecord.overtime % 60}m
                  </span>
                </div>
              )}
            </div>
          ) : (
            <div className="status-grid">
              <div className="time-entries">
                <div className="time-entry">
                  <span className="label">Status</span>
                  <span className={`time ${todayRecord?.status === "absent" ? "absent" : ""}`}>
                    {todayRecord?.status === "absent" ? "Absent" : "Not checked in"}
                  </span>
                </div>
              </div>
            </div>
          )}
          {todayRecord?.is_manual && todayRecord.status !== "absent" && (
            <span className="manual-tag">• Manually entered</span>
          )}
        </div>
      )}

      {isManualOpen && (
        <div className="dialog-overlay" onClick={() => setIsManualOpen(false)}>
          <div className="dialog-content" onClick={(e) => e.stopPropagation()}>
            <div className="dialog-header flex items-center gap-2 mb-4">
              <Clock className="w-6 h-6 text-indigo-600" />
              <h3 className="text-xl font-bold text-slate-800">
                {manualAction === "log"
                  ? "Full Attendance Log"
                  : manualAction === "in"
                    ? "Manual Check In"
                    : "Manual Check Out"}
              </h3>
            </div>
            <form onSubmit={handleManualSubmit}>
              <div className="form-group mb-3">
                <label className="text-sm font-medium text-slate-700">
                  Date
                </label>
                {calendarPreference === CALENDAR_TYPES.BS ? (
                  <NepaliDatePicker
                    inputClassName="time-input"
                    value={manualDate}
                    onChange={(value) => setManualDate(value)}
                    options={{ calenderLocale: "ne", valueLocale: "en" }}
                  />
                ) : (
                  <input
                    type="date"
                    required
                    value={manualDate}
                    onChange={(e) => setManualDate(e.target.value)}
                    className="time-input"
                  />
                )}
              </div>

              {(manualAction === "in" || manualAction === "log") && (
                <div className="form-group mb-3">
                  <label className="text-sm font-medium text-black">
                    Check In Time
                  </label>
                  <input
                    type="time"
                    required
                    value={manualCheckIn}
                    onChange={(e) => setManualCheckIn(e.target.value)}
                    className="time-input"
                  />
                </div>
              )}

              {(manualAction === "out" || manualAction === "log") && (
                <div className="form-group mb-3">
                  <label className="text-sm font-medium text-black">
                    Check Out Time
                  </label>
                  <input
                    type="time"
                    required
                    value={manualCheckOut}
                    onChange={(e) => setManualCheckOut(e.target.value)}
                    className="time-input"
                  />
                </div>
              )}

              <button type="submit" className="submit-btn text-white mt-4">
                Confirm Entry
              </button>
            </form>
          </div>
        </div>
      )}
    </div>
  );
}
