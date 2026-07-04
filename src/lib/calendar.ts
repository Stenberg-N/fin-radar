import { writable } from "svelte/store";

import type { CalendarDay } from "./types";

export let calendarDate = writable<Date>(new Date());
export const calendarDays = writable<CalendarDay[]>([]);

calendarDate.subscribe((newDate) => {
  const year = newDate.getFullYear();
  const month = newDate.getMonth();
  const daysArray: CalendarDay[] = [];

  let firstDayLastMonth = new Date(year, month, 1).getDay();
  let offset = firstDayLastMonth === 0 ? 6 : firstDayLastMonth - 1;
  let currentMonthDays = new Date(year, month + 1, 0).getDate();
  let lastMonthDays = new Date(year, month, 0).getDate();
  let previousMonth = month === 0 ? 11 : month - 1;

  for (let i = lastMonthDays - offset; i < lastMonthDays; i++) {
    let day = new Date(previousMonth === 11 ? year - 1 : year, previousMonth, i + 1);
    let isodate = `${String(day.getFullYear())}-${String(day.getMonth() + 1).padStart(2, '0')}-${String(day.getDate()).padStart(2, '0')}`;

    daysArray.push({ enabled: false, number: '' + (i + 1), date: day, isodate: isodate });
  }

  for (let i = 0; i < currentMonthDays; i++) {
    let day = new Date(year, month, i + 1);
    let isodate = `${String(day.getFullYear())}-${String(day.getMonth() + 1).padStart(2, '0')}-${String(day.getDate()).padStart(2, '0')}`;

    daysArray.push({ enabled: true, number: '' + (i + 1), date: day, isodate: isodate });
  }

  let i = 0;
  while (daysArray.length < 42) {
    let day = new Date(month === 11 ? year + 1 : year, (month + 1)%12, i + 1);
    let isodate = `${String(day.getFullYear())}-${String(day.getMonth() + 1).padStart(2, '0')}-${String(day.getDate()).padStart(2, '0')}`;

    daysArray.push({ enabled: false, number: '' + (i + 1), date: day, isodate: isodate });
    i++;
  }

  calendarDays.set(daysArray);
});