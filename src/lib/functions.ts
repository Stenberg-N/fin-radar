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