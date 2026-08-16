import { writable } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";

import type { CalendarEvent, CalendarDay, CalendarEventForm, CalendarEventWithTag, CalendarTag } from "./types";
import { sendAlert } from "./alert";

// Since the event form uses startTime and endTime split into hours and minutes, and Rust expects a single number for both of these, this data structure is used to sum the values up before sending them to Rust.
type EventForm = {
  isodate: string;
  title: string;
  description: string | null;
  start_time: number | null;
  end_time: number | null;
  tags: CalendarTag[];
}

export let calendarDate = writable<Date>(new Date());
let calendarIsodate: string;
export const calendarDays = writable<CalendarDay[]>([]);
export const calendarEvents = writable<CalendarEventWithTag[]>([]);
export const calendarTags = writable<CalendarTag[]>([]);

calendarDate.subscribe((newDate) => {
  const year = newDate.getFullYear();
  const month = newDate.getMonth();
  const daysArray: CalendarDay[] = [];

  let firstDayThisMonth = new Date(year, month, 1).getDay();
  let offset = firstDayThisMonth === 0 ? 6 : firstDayThisMonth - 1;
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
  calendarIsodate = ((d: Date) => `${String(d.getFullYear())}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate())}`)(newDate);
});

//
// HELPERS
//

const validateForm = (form: CalendarEventForm) => {
  if (form.isodate.trim() === '' || form.title.trim() === '') {
    sendAlert({
      message: "alert.missing-mandatory-input",
      isTimer: true,
      buttons: false,
    });
    return false;
  }

  if (
    (form.startTimeHours !== null && form.startTimeHours?.trim() !== '') ||
    (form.startTimeMinutes !== null && form.startTimeMinutes?.trim() !== '') ||
    (form.endTimeHours !== null && form.endTimeHours?.trim() !== '') ||
    (form.endTimeMinutes !== null && form.endTimeMinutes?.trim() !== '')
  ) {
    const hourRegex = /^([0-9]|0[0-9]|1[0-9]|2[0-3])$/;
    const minuteRegex = /^([0-9]|[0-5]\d)$/;

    if (
      !hourRegex.test(form.startTimeHours as string) ||
      !minuteRegex.test(form.startTimeMinutes as string) ||
      !hourRegex.test(form.endTimeHours as string) ||
      !minuteRegex.test(form.endTimeMinutes as string)
    ) {
      sendAlert({
        message: "alert.invalid-hh-mm",
        isTimer: true,
        buttons: false,
      });
      return false;
    }
  }

  return true;
};

const craftPayload = (form: CalendarEventForm) => {
  const payload: EventForm = {
    isodate: form.isodate,
    title: form.title,
    description: form.description,
    start_time: null,
    end_time: null,
    tags: form.tags,
  };

  if (form.startTimeHours !== null && form.startTimeMinutes !== null && form.endTimeHours !== null && form.endTimeMinutes !== null) {
    const startTime: number = (parseInt(form.startTimeHours) * 3600) + (parseInt(form.startTimeMinutes) * 60);
    const endTime: number = (parseInt(form.endTimeHours) * 3600) + (parseInt(form.endTimeMinutes) * 60);

    if (startTime > endTime) {
      sendAlert({
        message: "alert.invalid-start-end-time",
        isTimer: true,
        buttons: false,
      });
      return { payload: null, success: false };
    }

    payload.start_time = startTime;
    payload.end_time = endTime;
  }

  return { payload, success: true };
};

//
//
//

export const addCalendarEvent = async (form: CalendarEventForm) => {
  let needsRefresh: boolean = false;
  if (!form) return { success: false };

  try {
    const isFormValid = validateForm(form);
    if (!isFormValid) return { success: false };

    const result = craftPayload(form);
    if (!result.success) return { success: false };

    const newEvent: CalendarEvent = await invoke('add_calendar_event', { form: result.payload });
    calendarEvents.update((currentEvents) => [...currentEvents, { event: newEvent, tags: form.tags }]);

    if (newEvent.isodate.slice(0, 7) !== calendarIsodate.slice(0, 7)) needsRefresh = true;
    if (needsRefresh) getCalendarEvents(calendarIsodate.slice(0, 7));

    return { success: true };
  } catch (error) {
    sendAlert({
      message: "alert.add-calendar-event.fail",
      isTimer: true,
      buttons: false,
    });
    return { success: false };
  }
};

