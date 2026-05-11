export type User = {
  id: number;
  name: string;
  password: string;
  requires_password_reset: boolean;
}

export type Transaction = {
  id: number;
  user_id: number;
  category: string;
  date: string;
  description: string;
  amount: number;
  _type: string;
}

export type Alert = {
  id: number;
  isTimer: boolean;
  buttons: boolean;
  message: string;
  onConfirm: () => void;
  onCancel: () => void;
  value: string;
}

export type CalendarDay = {
  enabled: boolean;
  number: string;
  date: Date;
  isodate: string;
}

export type ViewStore = {
  isMenu: boolean;
  isChangePwOverlay: boolean;
  isRecoveryView: boolean;
  isCalendar: boolean;
}

export type Note = {
  id: number;
  user_id: number;
  tab_id: number;
  order_id: number;
  title: string;
  content: string;
}

export type Tab = {
  id: number;
  user_id: number;
  order_id: number;
  title: string;
}