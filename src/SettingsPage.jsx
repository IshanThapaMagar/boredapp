import React, { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Save, AlertCircle, CheckCircle2, HelpCircle } from "lucide-react";
import SettingsSidebar from "./components/SettingsSidebar";
import "./SettingsPage.css";

const DAYS_OF_WEEK = [
    "Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"
];

const ProfileSection = ({ user }) => {
    return (
        <div className="settings-section">
            <div className="section-header">
                <div className="header-text">
                    <h2>Account Profile</h2>
                    <p>Manage your account settings and profile information.</p>
                </div>
            </div>
            {/* ... rest of ProfileSection ... */}

            <div className="profile-info-grid">
                <div className="info-field">
                    <label>Username</label>
                    <div className="field-value">{user?.username || "N/A"}</div>
                </div>
                <div className="info-field">
                    <label>Email Address</label>
                    <div className="field-value">{user?.email || "N/A"}</div>
                </div>
                <div className="info-field">
                    <label>Account ID</label>
                    <div className="field-value"># {user?.id || "N/A"}</div>
                </div>
            </div>

            <div className="settings-notice">
                <AlertCircle size={18} />
                <p>Profile editing is currently managed by the administrator.</p>
            </div>
        </div>
    );
};

const CalendarSection = () => {
    const [officeHours, setOfficeHours] = useState([]);
    const [isSaving, setIsSaving] = useState(false);
    const [message, setMessage] = useState(null);
    const [calendarFormat, setCalendarFormat] = useState("np");

    useEffect(() => {
        fetchOfficeHours();
    }, []);

    const fetchOfficeHours = async () => {
        try {
            const hours = await invoke("get_office_hours");
            setOfficeHours(hours);
        } catch (err) {
            console.error("Failed to fetch office hours:", err);
            setMessage({ type: "error", text: "Failed to load working hours." });
        }
    };

    const handleHourChange = (dayIndex, field, value) => {
        setOfficeHours(prev => prev.map(hour => {
            if (hour.day_of_week === dayIndex) {
                return { ...hour, [field]: value };
            }
            return hour;
        }));
    };

    const saveOfficeHours = async () => {
        setIsSaving(true);
        setMessage(null);
        try {
            await invoke("save_office_hours", { hours: officeHours });
            setMessage({ type: "success", text: "Working hours saved successfully!" });
            setTimeout(() => setMessage(null), 3000);
        } catch (err) {
            console.error("Failed to save office hours:", err);
            setMessage({ type: "error", text: "Failed to save working hours." });
        } finally {
            setIsSaving(false);
        }
    };

    return (
        <div className="settings-section">
            <div className="section-header">
                <div className="header-text">
                    <h2>Calendar Settings</h2>
                    <p>Configure your standard weekly working hours and off-days.</p>
                </div>
                
                <div className="header-controls">
                    <div className="format-selector">
                        <select 
                            value={calendarFormat} 
                            onChange={(e) => setCalendarFormat(e.target.value)}
                            className="calendar-type-select"
                        >
                            <option value="np">Nepali (B.S.)</option>
                            <option value="en">English (A.D.)</option>
                        </select>
                        
                        <div className="tooltip-container">
                            <HelpCircle size={18} className="help-icon" />
                            <div className="tooltip-content">
                                Switch between Nepali Bikram Sambat (B.S.) and English Anno Domini (A.D.) calendar types for all date displays.
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            <div className="working-hours-container">
                {officeHours.length === 0 ? (
                    <div className="loading-state">Loading working hours...</div>
                ) : (
                    <div className="hours-list">
                        {officeHours.map((hour) => (
                            <div key={hour.day_of_week} className={`day-settings-row ${hour.is_off_day ? 'is-off' : ''}`}>
                                <div className="day-name">{DAYS_OF_WEEK[hour.day_of_week]}</div>
                                
                                <div className="time-controls">
                                    <div className="settings-input-group">
                                        <input
                                            type="time"
                                            value={hour.start_time || "09:00"}
                                            onChange={(e) => handleHourChange(hour.day_of_week, "start_time", e.target.value)}
                                            disabled={hour.is_off_day}
                                        />
                                    </div>
                                    <span className="separator">to</span>
                                    <div className="settings-input-group">
                                        <input
                                            type="time"
                                            value={hour.end_time || "17:00"}
                                            onChange={(e) => handleHourChange(hour.day_of_week, "end_time", e.target.value)}
                                            disabled={hour.is_off_day}
                                        />
                                    </div>
                                </div>

                                <label className="off-day-toggle">
                                    <input
                                        type="checkbox"
                                        checked={hour.is_off_day}
                                        onChange={(e) => handleHourChange(hour.day_of_week, "is_off_day", e.target.checked)}
                                    />
                                    <span className="toggle-label">Off Day</span>
                                </label>
                            </div>
                        ))}
                    </div>
                )}

                <div className="section-footer">
                    {message && (
                        <div className={`status-message ${message.type}`}>
                            {message.type === 'success' ? <CheckCircle2 size={18} /> : <AlertCircle size={18} />}
                            {message.text}
                        </div>
                    )}
                    <button 
                        className="save-settings-btn"
                        onClick={saveOfficeHours}
                        disabled={isSaving || officeHours.length === 0}
                    >
                        {isSaving ? "Saving..." : <><Save size={18} /> Save Calendar Settings</>}
                    </button>
                </div>
            </div>
        </div>
    );
};

const SettingsPage = ({ user }) => {
    const [subSection, setSubSection] = useState("profile");

    return (
        <div className="settings-page-wrapper">
            <SettingsSidebar 
                activeSection={subSection} 
                onSelect={setSubSection} 
            />

            <main className="settings-content-area">
                {subSection === "profile" && <ProfileSection user={user} />}
                {subSection === "calendar" && <CalendarSection />}
            </main>
        </div>
    );
};

export default SettingsPage;
