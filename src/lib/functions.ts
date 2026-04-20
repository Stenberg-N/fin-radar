export const validatePassword = (pw: string) => {
  const hasMinLength = pw.length >= 10;
  const noSpaces = !/\s/.test(pw);
  const hasNumbers = /\d/.test(pw);
  const hasUpperCase = /\p{Lu}+/gu.test(pw);
  const hasLowerCase = /\p{Ll}+/gu.test(pw);
  const hasSpecialChar = /[=\]!@#$€£¤%^&*(){}\[,.?+<>~§'":|/\\-]/.test(pw);

  return {
    isValid: hasMinLength && noSpaces && hasNumbers && hasUpperCase && hasLowerCase && hasSpecialChar,
  };
};

export const handleClickOutside = (
  node: HTMLElement,
  options: {
    getIgnoredElements: () => (HTMLButtonElement | HTMLDivElement | null)[];
    onOutsideClick: () => void;
    additionalElements?: (HTMLButtonElement | HTMLDivElement | null)[];
  }
) => {
  const { getIgnoredElements, onOutsideClick, additionalElements } = options;
  const handleClick = (e: MouseEvent) => {
    const target = e.target as Node;
    const ignored = getIgnoredElements();
    additionalElements?.forEach(el => { ignored.push(el); });

    if (node.contains(target)) return;

    for (const el of ignored) {
      if (el?.contains(target)) return;
    }

    onOutsideClick();
  };

  document.addEventListener('click', handleClick, true);

  return { destroy() { document.removeEventListener('click', handleClick, true); } };
};

export const togglePasswordVisibility = (button: EventTarget | null) => {
  if (!button) return;

  const node = button as HTMLButtonElement;
  const passwordInput = node.previousElementSibling as HTMLInputElement | null;
  const img = node.firstChild as HTMLImageElement | null;

  if (passwordInput && img) {
    const isPassword = passwordInput.type === "password";
    passwordInput.type = isPassword ? "text" : "password";
    img.src = isPassword ? "/eye-hidden.svg" : "/eye-visible.svg";
  }
};