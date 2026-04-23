import { writable } from "svelte/store";

type Language = 'en' | 'fi';

const translations: Record<Language, Record<string, string | string[] | Array<Record<string, string>> >> = {
  'en': {
    // MAIN LAYOUT
    "cancel.button": "Cancel",
    "confirm.button": "Confirm",
    "add.button": "Add",
    "edit.button": "Edit",
    "clear.button": "Clear",
    "delete.button": "Delete",
    "commit.button": "Commit",
    "main.layout.logout": "Sign out",
    "main.layout.button.menu-toggle": "Toggle menu",
    "main.layout.view-title": ["Home", "Transactions"],
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
    "alert.logout.confirmation-question": "Are you sure you want to sign out?",
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
    "alert.transactions-table.delete.confirmation": "Are you sure you want to delete the selected transactions?",
    "alert.transactions-table.delete.success": "Successfully deleted transactions: ",
    "alert.transactions-table.delete.fail": "Failed to delete transactions!",
    "alert.transactions-table.no-changes": "No changes detected!",
    "alert.transactions-table.update.success": "Transactions successfully updated: ",
    "alert.transactions-table.update.fail": "Updating transactions failed!",
    "alert.transactions-table.save-changes.confirmation": "Do you want to save the changes?",
    "alert.transactions-table.toggle-edit.confirmation": ["Are you sure you want to exit edit mode?", "Changes will not be saved!"],

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
    "add-transaction.categories.sub-title.expenses": "Expenses",
    "add-transaction.categories.sub-title.income": "Income",
    "add-transaction.categories.expenses": [
      { "add-transaction.expense.option1": "Rent/Mortgage" },
      { "add-transaction.expense.option2": "Taxes" },
      { "add-transaction.expense.option3": "Groceries" },
      { "add-transaction.expense.option4": "Utilities" },
      { "add-transaction.expense.option5": "Transportation" },
      { "add-transaction.expense.option6": "Travel" },
      { "add-transaction.expense.option7": "Entertainment" },
      { "add-transaction.expense.option8": "Healthcare" },
      { "add-transaction.expense.option9": "Insurance" },
      { "add-transaction.expense.option10": "Subscription" },
      { "add-transaction.expense.option11": "Education" },
      { "add-transaction.expense.option12": "Other" },
    ],
    "add-transaction.categories.income": [
      { "add-transaction.income.option1": "Salary" },
      { "add-transaction.income.option2": "Freelance" },
      { "add-transaction.income.option3": "Investments" },
    ],
    "add-transaction.input.date.title": "Date",
    "add-transaction.input.description.title": "Description",
    "add-transaction.input.amount.title": "Amount",

    // CALENDAR
    "calendar.weekdays": ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"],
    "calendar.monthnames": ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"],
    "calendar.current-day.name": ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"],

    // TRANSACTIONS TABLE
    "transactions-table.thead.headers": ["ID", "Date", "Amount", "Category", "Description", "Type"],
    "transaction-table.type.expense": "Expense",
    "transaction-table.type.income": "Income",
    "transactions-table.controls.header": "What do you want to do?",
    "transactions-table-controls.paragraph": ["You have currently", "transactions selected"],
    "transactions-table.controls.notification.header.editmode": "In edit mode",
    "transactions-table.controls.note": ["Note!", "Editing does not require transactions to be selected"],
    "transactions-table.edit.button.hover-title": "Toggle editing",
    "transactions-table.save.button.hover-title": "Save changes",
  },
  'fi': {
    // MAIN LAYOUT
    "cancel.button": "Peruuta",
    "confirm.button": "Vahvista",
    "add.button": "Lisää",
    "edit.button": "Muokkaa",
    "clear.button": "Tyhjennä",
    "delete.button": "Poista",
    "commit.button": "Tallenna",
    "main.layout.logout": "Kirjaudu ulos",
    "main.layout.button.menu-toggle": "Näytä/piilota valikko",
    "main.layout.view-title": ["Koti", "Tilitapahtumat"],
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
    "alert.transactions-table.delete.confirmation": "Halutako varmasti poistaa valitsemasi tilitapahtumat?",
    "alert.transactions-table.delete.success": "Tilitapahtumia poistettu onnistuneesti: ",
    "alert.transactions-table.delete.fail": "Tilitapahtumien poistaminen epäonnistui!",
    "alert.transactions-table.no-changes": "Muutoksia ei havaittu!",
    "alert.transactions-table.update.success": "Tilitapahtumia päivitettiin onnistuneesti: ",
    "alert.transactions-table.update.fail": "Tilitapahtumien päivittäminen epäonnistui!",
    "alert.transactions-table.save-changes.confirmation": "Haluatko tallentaa muutokset?",
    "alert.transactions-table.toggle-edit.confirmation": ["Haluatko varmasti poistua editointitilasta?", "Muutoksia ei talleneta!"],

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
    "add-transaction.categories.sub-title.expenses": "Menot",
    "add-transaction.categories.sub-title.income": "Tulot",
    "add-transaction.categories.expenses": [
      { "add-transaction.expense.option1": "Vuokra/Laina" },
      { "add-transaction.expense.option2": "Verot" },
      { "add-transaction.expense.option3": "Ruoka" },
      { "add-transaction.expense.option4": "Laskut" },
      { "add-transaction.expense.option5": "Kulkeminen" },
      { "add-transaction.expense.option6": "Matkustus" },
      { "add-transaction.expense.option7": "Viihde" },
      { "add-transaction.expense.option8": "Terveydenhuolto" },
      { "add-transaction.expense.option9": "Vakuutus" },
      { "add-transaction.expense.option10": "Tilausmaksu" },
      { "add-transaction.expense.option11": "Opiskelu" },
      { "add-transaction.expense.option12": "Muu" },
    ],
    "add-transaction.categories.income": [
      { "add-transaction.income.option1": "Palkka" },
      { "add-transaction.income.option2": "Freelance" },
      { "add-transaction.income.option3": "Osingot" },
    ],
    "add-transaction.input.date.title": "Päivämäärä",
    "add-transaction.input.description.title": "Kuvaus",
    "add-transaction.input.amount.title": "Summa",

    // CALENDAR
    "calendar.weekdays": ["Ma", "Ti", "Ke", "To", "Pe", "La", "Su"],
    "calendar.monthnames": ["Tammikuu", "Helmikuu", "Maaliskuu", "Huhtikuu", "Toukokuu", "Kesäkuu", "Heinäkuu", "Elokuu", "Syyskuu", "Lokakuu", "Marraskuu", "Joulukuu"],
    "calendar.current-day.name": ["Sunnuntai", "Maanantai", "Tiistai", "Keskiviikko", "Torstai", "Perjantai", "Lauantai"],

    // TRANSACTIONS TABLE
    "transactions-table.thead.headers": ["ID", "Päivämäärä", "Summa", "Kategoria", "Kuvaus", "Tyyppi"],
    "transaction-table.type.expense": "Meno",
    "transaction-table.type.income": "Tulo",
    "transactions-table.controls.header": "Mitä haluat tehdä?",
    "transactions-table-controls.paragraph": ["Sinulla on tällä hetkellä", "tilitapahtumaa valittuna"],
    "transactions-table.controls.notification.header.editmode": "Muokkaustilassa",
    "transactions-table.controls.note": ["Huom!", "Muokkaaminen ei vaadi tilitapahtumien valitsemista"],
    "transactions-table.edit.button.hover-title": "Aloita/Lopeta editointi",
    "transactions-table.save.button.hover-title": "Tallenna muutokset",
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

export const t = { subscribe: (run: (value: Record<string, string | string[] | Array<Record<string, string>>>) => void) => lang.subscribe((lang) => run(translations[lang])) };

// "fi" is used to traverse deeper to the translations. This is used to fetch the keys which are the same on both translations.
export const getTransactionCategories = (str: string) => { return translations["fi"][str] };