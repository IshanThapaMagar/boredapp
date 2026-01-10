import React, { useState } from "react";
import "./TodayStatus.css";

export default function TodayStatus() {
  const [clockedIn, setClockedIn] = useState(false);
  const [showModal, setShowModal] = useState(false);
  const [modalType, setModalType] = useState(null);
  const [manualTime, setManualTime] = useState("");

  const openModal = (type) => {
    setModalType(type);
    setManualTime("");
    setShowModal(true);
  };

  const closeModal = () => {
    setShowModal(false);
    setModalType(null);
  };

  const submitTime = () => {
    if (!manualTime) return;

    if (modalType === "in") {
      setClockedIn(true);
    } else if (modalType === "out") {
      setClockedIn(false);
    }

    closeModal();
  };

  return (
    <>
      <div className="status-wrapper">
        <div className="status-card">
          <div className="status-title">TODAY&apos;S STATUS</div>

          <div className="status-badge">
            {clockedIn ? "Working" : "Work Day"}
          </div>

          <div className={`clock-circle ${clockedIn ? "active" : ""}`}>
            <div className="play-icon" />
            <div className="clock-text">
              {clockedIn ? "Clocked In" : "Clock In"}
            </div>
          </div>

          <div className="action-row">
            <button
              className="action-btn"
              onClick={() => openModal("in")}
              disabled={clockedIn}
            >
              ▶ Manual In
            </button>

            <button
              className="action-btn"
              onClick={() => openModal("out")}
              disabled={!clockedIn}
            >
              ⏹ Manual Out
            </button>

            <button className="action-btn secondary">
              ＋ Full Entry
            </button>
          </div>
        </div>
      </div>

      {/* MODAL */}
      {showModal && (
        <div className="modal-backdrop">
          <div className="modal">
            <h3>
              {modalType === "in" ? "Manual Clock In" : "Manual Clock Out"}
            </h3>

            <label>Enter Time</label>
            <input
              type="time"
              value={manualTime}
              onChange={(e) => setManualTime(e.target.value)}
            />

            <div className="modal-actions">
              <button onClick={closeModal} className="cancel-btn">
                Cancel
              </button>
              <button onClick={submitTime} className="confirm-btn">
                Confirm
              </button>
            </div>
          </div>
        </div>
      )}
    </>
  );
}
