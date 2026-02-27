import React, { useEffect, useState } from "react";
import "./Clock.css";

export default function Clock() {
  const [now, setNow] = useState(new Date());
  const [nepaliDate, setNepaliDate] = useState(null);

  useEffect(() => {
    const timer = setInterval(() => setNow(new Date()), 1000);
    return () => clearInterval(timer);
  }, []);

  useEffect(() => {
    async function fetchNepaliDate() {
      try {
        const res = await fetch("https://calendar.bloggernepal.com/api/today");
        const data = await res.json();

        if (data?.res) {
          const todayAD = now.getDate();
          const todayObj =
            data.res.days.find(
              (day) => parseInt(day.ad) === todayAD
            ) || data.res.days[0];

          setNepaliDate({
            bsDay: todayObj.bs,
            bsMonth: data.res.name,
            bsYear: data.res.year,
          });
        }
      } catch (err) {
        console.error("Error fetching Nepali date:", err);
      }
    }

    fetchNepaliDate();
  }, [now]);

  const time = now.toLocaleTimeString("en-US", {
    hour: "2-digit",
    minute: "2-digit",
    hour12: true,
  });

  const engDate = now.toLocaleDateString("en-US", {
    weekday: "long",
    month: "long",
    day: "numeric",
    year: "numeric",
  });

  return (
    <nav className="clock-navbar">
     <div className="left-group">
        <div className="app-title">Track it</div>
     </div>
      <div className="right-group">
        <div className="datetime-stack">
          <div className="clock-time">{time}</div>
          <div className="eng-date">{engDate}</div>
          <div className="nepali-date">
            {nepaliDate
              ? `${nepaliDate.bsMonth} ${nepaliDate.bsDay}, ${nepaliDate.bsYear}`
              : "Loading Nepali Date..."}
          </div>
        </div>

        <button className="settings-btn" aria-label="Settings">
          ⚙
        </button>
      </div>
    </nav>
  );
}
