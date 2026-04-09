import React, { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { X } from "lucide-react";
import "./SettingsModal.css";

const DAYS_OF_WEEK = [
  "Sunday",
  "Monday",
  "Tuesday",
  "Wednesday",
  "Thursday",
  "Friday",
  "Saturday",
];

export default function SettingsModal({ isOpen, onClose }) {
  const [officeHours, setOfficeHours] = useState([]);
  const [isSavingHours, setIsSavingHours] = useState(false);
  const [hoursMessage, setHoursMessage] = useState(null);
  const [userId, setUserId] = useState(null);

  useEffect(() => {
    const storedUser =
      localStorage.getItem("user") || sessionStorage.getItem("user");
    if (!storedUser) return;

    try {
      const user = JSON.parse(storedUser);
      setUserId(user?.id || null);
    } catch (err) {
      console.error("Failed to parse stored user:", err);
      setUserId(null);
    }
  }, []);

  useEffect(() => {
    if (isOpen && userId) {
      fetchOfficeHours();
    } else if (!isOpen) {
      setHoursMessage(null);
    }
  }, [isOpen, userId]);

  const fetchOfficeHours = async () => {
    if (!userId) return;
    try {
      const hours = await invoke("get_office_hours", { userId });
      setOfficeHours(hours);
    } catch (err) {
      console.error("Failed to fetch office hours:", err);
      setHoursMessage({ type: "error", text: "Failed to load working hours." });
    }
  };

  const handleHourChange = (dayIndex, field, value) => {
    setOfficeHours((prev) =>
      prev.map((hour) => {
        if (hour.day_of_week === dayIndex) {
          return { ...hour, [field]: value };
        }
        return hour;
      }),
    );
  };

  const saveOfficeHours = async () => {
    if (!userId) return;
    setIsSavingHours(true);
    setHoursMessage(null);
    try {
      await invoke("save_office_hours", { userId, hours: officeHours });
      setHoursMessage({
        type: "success",
        text: "Working hours saved successfully!",
      });
      setTimeout(() => setHoursMessage(null), 3000);
    } catch (err) {
      console.error("Failed to save office hours:", err);
      setHoursMessage({ type: "error", text: "Failed to save working hours." });
    } finally {
      setIsSavingHours(false);
    }
  };

  if (!isOpen) return null;

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal-content" onClick={(e) => e.stopPropagation()}>
        <div className="modal-header">
          <h2>Settings - Working Hours</h2>
          <button className="close-btn" onClick={onClose}>
            <X size={24} />
          </button>
        </div>

        <div className="modal-body">
          <div className="hours-tab">
            {officeHours.length === 0 ? (
              <p>Loading...</p>
            ) : (
              <div className="hours-list">
                {officeHours.map((hour) => (
                  <div key={hour.day_of_week} className="day-row">
                    <div className="day-label">
                      {DAYS_OF_WEEK[hour.day_of_week]}
                    </div>

                    <div className="time-inputs">
                      <input
                        type="time"
                        value={hour.start_time || ""}
                        onChange={(e) =>
                          handleHourChange(
                            hour.day_of_week,
                            "start_time",
                            e.target.value,
                          )
                        }
                        disabled={hour.is_off_day}
                      />
                      <span>to</span>
                      <input
                        type="time"
                        value={hour.end_time || ""}
                        onChange={(e) =>
                          handleHourChange(
                            hour.day_of_week,
                            "end_time",
                            e.target.value,
                          )
                        }
                        disabled={hour.is_off_day}
                      />
                    </div>

                    <label className="off-day-toggle">
                      <input
                        type="checkbox"
                        checked={hour.is_off_day}
                        onChange={(e) =>
                          handleHourChange(
                            hour.day_of_week,
                            "is_off_day",
                            e.target.checked,
                          )
                        }
                      />
                      Off Day
                    </label>
                  </div>
                ))}
              </div>
            )}

            {hoursMessage && (
              <div className={`message ${hoursMessage.type}`}>
                {hoursMessage.text}
              </div>
            )}
          </div>
        </div>

        <div className="modal-footer">
          <button
            className="btn-primary"
            onClick={saveOfficeHours}
            disabled={isSavingHours}
          >
            {isSavingHours ? "Saving..." : "Save Working Hours"}
          </button>
          <button className="btn-secondary" onClick={onClose}>
            Close
          </button>
        </div>
      </div>
    </div>
  );
}
