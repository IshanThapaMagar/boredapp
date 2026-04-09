import React, { useState } from "react";
import { format } from "date-fns";
import { invoke } from "@tauri-apps/api/core";
import Flatpickr from "react-flatpickr";
import "flatpickr/dist/themes/light.css";
import "@sajanm/nepali-date-picker/dist/nepali.datepicker.v5.0.6.min.css";
import "@sajanm/nepali-date-picker/dist/nepali.datepicker.v5.0.6.min.js";
import { Palmtree, Plus } from "lucide-react";
import { Button } from "./components/ui/button";
import { Label } from "./components/ui/label";
import { Textarea } from "./components/ui/textarea";
import { CALENDAR_TYPES, adToBs, bsToAd, formatAdDate } from "./lib/calendar";
import { toast } from "sonner";
import "./HolidayForm.css";

export function HolidayForm({ userId, calendarPreference, onSaved }) {
  const [dateRange, setDateRange] = useState([new Date(), new Date()]);
  const [bsStartDate, setBsStartDate] = useState("");
  const [bsEndDate, setBsEndDate] = useState("");
  const bsRangeInputRef = React.useRef(null);
  const [type, setType] = useState("public_holiday");
  const [notes, setNotes] = useState("");
  const [isSubmitting, setIsSubmitting] = useState(false);

  const isBsMode = calendarPreference === CALENDAR_TYPES.BS;

  React.useEffect(() => {
    const todayAd = formatAdDate(new Date());
    const todayBs = adToBs(todayAd);
    setBsStartDate(todayBs);
    setBsEndDate(todayBs);
  }, []);

  React.useEffect(() => {
    if (!isBsMode || !bsRangeInputRef.current) return;

    const input = bsRangeInputRef.current;
    const defaultValue =
      bsStartDate && bsEndDate ? `${bsStartDate} - ${bsEndDate}` : "";

    input.value = defaultValue;

    input.NepaliDatePicker({
      range: true,
      dateFormat: "YYYY-MM-DD",
      value: defaultValue,
      onSelect: (selected) => {
        if (!Array.isArray(selected) || selected.length === 0) {
          return;
        }

        const start = selected[0]?.value || "";
        const end = selected[1]?.value || start;

        setBsStartDate(start);
        setBsEndDate(end);
        input.value = end ? `${start} - ${end}` : start;
      },
    });
  }, [isBsMode]);

  const handleSubmit = async (e) => {
    e.preventDefault();

    let startDate;
    let endDate;

    if (isBsMode) {
      if (!bsStartDate || !bsEndDate) {
        toast.error("Please enter a BS date range");
        return;
      }

      try {
        startDate = new Date(`${bsToAd(bsStartDate)}T00:00:00`);
        endDate = new Date(`${bsToAd(bsEndDate)}T00:00:00`);
      } catch (err) {
        toast.error("Invalid BS date. Use YYYY-MM-DD.");
        return;
      }
    } else {
      if (!dateRange || dateRange.length === 0) {
        toast.error("Please select a date range");
        return;
      }

      startDate = dateRange[0];
      endDate = dateRange.length > 1 ? dateRange[1] : dateRange[0];
    }

    if (startDate > endDate) {
      toast.error("Start date must be before end date");
      return;
    }

    setIsSubmitting(true);
    try {
      const start = new Date(startDate);
      const end = new Date(endDate);
      const datesToLog = [];

      for (let d = new Date(start); d <= end; d.setDate(d.getDate() + 1)) {
        datesToLog.push(new Date(d));
      }

      for (const d of datesToLog) {
        const adDateStr = format(d, "yyyy-MM-dd");
        const bsDateStr = adToBs(adDateStr);

        await invoke("add_leave_log", {
          log: {
            id: null,
            user_id: userId,
            leave_date: adDateStr,
            leave_date_ad: adDateStr,
            leave_date_bs: bsDateStr,
            leave_type: type,
            notes: notes || "",
            absent_date_bs: bsDateStr,
          },
        });
      }

      setNotes("");
      toast.success(
        type === "public_holiday"
          ? "Public holiday(s) logged"
          : "Absence(s) marked",
      );
      if (onSaved) {
        onSaved();
      }
    } catch (err) {
      console.error("Failed to add leave log:", err);
      toast.error("Failed to log entry: " + err);
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <div className="glass-card rounded-2xl p-6 animate-fade-in w-full text-black">
      <div className="flex items-center gap-3 mb-6">
        <div className="p-2 rounded-xl bg-warning/10">
          <Palmtree className="w-5 h-5 text-warning" />
        </div>
        <h2 className="text-lg font-semibold">Log Leave</h2>
      </div>

      <form onSubmit={handleSubmit} className="space-y-4">
        <div className="space-y-2">
          {isBsMode ? (
            <input
              id="holidayDateRange"
              ref={bsRangeInputRef}
              type="text"
              placeholder="Select Date"
              readOnly
              className="flex h-11 w-full rounded-md border border-input bg-transparent px-3 py-1 text-base shadow-sm"
            />
          ) : (
            <Flatpickr
              id="holidayDateRange"
              value={dateRange}
              onChange={(dates) => setDateRange(dates)}
              options={{
                mode: "range",
                dateFormat: "Y-m-d",
              }}
              className="flex h-11 w-full rounded-md border border-input bg-transparent px-3 py-1 text-base shadow-sm transition-colors file:border-0 file:bg-transparent file:text-sm file:font-medium file:text-foreground placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50 md:text-sm"
            />
          )}
        </div>

        <div className="space-y-2">
          <Label className="text-sm">Type</Label>
          <div className="flex gap-2">
            <Button
              type="button"
              variant={type === "public_holiday" ? "default" : "outline"}
              onClick={() => setType("public_holiday")}
              className="flex-1 h-11 whitespace-nowrap"
              style={{ padding: "0 4px", fontSize: "0.85rem" }}
            >
              Public Holiday
            </Button>
            <Button
              type="button"
              variant={type === "half_day" ? "secondary" : "outline"}
              onClick={() => setType("half_day")}
              className="flex-1 h-11 whitespace-nowrap"
              style={{ padding: "0 4px", fontSize: "0.85rem" }}
            >
              Half Day
            </Button>
            <Button
              type="button"
              variant={type === "absent" ? "destructive" : "outline"}
              onClick={() => setType("absent")}
              className="flex-1 h-11 whitespace-nowrap"
              style={{ padding: "0 4px", fontSize: "0.85rem" }}
            >
              Absent
            </Button>
          </div>
        </div>

        <div className="space-y-2">
          <Label htmlFor="notes" className="text-sm">
            Notes (optional)
          </Label>
          <Textarea
            id="notes"
            placeholder="e.g., Reason for absence..."
            value={notes}
            onChange={(e) => setNotes(e.target.value)}
            className="min-h-[80px] resize-none"
          />
        </div>

        <Button
          type="submit"
          className="w-full h-11 flex items-center justify-center gap-2"
          disabled={isSubmitting}
        >
          <Plus className="w-4 h-4" />
          {isSubmitting ? "Logging..." : "Log Entry"}
        </Button>
      </form>
    </div>
  );
}
