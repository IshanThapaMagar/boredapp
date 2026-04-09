import NepaliDate from "nepali-date-converter";

export const CALENDAR_TYPES = {
  AD: "ad",
  BS: "bs",
};

export const pad2 = (value) => `${value}`.padStart(2, "0");

export const formatAdDate = (date) => {
  const year = date.getFullYear();
  const month = pad2(date.getMonth() + 1);
  const day = pad2(date.getDate());
  return `${year}-${month}-${day}`;
};

export const adToBs = (adDate) => {
  if (!adDate) return "";
  const nd = new NepaliDate(new Date(`${adDate}T00:00:00`));
  return nd.format("YYYY-MM-DD");
};

export const bsToAd = (bsDate) => {
  if (!bsDate) return "";
  const parsed = new NepaliDate(bsDate).toJsDate();
  return formatAdDate(parsed);
};

export const toDatePayload = (value, calendarPreference) => {
  if (!value) {
    return {
      ad: "",
      bs: "",
    };
  }

  if (calendarPreference === CALENDAR_TYPES.BS) {
    const ad = bsToAd(value);
    return {
      ad,
      bs: value,
    };
  }

  return {
    ad: value,
    bs: adToBs(value),
  };
};

export const getTodayByPreference = (calendarPreference) => {
  const todayAd = formatAdDate(new Date());
  if (calendarPreference === CALENDAR_TYPES.BS) {
    return adToBs(todayAd);
  }

  return todayAd;
};

export const getDisplayDate = (record, calendarPreference) => {
  if (calendarPreference === CALENDAR_TYPES.BS) {
    return record?.attendance_date_bs || record?.leave_date_bs || "-";
  }

  return record?.attendance_date_ad || record?.leave_date_ad || record?.date || record?.leave_date || "-";
};

export const NEPALI_MONTHS = [
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

export const getBsMonthRange = (year, month) => {
  // month is 1-indexed (1-12)
  const start = new NepaliDate(year, month - 1, 1).toJsDate();
  // To get the end of the month, we can go to the 1st of next month and subtract 1 day
  // Or just use a large enough number and nepali-date-converter might handle it?
  // Actually, let's just find out how many days are in that BS month.
  // nepali-date-converter doesn't seem to have a direct getDaysInMonth.
  // But we can go to day 32 and see if it rolls over, or just go to 1st of next month - 1 day.
  
  let nextYear = year;
  let nextMonth = month; // nextMonth index (0-indexed) would be month
  if (nextMonth > 11) {
    nextMonth = 0;
    nextYear++;
  }
  
  const end = new Date(new NepaliDate(nextYear, nextMonth, 1).toJsDate());
  end.setDate(end.getDate() - 1);
  
  return {
    from: formatAdDate(start),
    to: formatAdDate(end)
  };
};
