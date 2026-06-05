<script lang="ts">
  import { onMount, getContext, onDestroy } from "svelte";
  import { fade, fly } from "svelte/transition";
  import { cubicInOut } from "svelte/easing";
  import { goto, beforeNavigate } from "$app/navigation";
  import { load, Store } from '@tauri-apps/plugin-store';

  import { lang, t } from "$lib/i18n";
  import { user } from "$lib/user";
  import { createNote, createTab, getNotes, getTabs, notes, tabs, updateTab, deleteTab, updateTabColor, stopNoteBatchFlush, startNoteBatchFlush } from "$lib/notes";
  import { sendAlert } from "$lib/alert";
  import { handleClickOutside, handleHorizontalScroll } from "$lib/functions";

  import NoteComponent from "../../components/notes/Note.svelte";
  import ContextMenu from "../../components/notes/ContextMenu.svelte";
  import ToggleSwitch from "../../components/ToggleSwitch.svelte";

  // MAIN
  let displayNotes = $derived($notes.filter(n => n.tab_id === currentTabId));
  let displayTabs = $derived($tabs);

  // WITHOUT CLASSIFICATION
  let windowInnerHeight = $derived.by(() => { return windowDimensions.getWindowHeight(); });
  let isDeleteModalVisible = $state<boolean>(false);
  let isColorOptions = $state<boolean>(false);
  let pendingNavigation = $state<string | null>(null);
  let focusedNoteControls = $state<{
    applyProperty: (command: string) => void;
    isTitleActive: boolean;
  } | null>(null);
  let fontSize = $state<string>('');
  let isColorForNotes = $state<boolean>(false);
  let isColorForText = $state<boolean>(false);
  let noteColor = $state<string | null>(null);
  let zoomedNoteId = $state<number | null>(null);
  let zoomedNote = $derived(displayNotes.find(n => n.id === zoomedNoteId));
  let isNoteUpdating = $state<boolean>(false);

  // CURSOR & POSITION
  let cursorTimer: number;
  let cursorX = $state(0);
  let cursorY = $state(0);
  let contextMenuCursorPosX = $state<number>(0);
  let contextMenuCursorPosY = $state<number>(0);
  let colorOptionsCursorPosX = $state<number>(0);
  let colorOptionsCursorPosY = $state<number>(0);

  // STORE
  let store: Store;
  let noteColumns = $state<number | null>(null);
  let noteHeight = $state<number | null>(null);
  let noteBgColor = $state<number | null>(null);
  const mainContainerHeight = $derived(windowInnerHeight - 254);
  const noteGridRows = $derived(noteHeight === 1 ? mainContainerHeight : (mainContainerHeight - 20) / 2); 

  // ADDITIONAL IGNORABLE ELEMENTS FOR HANDLEOUTSIDECLICK
  let toggleColorsButton = $state<HTMLButtonElement | null>(null);
  let toggleHeadingOptions = $state<HTMLButtonElement | null>(null);
  let toggleColorsEditorButton = $state<HTMLButtonElement | null>(null);
  let toolBarMainButtonRefs = $state<HTMLButtonElement[]>([]);
  let toolBarEditorButtonRefs = $state<HTMLButtonElement[]>([]);

  // TABS
  let currentTabId = $state<number | null>(null);
  let editingTabId = $state<number | null>(null);
  let editingTabTitle = $derived.by(() => { const tab = $tabs.find(t => t.id === editingTabId); return tab ? tab.title : 'Unknown title' });
  let editingTabInput = $state<HTMLInputElement | null>(null);

  // CONTEXT MENU
  let contextMenuTabId = $state<number | null>(null);
  let isContextMenu = $state<boolean>(false);

  // TOP TOOLBAR
  const toolBarMainButtons = [
    { titleKey: "add.button", icon: "/plus.svg", command: async () => await addNote() },
    { titleKey: "delete.button", icon: "/trash-can.svg", command: () => {
      isDeleteModalVisible = true;
      sendAlert("alert.delete-tab.confirmation", false, true, async () => { if (currentTabId !== null) { await handleTabDelete(currentTabId); currentTabId = null; } else {} }, () => isDeleteModalVisible = false, $tabs.find(t => t.id === currentTabId)?.title);
    }},
    { titleKey: "notes.change-tab-color", icon: "/palette.svg", command: () => { handleColorMenu(); isColorForNotes = false; } },
  ];
  const toolBarSelectElements = [
    { titleKey: "notes.columns-amount", options: ["1", "2", "3", "4", "5"], get: () => noteColumns, set: (value: number) => noteColumns = value },
    { titleKey: "notes.note-height", options: ["100%", "50%"], get: () => noteHeight, set: (value: number) => noteHeight = value },
    // The array below has two instances of the same value inside its options, since that value is the key that is used to fetch values from the translations store.
    // Since there are two options, light and dark, there needs to be two instances of the said key for rendering the option elements inside the select tag.
    { titleKey: "notes.note-bg-color", options: ["notes.bg-color-options", "notes.bg-color-options"], get: () => noteBgColor, set: (value: number) => noteBgColor = value },
  ];
  const toolBarEditorButtons = [
    { name: "heading", icon: "/heading.svg" },
    { name: "underline", icon: "/underline.svg" },
    { name: "bold", icon: "/bold.svg" },
    { name: "italic", icon: "/italic.svg" },
    { name: "bullet-list", icon: "/bulleted-list.svg" },
    { name: "align-left", icon: "/align-left.svg" },
    { name: "align-center", icon: "/align-center.svg" },
    { name: "align-right", icon: "/align-right.svg" },
  ];

  const availableColors = [
    // DIMMER
    { value: "transparent", title: ["No color", "Ei väriä"]},
    { value: "black", title: ["Black", "Musta"] },
    { value: "rgba(200, 200, 200, 1)", title: ["White", "Valkoinen"]},
    { value: "rgba(113, 45, 255, 0.25)", title: ["Purple", "Purppura"] },
    { value: "rgba(255, 70, 70, 0.25)", title: ["Red", "Punainen"] },
    { value: "rgba(255, 0, 255, 0.25)", title: ["Pink", "Pinkki"] },
    { value: "rgba(255, 150, 72, 0.25)", title: ["Orange", "Oranssi"] },
    { value: "rgba(255, 220, 0, 0.25)", title: ["Yellow", "Keltainen"] },
    { value: "rgba(94, 255, 94, 0.25)", title: ["Green", "Vihreä"] },
    { value: "rgba(215, 255, 0, 0.25)", title: ["Lime", "Lime"] },
    { value: "rgba(0, 255, 240, 0.25)", title: ["Turquoise", "Turkoosi"] },
    { value: "rgba(0, 140, 255, 0.25)", title: ["Blue", "Sininen"] },

    // BRIGHTER
    { value: "#f6f6f6", title: ["White", "Valkoinen"]},
    { value: "rgba(113, 45, 255, 1)", title: ["Purple", "Purppura"] },
    { value: "rgba(255, 70, 70, 1)", title: ["Red", "Punainen"] },
    { value: "rgba(255, 0, 255, 1)", title: ["Pink", "Pinkki"] },
    { value: "rgba(255, 150, 72, 1)", title: ["Orange", "Oranssi"] },
    { value: "rgba(255, 220, 0, 1)", title: ["Yellow", "Keltainen"] },
    { value: "rgba(94, 255, 94, 1)", title: ["Green", "Vihreä"] },
    { value: "rgba(215, 255, 0, 1)", title: ["Lime", "Lime"] },
    { value: "rgba(0, 255, 240, 1)", title: ["Turquoise", "Turkoosi"] },
    { value: "rgba(0, 140, 255, 1)", title: ["Blue", "Sininen"] },
  ];

  onMount(() => {
    (async () => {
      await getTabs();
      startNoteBatchFlush();
      store = await load('note-preferences.json', { defaults: { autoSave: false } });
      noteColumns = await store.get<number | null>('note-columns') ?? 4;
      noteHeight = await store.get<number | null>('note-height') ?? 1;
      noteBgColor = await store.get<number | null>('note-bg-color') ?? 1;
      window.addEventListener('mousemove', updateCursorPos);
      return () => {
        if (cursorTimer) clearTimeout(cursorTimer);
        removeEventListener('mousemove', updateCursorPos);
      };
    })();
  });

  onDestroy(() => {
    removeEventListener('mousemove', updateCursorPos);
    (async () => await stopNoteBatchFlush())();
  });

  beforeNavigate(({ to, cancel }) => {
    if (!to || !isNoteUpdating) return;

    cancel();
    pendingNavigation = to.url.pathname;
    sendAlert("alert.notes.unsaved-changes", true, false);
  });

  $effect(() => {
    if (pendingNavigation !== null && !isNoteUpdating) {
      goto(pendingNavigation);
      pendingNavigation = null;
    }
  });

  $effect(() => {
    const _currentTabId = currentTabId;
    if (_currentTabId !== null) {
      const timer = setTimeout(() => {
        (async () => await getNotes( _currentTabId))();
      }, 200);

      return () => clearTimeout(timer);
    }
  });

  // STORE SAVE EFFECTS
  $effect(() => {
    if (noteColumns !== null && store) {
      (async () => {
        await store.set('note-columns', noteColumns);
        await store.save();
      })();
    }
  });
  $effect(() => {
    if (noteHeight !== null && store) {
      (async () => {
        await store.set('note-height', noteHeight);
        await store.save();
      })();
    }
  });
  $effect(() => {
    if (noteBgColor !== null && store) {
      (async () => {
        await store.set('note-bg-color', noteBgColor);
        await store.save();
      })();
    }
  });

  // Used to collect toolbar's button references and bind the button for showing heading options to toggleHeadingOptions and bind the button for color options to toggleColorsButton,
  // and pass those to handleClickOutside to be ignored, since Svelte's bind:this doesn't allow conditional expressions.
  $effect(() => {
    if (toolBarMainButtonRefs[2]) toggleColorsButton = toolBarMainButtonRefs[2];
  });

  $effect(() => {
    if (toolBarEditorButtonRefs[0]) toggleHeadingOptions = toolBarEditorButtonRefs[0];
  });

  /***********************************************************************************************************************************\
  |
  | Context, Helper & Wrapper functions
  |
  \***********************************************************************************************************************************/
  const getIgnoredElements = getContext<() => (HTMLButtonElement | HTMLDivElement | null)[]>('ignoredElements');
  const windowDimensions = getContext<{ getWindowHeight: () => number, getWindowWidth: () => number }>('windowDimensions');
  const handleOutsideClick = () => { isColorOptions = false };
  const changeNoteColor = (color: string) => {
    noteColor = color;
    isColorForText
    ? focusedNoteControls?.applyProperty('fore-color')
    : focusedNoteControls?.applyProperty('bg-color');
  };

  const updateCursorPos = (e: MouseEvent) => {
    if (cursorTimer) clearTimeout(cursorTimer);

    cursorTimer = setTimeout(() => {
      cursorX = e.clientX;
      cursorY = e.clientY;
    }, 10);
  };

  const handleTabEditStart = (contextmenu?: boolean) => {
    if (contextmenu) {
      editingTabId = contextMenuTabId;
      isContextMenu = false;
    }
    else editingTabId = currentTabId;
  };

  const handleContextMenu = (tabId: number) => {
    contextMenuTabId = tabId;
    isContextMenu = true;
    contextMenuCursorPosX = cursorX - 390;
    contextMenuCursorPosY = cursorY - 234;
  };

  const handleContextMenuDelete = () => {
    isDeleteModalVisible = true;
    isContextMenu = false;
    sendAlert("alert.delete-tab.confirmation", false, true,
      async () => { if (contextMenuTabId !== null) { await handleTabDelete(contextMenuTabId); } else {} },
      () => { isDeleteModalVisible = false; contextMenuTabId = null; },
      $tabs.find(t => t.id === contextMenuTabId)?.title
    );
  };

  const handleContextMenuTabColor = async (color: string) => {
    if (!$tabs.some(t => t.id === contextMenuTabId) || contextMenuTabId === null) return;
    const result = await updateTabColor(contextMenuTabId, color);
    if (!result.success) sendAlert("alert.tab-color-update.fail", true, false);
  };

  const handleColorMenu = () => {
    colorOptionsCursorPosX = cursorX - 150;
    colorOptionsCursorPosY = cursorY - 50;
    isColorOptions = !isColorOptions;
  };

  /***********************************************************************************************************************************/

  const addNote = async () => {
    if (currentTabId === null) return;

    const result = await createNote(currentTabId, ($lang === 'en' ? "Title" : "Otsikko"), ($lang === 'en' ? "No content" : "Ei sisältöä"));
    if (!result.success) sendAlert("alert.add-note.fail", true, false);
  };

  const addTab = async () => {
    const result = await createTab(($lang === 'en' ? "New tab" : "Uusi välilehti"));
    if (!result.success) sendAlert("alert.add-tab.fail", true, false);
  };

  const saveTabEdit = async () => {
    if (!editingTabId) return;
    if (editingTabTitle.trim() === '') {
      sendAlert("alert.tab.no-title", true, false);
      return;
    }

    const result = await updateTab(editingTabId, editingTabTitle);
    if (!result.success) sendAlert("alert.update-tab.fail", true, false);
    editingTabId = null;
  };

  const exitTabEdit = () => {
    editingTabId = null;
  };

  const handleTabDelete = async (tabId: number | null) => {
    if (!$tabs.some(t => t.id === tabId) || tabId === null) return;

    const result = await deleteTab(tabId);
    if (result.success) sendAlert("alert.delete-tab.success", true, false);
    else sendAlert("alert.delete-tab.fail", true, false);
    if (contextMenuTabId === currentTabId) currentTabId = null;
    isDeleteModalVisible = false;
    contextMenuTabId = null;
    isContextMenu = false;
  };

  const handleUpdateTabColor = async (color: string) => {
    if (!$tabs.some(t => t.id === currentTabId) || currentTabId === null) return;
    const result = await updateTabColor(currentTabId, color);
    if (!result.success) sendAlert("alert.tab-color-update.fail", true, false);
    isColorOptions = false;
  };
