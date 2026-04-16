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

export const zip = (str: string[], keys: string[], ...arrays: string[][]): Record<string, string>[] => {
  if (keys.length !== arrays.length) { console.error("The amount of keys must match the amount of arrays"); return []; }

  const minLength = Math.min(...arrays.map(arr => arr.length));
  return Array.from({ length: minLength }, (_, i) => {
    const dict: Record<string, string> = {};
    arrays.forEach((arr, index) => {
      dict[str[0]] = str[1];
      dict[keys[index]] = arr[i];
    });
    return dict;
  });
};