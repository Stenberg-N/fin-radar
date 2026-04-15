import { writable } from "svelte/store";

type Language = 'en' | 'fi';

const translations: Record<Language, Record<string, string | string[]>> = {
  'en': {
    // MAIN LAYOUT
    "cancel.button": "Cancel",
    "confirm.button": "Confirm",
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
    "alert.password.recover.missing-info": "Missing required fields!",
    "alert.password.recover.fail": "Account recovery failed!",
    "alert.password.recover.cancel-confirmation-question": "Cancel password recovery?",
    "alert.password-recover.cancel.success": "Password recovery cancelled!",
    "alert.password-recover.cancel.fail": "An error occurred!",
    "alert.registration.message.fail": "Account creation failed!",
    "alert.registration.message.success": "Account created!",
    "alert.login.message.fail": "Invalid login information!",
    "alert.delete-user.confirmation-question": "Are you sure you want to delete your account?",
    "alert.delete-user.message.fail": "Failed to delete user",
    "alert.delete-user.message.success": "User deleted successfully!",
    "alert.logout.confirmation-question": "Are you sure you want to logout?",
    "alert.copy-text.fail": "Failed to copy!",
    "alert.copy-text.success": "Copied!",
    "alert.backup-db.success": "Database backup successful!",
    "alert.backup-db.fail": "Database backup failed!",
    "alert.add-transaction.amount.comma": "Please use dot ( . ) as a decimal point!",
    "alert.add-transaction.amount.minus": "Please mark expenses without a minus sign!",
    "alert.add-transaction.date.input": "Only numbers and the hyphen ( - ) are allowed! Mark the date in DD-MM-YYYY format",
    "alert.add-transaction.cancel.question": "Are you sure you want to clear all the form's fields?",
    "alert.add-transaction.no-category": "No category selected!",
    "alert.add-transaction.input-missing": "Please fill all the fields!",
    "alert.add-transaction.success": "Transaction added successfully!",
    "alert.add-transaction.fail": "Failed to add transaction!",

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
    "recovery-key.modal.title": "Account recovery key",
    "recovery-key.modal.paragraph": "Save the following code in case you forget your password to recover your account!",
    "recovery-key.modal.confirm": "Got it!",
    "form.forgot-password.question": "Forgot your password?",
    "form.forgot-password.button": "Change it",
    "form.forgot-password.title": "Account recovery",
    "form.forgot-password.paragraph": "Please provide the username of the account and your recovery key",
    "form.forgot-password.recovery-key.title": "Recovery key",
    "form.forgot-password.button.confirm": "Confirm",

    // CHANGE PASSWORD
    "form.change-password.title": "Change password",
    "form.change-password.current-password.title": "Current password",
    "form.change-password.new-password.title": "New password",
    "form.change-password.confirm-new-password.title": "Confirm new password",
    "form.change-password.cancel-recovery.message": ["NOTE!", "If you remembered your password again, DO NOT reset your password. Your recovery key WILL be used and you will not have another one to reset your password in the future if you happen to forget it."],

    // HOME PAGE
    "add-transaction-title": "Add transaction",
    "add-transaction.categories.title": "Categories",
    "add-transaction.categories.sub-title.expenses": "Expenses",
    "add-transaction.categories.sub-title.income": "Income",
    "add-transaction.categories.option1": "Rent/Mortgage",
    "add-transaction.categories.option2": "Taxes",
    "add-transaction.categories.option3": "Groceries",
    "add-transaction.categories.option4": "Utilities",
    "add-transaction.categories.option5": "Transportation",
    "add-transaction.categories.option6": "Travel",
    "add-transaction.categories.option7": "Entertainment",
    "add-transaction.categories.option8": "Healthcare",
    "add-transaction.categories.option9": "Insurance",
    "add-transaction.categories.option10": "Subscription",
    "add-transaction.categories.option11": "Education",
    "add-transaction.categories.option12": "Other",
    "add-transaction.categories.option13": "Salary",
    "add-transaction.categories.option14": "Freelance",
    "add-transaction.categories.option15": "Investments",
    "add-transaction.input.date.title": "Date",
    "add-transaction.input.description.title": "Description",
    "add-transaction.input.amount.title": "Amount",

    // CALENDAR
    "calendar.weekdays": ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"],
    "calendar.monthnames": ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"],
    "calendar.current-day.name": ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"],
  },
  'fi': {
    // MAIN LAYOUT
    "cancel.button": "Peruuta",
    "confirm.button": "Vahvista",
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
    "alert.password.recover.missing-info": "Pakollisia kenttiä puuttuu!",
    "alert.password.recover.fail": "Tilinpalautus epäonnistui!",
    "alert.password.recover.cancel-confirmation-question": "Peruuta salasanan palauttaminen?",
    "alert.password-recover.cancel.success": "Salasanan palauttaminen peruttu!",
    "alert.password-recover.cancel.fail": "Tapahtui virhe!",
    "alert.registration.message.fail": "Tilin luominen epäonnistui!",
    "alert.registration.message.success": "Tili luotu!",
    "alert.login.message.fail": "Kirjautumistiedot ovat väärät!",
    "alert.delete-user.confirmation-question": "Haluatko varmasti poistaa tilisi?",
    "alert.delete-user.message.fail": "Tilin poistaminen epäonnistui",
    "alert.delete-user.message.success": "Tili poistettiin!",
    "alert.logout.confirmation-question": "Haluatko varmasti kirjautua ulos?",
    "alert.copy-text.fail": "Kopiointi epäonnistui!",
    "alert.copy-text.success": "Kopioitu!",
    "alert.backup-db.success": "Datan varmuuskopiointi onnistui!",
    "alert.backup-db.fail": "Datan varmuuskopiointi epäonnistui!",
    "alert.add-transaction.amount.comma": "Käytä pistettä ( . ) desimaalimerkkinä!",
    "alert.add-transaction.amount.minus": "Merkkaa menon määrä ilman miinus merkkiä!",
    "alert.add-transaction.date.input": "Vain numerot ja viiva ( - ) ovat sallittuja! Kirjoita päivämäärä PP-KK-VVVV muodossa",
    "alert.add-transaction.cancel.question": "Haluatko varmasti tyhjentää kentät?",
    "alert.add-transaction.no-category": "Kategoriaa ei ole valittu!",
    "alert.add-transaction.input-missing": "Joitain kenttiä ei ole täytetty!",
    "alert.add-transaction.success": "Tilitapahtuma lisätty onnistuneesti!",
    "alert.add-transaction.fail": "Tilitapahtuman käsittelyssä tapahtui virhe!",

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
    "recovery-key.modal.title": "Tilin palautuskoodi",
    "recovery-key.modal.paragraph": "Tallenna alla oleva koodi jos satut unohtamaan salasanasi, jotta voit palauttaa tilisi!",
    "recovery-key.modal.confirm": "Selvä!",
    "form.forgot-password.question": "Unohditko salasanasi?",
    "form.forgot-password.button": "Vaihda se",
    "form.forgot-password.title": "Tilinpalautus",
    "form.forgot-password.paragraph": "Syötä tilin käyttäjänimi ja palautuskoodi",
    "form.forgot-password.recovery-key.title": "Palautuskoodi",

    // CHANGE PASSWORD
    "form.change-password.title": "Vaihda salasana",
    "form.change-password.current-password.title": "Nykyinen salasana",
    "form.change-password.new-password.title": "Uusi salasana",
    "form.change-password.confirm-new-password.title": "Vahvista uusi salasana",
    "form.change-password.cancel-recovery.message": ["HUOM!", "Jos muistit salasanasi uudelleen, mutta et asettanut uutta salasanaa palautuskoodin käytön jälkeen, älä aseta uutta salasanaa. Palautuskoodisi tullaan käyttämään tässä tapauksessa ja sinulla ei ole enää palautuskoodia käytettävissä, jos satut tulevaisuudessa unohtamaan salasanasi."],

    // HOME PAGE
    "add-transaction-title": "Lisää tilitapahtuma",
    "add-transaction.categories.title": "Kategoriat",
    "add-transaction.categories.sub-title.expenses": "Menot",
    "add-transaction.categories.sub-title.income": "Tulot",
    "add-transaction.categories.option1": "Vuokra/Laina",
    "add-transaction.categories.option2": "Verot",
    "add-transaction.categories.option3": "Ruoka",
    "add-transaction.categories.option4": "Laskut",
    "add-transaction.categories.option5": "Kulkeminen",
    "add-transaction.categories.option6": "Matkustus",
    "add-transaction.categories.option7": "Viihde",
    "add-transaction.categories.option8": "Terveydenhuolto",
    "add-transaction.categories.option9": "Vakuutus",
    "add-transaction.categories.option10": "Tilausmaksu",
    "add-transaction.categories.option11": "Opiskelu",
    "add-transaction.categories.option12": "Muu",
    "add-transaction.categories.option13": "Palkka",
    "add-transaction.categories.option14": "Freelance",
    "add-transaction.categories.option15": "Osingot",
    "add-transaction.input.date.title": "Päivämäärä",
    "add-transaction.input.description.title": "Kuvaus",
    "add-transaction.input.amount.title": "Summa",

    // CALENDAR
    "calendar.weekdays": ["Ma", "Ti", "Ke", "To", "Pe", "La", "Su"],
    "calendar.monthnames": ["Tammikuu", "Helmikuu", "Maaliskuu", "Huhtikuu", "Toukokuu", "Kesäkuu", "Heinäkuu", "Elokuu", "Syyskuu", "Lokakuu", "Marraskuu", "Joulukuu"],
    "calendar.current-day.name": ["Sunnuntai", "Maanantai", "Tiistai", "Keskiviikko", "Torstai", "Perjantai", "Lauantai"],
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