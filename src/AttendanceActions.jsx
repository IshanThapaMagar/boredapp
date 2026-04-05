import React, { useState } from "react";
import { LogIn, LogOut, Clock, Edit2 } from "lucide-react";
import "./AttendanceActions.css";

export function AttendanceActions({
  todayRecord,
  todayLeave,
  onCheckIn,
  onCheckOut,
  onManualLog,
}) {
  const [manualDate, setManualDate] = useState("");
  const [manualCheckIn, setManualCheckIn] = useState("");
  const [manualCheckOut, setManualCheckOut] = useState("");
  const [isManualOpen, setIsManualOpen] = useState(false);
  const [manualAction, setManualAction] = useState("in"); // 'in', 'out', or 'log'

  const isCheckedIn = todayRecord?.status === "checked-in";
  const isCheckedOut = todayRecord?.status === "checked-out";

  // Assuming check-in after 10:00 is considered late
  const isLate = todayRecord?.check_in && todayRecord.check_in > "10:00";

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
    setManualDate(formatDate(now));
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

  const disabled = (todayLeave !== null && todayLeave !== undefined) || false;

  return (
    <div className="attendance-actions">
      {todayLeave ? (
        <div className="leave-notice">
          <h3 className="leave-notice-title">
            {todayLeave.leave_type === "public_holiday"
              ? "Today is a public holiday"
              : "You're on leave today"}
          </h3>
          {todayLeave.notes && (
            <p className="leave-notice-notes">{todayLeave.notes}</p>
          )}
        </div>
      ) : null}
      <div className="main-actions">
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

      {todayRecord && (
        <div className="status-card">
          <h4 className="status-title">Today's Status</h4>
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
          {todayRecord.is_manual && (
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
                <input
                  type="date"
                  required
                  value={manualDate}
                  onChange={(e) => setManualDate(e.target.value)}
                  className="time-input"
                />
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
