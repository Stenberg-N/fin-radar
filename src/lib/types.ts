import { Editor } from "@tiptap/core";

export type ExpiresIn = "Hours1" | "Hours4" | "Hours8" | "Hours12";

export type SafeUser = {
  id: number;
  name: string;
  requires_password_reset: boolean;
  created_at: string;
  last_password_change: string;
};

export type Transaction = {
  id: number;
  user_id: number;
  category: string;
  date: string;
  description: string;
  amount: number;
  _type: string;
};

export type CalendarDay = {
  enabled: boolean;
  number: string;
  date: Date;
  isodate: string;
};

export type CalendarEvent = {
  id: number;
  user_id: number;
  isodate: string;
  title: string;
  description: string | null;
  start_time: number | null;
  end_time: number | null;
};

export type CalendarTag = {
  id: number;
  name: string;
  user_id: number;
};

export type CalendarEventWithTag = {
  event: CalendarEvent;
  tags: CalendarTag[];
};

export type CalendarEventForm = {
  isodate: string;
  title: string;
  description: string | null;
  startTimeHours: string | null;
  startTimeMinutes: string | null;
  endTimeHours: string | null;
  endTimeMinutes: string | null;
  tags: CalendarTag[];
};

export type Note = {
  id: number;
  user_id: number;
  tab_id: number;
  order_id: number;
  title: string;
  content: string;
  bg_color: string;
};

export type EditorState = {
  isTaskListActive: boolean;
  canAddNewItem: boolean;
  canIndent: boolean;
  canOutdent: boolean;
  isUnderline: boolean;
  isBold: boolean;
  isItalic: boolean;
  isBulletList: boolean;
  fontSize: string;
  textAlignment: '' | 'right' | 'center' | 'left';
};

export type FocusedNoteControls = {
  applyProperty: (command: string) => void;
  isTitleActive: boolean;
  focusedEditor: Editor;
  id: number;
} | null;

export type Tab = {
  id: number;
  user_id: number;
  order_id: number;
  title: string;
  color: string;
};

export type TabIdTitle = {
  id: number;
  title: string;
};

export type Timer = {
  id: number;
  user_id: number;
  order_id: number;
  duration: number;
  title: string;
  message?: string;
};