</script>

{#if isContextMenu}
  <ContextMenu {handleContextMenuDelete} cursorPosX={contextMenuCursorPosX} cursorPosY={contextMenuCursorPosY} {availableColors} {handleContextMenuTabColor} {handleTabEditStart} setContextMenuVisibility={(state) => isContextMenu = state} />
{/if}

{#if isColorOptions}
  <div class="horizontal-flex-container notes-color-menu" style="top: {colorOptionsCursorPosY}px; left: {colorOptionsCursorPosX}px;"
    use:handleClickOutside={{ getIgnoredElements, onOutsideClick: handleOutsideClick, additionalElements: [toggleColorsButton, toggleColorsEditorButton] }}
    transition:fade={{ duration: 200, easing: cubicInOut }}
  >
    {#if isColorForNotes}
      <div class="element-wrapper-for-title vertical-flex-container">
        <p class="element-paragraph-title">{$t["notes.for-text-color.option"]}</p>
        <ToggleSwitch
          activeDerivedFrom={isColorForText}
          onClickCommand={() => isColorForText = !isColorForText}
          translationKey={"notes.for-text-color.option"}
          height={25}
        />
      </div>
    {/if}
    <p style="width: 100%; margin-top: 0;">{$lang === 'en' ? "Dark" : "Tummat"}</p>
    {#each availableColors as color, i (i)}
      <button class="transparent-button" title={$lang === 'en' ? color.title[0] : color.title[1]} style="background-color: {color.value}; border-radius: 50%;"
        onclick={() => isColorForNotes ? changeNoteColor(color.value) : handleUpdateTabColor(color.value)}
      ></button>
      {#if i === 11}
        <p style="width: 100%;">{$lang === 'en' ? "Bright" : "Kirkkaat"}</p>
      {/if}
    {/each}
  </div>
{/if}

{#if zoomedNote}
  <div id="zoomed-note-container" class="vertical-flex-container" transition:fade={{ duration: 250, easing: cubicInOut }}>
    <p id="zoomed-note-saving" class:opacity-breathing={isNoteUpdating}>
      {isNoteUpdating ? $t["notes.zoomed-note.saving-in-progress"] : $t["notes.zoomed-note.has-saved"]}
    </p>
    <div id="zoomed-note-wrapper" transition:fly={{ y: windowInnerHeight, duration: 250, easing: cubicInOut }}>
      <NoteComponent note={zoomedNote} {cursorY} {cursorX} {fontSize} {noteColor} {toggleHeadingOptions} {zoomedNote} {isNoteUpdating} {noteBgColor}
        onFocusChange={(controls) => focusedNoteControls = controls}
        updateFontSize={(currentFontSize) => fontSize = currentFontSize}
        setZoomedNote={(noteId) => zoomedNoteId = noteId}
        updateOngoing={(state) => isNoteUpdating = state}
      />
    </div>
  </div>
{/if}

<div id="notes-main-container" class="vertical-flex-container">
  <div id="notes-main-toolbar" class="vertical-flex-container">
    <div class="primary-toolbar horizontal-flex-container">
      {#each toolBarMainButtons as button, i (button.titleKey)}
        <button class="primary-button horizontal-flex-container"
          class:disabled={currentTabId === null}
          disabled={currentTabId === null}
          style="gap: 8px;"
          onclick={() => currentTabId !== null ? button.command() : {}}
          bind:this={toolBarMainButtonRefs[i]}
        >
          <img src={button.icon} alt="Add" class="img-small" />
          {$t[button.titleKey]}
        </button>
      {/each}
      {#each toolBarSelectElements as element, idx (element.titleKey)}
        <div class="element-wrapper-for-title vertical-flex-container" title={idx === 2 ? $t["notes.note-bg-color"][1] as string : ""}>
          <p class="element-paragraph-title">{idx === 2 ? $t[element.titleKey][0] : $t[element.titleKey]}</p>
          <select class="primary-input" value={element.get()} onchange={(e) => element.set(Number((e.target as HTMLSelectElement)?.value))}>
            {#each element.options as item, i (i)}
              <option style="background-color: #0f0f0f;" value={i+1}>{idx === 2 ? $t[item][i] : item}</option>
            {/each}
          </select>
        </div>
      {/each}
    </div>
    <div class="primary-toolbar horizontal-flex-container" use:handleHorizontalScroll={{ scrollMultiplier: 0.4 }} class:note-zoomed={zoomedNote}>
      <button class="transparent-button-highlight" title={$t["exit-zoom.button"] as string} 
        class:disabled={!zoomedNote || isNoteUpdating}
        disabled={!zoomedNote || isNoteUpdating}
        onclick={() => zoomedNoteId = null}
      >
        <img src="/zoom-out.svg" alt="Zoom out" class="img-small" />
      </button>
      <div class="element-wrapper-for-title vertical-flex-container">
        <p class="element-paragraph-title">{$t["notes.font-size.select"]}</p>
        <select class="primary-input" class:disabled={!currentTabId} disabled={!currentTabId} bind:value={fontSize} onchange={() => focusedNoteControls?.applyProperty('set-fontsize')}>
          {#each [...Array(40).keys()].map(i => i + 9 + "px") as option (option)}
            <option style="background-color: #0f0f0f;" value={option}>{`${option}`}</option>
          {/each}
        </select>
      </div>
      <button class="transparent-button-highlight" title={$t["note-toolbar.button.titles"][$t["note-toolbar.button.titles"].length - 1] as string}
        class:disabled={!currentTabId}
        disabled={!currentTabId}
        bind:this={toggleColorsEditorButton}
        onclick={() => { handleColorMenu(); isColorForNotes = true; }}
      >
        <img src="/palette.svg" alt="Palette" class="img-small" />
      </button>
      {#each toolBarEditorButtons as button, i (button.name)}
        {@const disabled = (i === 0 || i === 4) && focusedNoteControls?.isTitleActive}
        <button class="transparent-button-highlight" title={$t["note-toolbar.button.titles"][i] as string} class:disabled={disabled || !currentTabId} disabled={disabled || !currentTabId}
          bind:this={toolBarEditorButtonRefs[i]} onclick={() => focusedNoteControls?.applyProperty(button.name)}
        >
          <img src={button.icon} alt={button.icon} class="img-small" />
        </button>
      {/each}
    </div>
  </div>

  {#if currentTabId === null}
    <p style="justify-self: center; font-weight: bold;">{$t["notes.no-current-tabid"]}</p>
  {:else}
    {#if displayNotes.length <= 0}
      <div class="vertical-flex-container">
        <p style="font-weight: bold;">{$t["notes.no-notes-yet"]}</p>
        <img src="/notes.svg" alt="Notes" style="width: 6rem; height: 8rem;" />
      </div>
    {:else}
      <div id="notes-container" style="grid-template-columns: repeat({noteColumns}, 1fr); grid-auto-rows: {noteGridRows}px;">
        {#each displayNotes as note (note.id)}
          <NoteComponent {note} {cursorY} {cursorX} {fontSize} {noteColor} {toggleHeadingOptions} {zoomedNote} {isNoteUpdating} {noteBgColor}
            onFocusChange={(controls) => focusedNoteControls = controls}
            updateFontSize={(currentFontSize) => fontSize = currentFontSize}
            setZoomedNote={(noteId) => zoomedNoteId = noteId}
            updateOngoing={(state) => isNoteUpdating = state}
          />
        {/each}
      </div>
    {/if}
  {/if}

  <div id="notes-tabbar" class="horizontal-flex-container">
    <button id="notes-tab-add-button" class="primary-button horizontal-flex-container" onclick={() => addTab()}><img src="/plus.svg" alt="Plus" class="img-small" />{$t["notes.add-tab.button"]}</button>
    <div id="notes-tabs-list" class="horizontal-flex-container" use:handleHorizontalScroll>
      {#each displayTabs as tab (tab.id)}
        <div class="notes-tab-outer-container">
          <button class="transparent-button-highlight" style="background-color: {tab.color}; color: {tab.color === availableColors[1].value ? 'black' : '#f6f6f6'}"
            onclick={() => currentTabId = tab.id}
            oncontextmenu={(e) => { e.preventDefault(); handleContextMenu(tab.id); }}
            ondblclick={() => handleTabEditStart()}
            onkeydown={(e) => { if (e.key === "Enter") saveTabEdit(); if (e.key === "Escape") exitTabEdit(); }}
            class:in-editmode={tab.id === editingTabId}
            class:disabled={isDeleteModalVisible}
            class:currentTab={tab.id === currentTabId}
            disabled={isDeleteModalVisible}
            title={tab.title}
          >
            {#if editingTabId === tab.id}
              <input class="transparent-input" type="text" bind:value={editingTabTitle} bind:this={editingTabInput} onblur={() => saveTabEdit()} onclick={(e) => e.stopPropagation()} />
              {#each [editingTabInput], i (i)}
                {onMount(() => editingTabInput?.focus())}
              {/each}
            {:else}
              <span class:slideText={tab.title.length >= 18} style="color: {(tab.color === availableColors[2].value || tab.color === availableColors[12].value) ? "black" : "#f6f6f6"}">{tab.title}</span>
            {/if}
          </button>
        </div>
      {/each}
    </div>
  </div>
</div>

<style>
  .currentTab::after {
    content: "";
    position: absolute;
    bottom: 0;
    left: 0;
    height: 3px;
    width: 100%;
    background-color: rgba(255, 70, 70, 1);
  }
  #notes-main-container {
    justify-content: space-between;
    height: 100%;
    width: 100%;
  }

  #notes-main-toolbar {
    justify-content: flex-start;
    width: 100%;
    min-height: 112px;
    height: 112px;
  }

  .primary-toolbar:nth-of-type(2) {
    position: fixed;
    top: 106px;
    left: 150px;
    width: calc(100% - 150px);
    align-items: flex-start;
    padding: 8px 8px 4px 8px;
    overflow-x: auto;
    scrollbar-gutter: stable;
    transition: top 250ms cubic-bezier(0.65, 0, 0.35, 1), left 250ms cubic-bezier(0.65, 0, 0.35, 1);
  }

  .primary-toolbar:nth-of-type(2) button {
    min-width: 31px;
    width: 31px;
    height: 31px;
    margin-top: 4px;
    border-radius: 6px;
  }
  .primary-toolbar:nth-of-type(2) select.disabled, .primary-toolbar:nth-of-type(2) button.disabled {
    background-color: transparent;
    cursor: not-allowed;
  }

  .primary-toolbar.note-zoomed {
    position: fixed;
    z-index: 100;
    top: 0;
    left: 0;
    width: 100%;
  }

  .primary-toolbar .element-wrapper-for-title {
    min-width: 34px;
    max-width: 64px;
  }
  .primary-toolbar:nth-of-type(2) .element-wrapper-for-title {
    min-width: 52px;
  }

  .element-wrapper-for-title select {
    max-height: 100%;
    padding: 0 2px;
    color: #f6f6f6;
    font-size: clamp(0.75rem, 0.9cqw, 0.8rem);
  }
  .element-wrapper-for-title select:hover {
    cursor: pointer;
  }

  #notes-container {
    display: grid;
    gap: 20px;
    padding: 20px 14px 20px 20px;
    width: 100%;
    height: 100%;
    overflow-y: auto;
    scrollbar-gutter: stable;
  }

  #notes-tabbar {
    justify-content: flex-start;
    width: 100%;
    min-height: 32px;
    height: 32px;
    padding: 0 4px 0 0;
    gap: 20px;
    border-top: 1px solid #333;
    overflow: hidden;
  }

  #notes-tabbar button {
    justify-content: flex-start;
    gap: 4px;
    padding: 6px 8px;
    transform: none;
    box-shadow: none;
    outline: none;
  }

  #notes-tabbar #notes-tab-add-button {
    height: 27px;
    min-width: 130px;
    width: 130px;
    gap: 8px;
    border-radius: 0 6px 6px 0;
  }

  #notes-tabs-list {
    height: 100%;
    justify-content: flex-start;
    align-items: flex-start;
    gap: 4px;
    padding-top: 4px;
    overflow-x: auto;
    overflow-y: hidden;
  }

  .notes-tab-outer-container {
    min-height: 23px;
    height: 23px;
    border-right: 1px solid #333;
    padding-right: 4px;
  }
  .notes-tab-outer-container:first-of-type {
    border-left: 1px solid #333;
    padding-left: 4px;
  }

  #notes-tabs-list button {
    position: relative;
    width: 6rem;
    height: 100%;
    padding: 0;
    border-radius: 4px;
    overflow: hidden;
  }
  #notes-tabs-list button > * {
    width: 100%;
    padding-left: 4px;
  }
  #notes-tabs-list button:not(.disabled):hover::before {
    position: absolute;
    content: "";
    inset: 0;
    z-index: -1;
    border-radius: 4px;
  }
  #notes-tabs-list button:not(.disabled):hover::before {
    background-color: #222 !important;
  }

  #notes-tabs-list button span {
    text-align: left;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  #notes-tabs-list button span.slideText:hover {
    text-overflow: unset;
    overflow: visible;
    animation: slideLeft 3s linear infinite;
  }

  .notes-color-menu {
    z-index: 1000;
    background-color: #181818;
    outline: 1px solid #333;
  }

  #zoomed-note-container {
    position: fixed;
    inset: 0;
    z-index: 100;
    background-color: rgba(15, 15, 15, 1);
  }

  #zoomed-note-wrapper {
    width: 100%;
    height: 100%;
    padding: 120px 25%;
  }

  #zoomed-note-saving {
    position: fixed;
    top: 48px;
    font-weight: bold;
    user-select: none;
  }
</style>