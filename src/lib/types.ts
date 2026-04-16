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
  show: boolean;
  isTimer: boolean;
  buttons: boolean;
  message: string;
  onConfirm: () => void;
  onCancel: () => void;
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