export const getCalendarEvents = async (yearMonth: string) => {
  if (yearMonth.trim() === '') return { success: false };

  try {
    const result: CalendarEventWithTag[] = await invoke('get_calendar_events', { yearMonth: yearMonth });
    calendarEvents.set(result);

    return { success: true };
  } catch (error) {
    sendAlert({
      message: "alert.get-calendar-events.fail",
      isTimer: true,
      buttons: false,
    });
    return { success: false };
  }
};

export const deleteCalendarEvent = async (event: CalendarEvent) => {
  if (!event) return { success: false };

  try {
    const deletedEvent: CalendarEvent = await invoke('delete_calendar_event', { event: event });
    calendarEvents.update((currentEvents) => [...currentEvents.filter(obj => obj.event.id !== deletedEvent.id)]);

    sendAlert({
      message: "alert.delete-calendar-event.success",
      isTimer: true,
      buttons: false,
      additionalText: [deletedEvent.title],
    });
    return { success: true };
  } catch (error) {
    sendAlert({
      message: "alert.delete-calendar-event.fail",
      isTimer: true,
      buttons: false,
    });
    return { success: false };
  }
};

export const updateCalendarEvent = async (form: CalendarEventForm, event: CalendarEvent) => {
  if (!form) return { success: false };

  if (form.startTimeHours !== null && form.startTimeMinutes !== null && form.endTimeHours !== null && form.endTimeMinutes !== null) {
    const startTime: number = (parseInt(form.startTimeHours) * 3600) + (parseInt(form.startTimeMinutes) * 60);
    const endTime: number = (parseInt(form.endTimeHours) * 3600) + (parseInt(form.endTimeMinutes) * 60);

    if (
      event.isodate == form.isodate &&
      event.title === form.title &&
      event.description === form.description &&
      event.start_time === startTime &&
      event.end_time === endTime
    ) {
      sendAlert({
        message: "alert.saving.no-changes",
        isTimer: true,
        buttons: false,
      });
      return { success: false };
    }
  }

  try {
    const isFormValid = validateForm(form);
    if (!isFormValid) return { success: false };

    const result = craftPayload(form);
    if (!result.success) return { success: false };

    const updatedEvent: CalendarEvent = await invoke('update_calendar_event', { form: result.payload, event: event });
    calendarEvents.update((currentEvents) =>
      currentEvents.map((obj) => {
        return obj.event.id === updatedEvent.id ? { event: updatedEvent, tags: obj.tags } : { event: obj.event, tags: obj.tags };
      })
    );

    return { success: true };
  } catch (error) {
    sendAlert({
      message: "alert.update-calendar-event.fail",
      isTimer: true,
      buttons: false,
    });
    return { success: false };
  }
};

export const addCalendarTag = async (name: string) => {
  if (name.trim() === '' || !name) return { success: false };

  try {
    const newTag: CalendarTag = await invoke('add_calendar_tag', { name: name });
    calendarTags.update((currentTags) => [...currentTags, newTag]);

    return { success: true };
  } catch (error) {
    sendAlert({
      message: "alert.add-calendar-tag.fail",
      isTimer: true,
      buttons: false,
    });
    return { success: false };
  }
};

export const getCalendarTags = async () => {
  try {
    const tags: CalendarTag[] = await invoke('get_calendar_tags');
    calendarTags.set(tags);

    return { success: true };
  } catch (error) {
    return { success: false };
  }
};

export const deleteCalendarTag = async (tagId: number) => {
  if (!tagId) return { success: false };

  try {
    const deletedTagId = await invoke('delete_calendar_tag', { tagId: tagId });

    calendarEvents.update((currentEvents) =>
      currentEvents.map((obj) => {
        return { event: obj.event, tags: obj.tags ? obj.tags.filter(t => t.id !== deletedTagId) : [] };
      })
    );

    return { success: true };
  } catch (error) {
    sendAlert({
      message: "alert.delete-calendar-tag.fail",
      isTimer: true,
      buttons: false,
    });
    return { success: false };
  }
};