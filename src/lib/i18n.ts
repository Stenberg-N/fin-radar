import { writable } from "svelte/store";

type Language = 'en' | 'fi';

const translations: Record<Language, Record<string, string | string[]>> = {
  'en': {
    // MAIN LAYOUT
    "main.layout.logout": "Logout",
    "main.layout.button.menu-toggle": "Toggle menu",
    "main.layout.view-title": ["Home"],
    "language.button.title": "Vaihda suomeen",

    // SETTINGS BANNER
    "settings-banner.title": "Menu",
    "settings-banner.button.delete-user": "Delete user",
    "settings-banner.button.open-data": "Open data",
    "settings-banner.button.backup-db": "Backup database",
    "settings-banner.button.change-password": "Change password",

    // ALERTS
    "alert.button.confirm": "Confirm",
    "alert.button.cancel": "Cancel",
    "alert.password.mismatch": "Passwords are not matching!",
    "alert.password.requirements-not-met": [
      "The password does not meet the requirements!",
      "The requirements:",
      "At least 10 characters long",
      "Have no spaces",
      "Have at least one number",
      "Have at least one uppercase and lowercase letter",
      "Have at least one special character",
    ],
    "alert.password.change.success": "Password changed successfully!",
    "alert.password.change.fail": "Password change failed!",
    "alert.registration.message.fail": "Account creation failed!",
    "alert.registration.message.success": "Account created!",
    "alert.login.message.fail": "Invalid login information!",
    "alert.delete-user.confirmation-question": "Are you sure you want to delete your account?",
    "alert.delete-user.message.fail": "Failed to delete user",
    "alert.delete-user.message.success": "User deleted successfully!",
    "alert.logout.confirmation-question": "Are you sure you want to logout?",

    // REGISTRATION & LOGIN
    "form.login.title": "Login",
    "form.login.button": "Login",
    "form.register.title": "Registration",
    "form.register.button": "Create account",
    "form.username.title": "Username",
    "form.password.title": "Password",
    "form.confirm-password.title": "Confirm password",
    "form.no-account.question": "No account?",
    "form.no-account.button": "Create one!",
    "form.already-account.question": "Already have an account?",
    "form.already-account.button": "Login!",
    "form.password-visibility.hide": "Hide",
    "form.password-visibility.show": "Show",

    // CHANGE PASSWORD
    "form.change-password.title": "Change password",
    "form.change-password.current-password.title": "Current password",
    "form.change-password.new-password.title": "New password",
    "form.change-password.confirm-new-password.title": "Confirm new password",
    "form.change-password.button.confirm": "Confirm",

    title: "Welcome"
  },
  'fi': {
    // MAIN LAYOUT
    "main.layout.logout": "Kirjaudu ulos",
    "main.layout.button.menu-toggle": "Näytä/piilota valikko",
    "main.layout.view-title": ["Koti"],
    "language.button.title": "Switch to English",

    // SETTINGS BANNER
    "settings-banner.title": "Valikko",
    "settings-banner.button.delete-user": "Poista tili",
    "settings-banner.button.open-data": "Avaa data",
    "settings-banner.button.backup-db": "Varmuuskopioi data",
    "settings-banner.button.change-password": "Vaihda salasana",

    // ALERTS
    "alert.button.confirm": "Vahvista",
    "alert.button.cancel": "Peruuta",
    "alert.password.mismatch": "Salasanat eivät täsmää!",
    "alert.password.requirements-not-met": [
      "Salasana ei täytä vaatimuksia!",
      "Vaatimukset:",
      "Vähintään 10 merkkiä pitkä",
      "Ei saa sisältää välilyöntejä",
      "Sisältää ainakin yhden numeron",
      "Sisältää ainakin yhden ison ja pienen kirjaimen",
      "Sisältää ainakin yhden erityisen merkin, esim. !, $",
    ],
    "alert.password.change.success": "Salasana vaihto onnistui!",
    "alert.password.change.fail": "Salasanan vaihto epäonnistui!",
    "alert.registration.message.fail": "Tilin luominen epäonnistui!",
    "alert.registration.message.success": "Tili luotu!",
    "alert.login.message.fail": "Kirjautumistiedot ovat väärät!",
    "alert.delete-user.confirmation-question": "Haluatko varmasti poistaa tilisi?",
    "alert.delete-user.message.fail": "Tilin poistaminen epäonnistui",
    "alert.delete-user.message.success": "Tili poistettiin!",
    "alert.logout.confirmation-question": "Haluatko varmasti kirjautua ulos?",

    // REGISTRATION & LOGIN
    "form.login.title": "Kirjautuminen",
    "form.login.button": "Kirjaudu",
    "form.register.title": "Tilinluonti",
    "form.register.button": "Luo tili",
    "form.username.title": "Käyttäjänimi",
    "form.password.title": "Salasana",
    "form.confirm-password.title": "Vahvista salasana",
    "form.no-account.question": "Ei vielä tiliä?",
    "form.no-account.button": "Luo se!",
    "form.already-account.question": "On jo tili?",
    "form.already-account.button": "Kirjaudu!",
    "form.password-visibility.hide": "Piilota",
    "form.password-visibility.show": "Näytä",

    // CHANGE PASSWORD
    "form.change-password.title": "Vaihda salasana",
    "form.change-password.current-password.title": "Nykyinen salasana",
    "form.change-password.new-password.title": "Uusi salasana",
    "form.change-password.confirm-new-password.title": "Vahvista uusi salasana",
    "form.change-password.button.confirm": "Vahvista",

    title: "Tervetuloa"
  }
}

const isValidLanguage = (lang: string): lang is Language => ['en', 'fi'].includes(lang);

const getInitialLanguage = () => {
  const saved = localStorage.getItem('lang');

  if (saved && isValidLanguage(saved)) { 
    return saved;
  } else {
    const lang = navigator.language.split('-')[0];
    return isValidLanguage(lang) ? lang : 'en';
  }
}

const createLangStore = () => {
  const { subscribe, set, update } = writable<Language>(getInitialLanguage());
  return { subscribe, set: (lang: Language) => { localStorage.setItem('lang', lang); set(lang); }, update };
}

export const lang = createLangStore();

export const t = { subscribe: (run: (value: Record<string, string | string[]>) => void) => lang.subscribe((lang) => run(translations[lang])) };