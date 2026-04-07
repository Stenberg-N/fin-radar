export type User = {
  id: number;
  name: string;
  password: string;
  requires_password_reset: boolean;
}

export type Transaction = {
  id: number;
  category: string;
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