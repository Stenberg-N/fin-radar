<script lang="ts">
  import { onMount, onDestroy, untrack } from "svelte";
  import { fade, fly } from "svelte/transition";
  import { flip } from "svelte/animate";
  import { cubicInOut } from "svelte/easing";
  import { goto, beforeNavigate } from "$app/navigation";
  import type { Editor } from "@tiptap/core";
  import { SvelteSet } from "svelte/reactivity";

  import { i18n } from "$lib/i18n/i18n.svelte";
  import type { Note } from "$lib/types";
  import { createNote, createTab, getNotes, getTabs, notes, tabs, updateTab, deleteTab, updateTabColor, stopNoteBatchFlush, startNoteBatchFlush, isNoteUpdateBatchOngoing } from "$lib/notes";
  import { sendAlert } from "$lib/alert";
  import { handleClickOutside, handleHorizontalScroll, HoverTitle } from "$lib/actions.svelte";
  import { viewport } from "$lib/viewport";
  import { handlePointerDown, handlePointerMove, handlePointerUp } from "$lib/dragAndDrop";
  import { userPrefs, updateUserPrefs } from "$lib/prefsStore";

  import NoteComponent from "../../components/notes/Note.svelte";
  import ContextMenu from "../../components/notes/ContextMenu.svelte";
  import ToggleSwitch from "../../components/ToggleSwitch.svelte";
  import ModalWrapper from "../../components/ModalWrapper.svelte";
  import SearchBar from "../../components/SearchBar.svelte";

  type HoverContent = { content: string };

  // MAIN
  let searchRegex = $state<RegExp | null>(null);
  let frozenIds = $state<SvelteSet<number> | null>(null);
  const displayNotes = $derived(
    (frozenIds
      ? $notes.filter((n) => frozenIds!.has(n.id))
      : $notes.filter(n => n.tab_id === currentTabId)
    ).sort((a, b) => a.order_id - b.order_id)
  );
  const displayTabs = $derived($tabs.sort((a, b) => a.order_id - b.order_id));
  let focusedNoteControls = $state<{
    applyProperty: (command: string) => void;
    isTitleActive: boolean;
    focusedEditor: Editor;
  } | null>(null);
  let editorState = $state<{
    isTaskListActive: boolean,
    canAddNewItem: boolean,
    canIndent: boolean,
    canOutdent: boolean
    isUnderline: boolean,
    isBold: boolean,
    isItalic: boolean,
    isBulletList: boolean,
    fontSize: string,
    textAlignment: '' | 'right' | 'center' | 'left',
  }>({
    isTaskListActive: false,
    canAddNewItem: false,
    canIndent: false,
    canOutdent: false,
    isUnderline: false,
    isBold: false,
    isItalic: false,
    isBulletList: false,
    fontSize: '',
    textAlignment: '',
  });

  // WITHOUT CLASSIFICATION
  let isDeleteModalVisible = $state<boolean>(false);
  let isColorOptions = $state<boolean>(false);
  let pendingNavigation = $state<string | null>(null);
  let isColorForNotes = $state<boolean>(false);
  let isColorForText = $state<boolean>(false);
  let noteColor = $state<string | null>(null);
  let zoomedNoteId = $state<number | null>(null);
  const zoomedNote = $derived(displayNotes.find(n => n.id === zoomedNoteId));
  let noteDragIndex = $state<number | null>(null);
  const hover = new HoverTitle<HoverContent>();

  // STORE
  const noteColumns = $derived($userPrefs.notePrefs["noteColumns"]);
  const noteHeight = $derived($userPrefs.notePrefs["noteHeight"]);
  const noteBgColor = $derived($userPrefs.notePrefs["noteBgColor"]);
  const mainBgColor = $derived($userPrefs.notePrefs["mainBgColor"]);
  const mainContainerHeight = $derived($viewport.height - 254);
  const noteGridRows = $derived(noteHeight === "100%" ? mainContainerHeight - 16 : (mainContainerHeight - 40) / 2); 

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
  let tabDragIndex = $state<number | null>(null);

  // CONTEXT MENU
  let contextMenuTabId = $state<number | null>(null);
  let isContextMenu = $state<boolean>(false);

  // TOP TOOLBAR
  const toolBarMainButtons = [
    { titleKey: "add.button", icon: "/plus.svg", command: async () => await addNote() },
    { titleKey: "delete.button", icon: "/trash-can.svg", command: () => {
      isDeleteModalVisible = true;
      sendAlert({
        message: "alert.delete-tab.confirmation",
        isTimer: false,
        buttons: true,
        onConfirm: async () => {
          if (currentTabId !== null) {
            await handleTabDelete(currentTabId);
            currentTabId = null;
          } else {}
        },
        onCancel: () => isDeleteModalVisible = false,
        additionalText: $tabs.find(t => t.id === currentTabId)?.title
      });
    }},
    { titleKey: "notes.change-tab-color", icon: "/palette.svg", command: () => { handleColorMenu(); isColorForNotes = false; } },
  ];
  const toolBarSelectElements = [
    {
      get titleKey() { return i18n.t["notes.columns-amount"] as string; },
      options: ["1", "2", "3", "4", "5"],
      get: () => String(noteColumns),
      set: (value: string) => updateUserPrefs("notePrefs", "noteColumns", Number(value))
    },
    {
      get titleKey() { return i18n.t["notes.note-height"] as string; },
      options: ["100%", "50%"],
      get: () => noteHeight,
      set: (value: string) => updateUserPrefs("notePrefs", "noteHeight", value as "100%" | "50%")
    },
    {
      get titleKey() { return i18n.t["notes.note-bg-color"] as string[]; },
      options: ["dark", "light"],
      get: () => noteBgColor,
      set: (value: string) => updateUserPrefs("notePrefs", "noteBgColor", value as "dark" | "light")
    },
    {
      get titleKey() { return i18n.t["notes.main-bg-color"] as string[]; },
      options: ["dark", "light"],
      get: () => mainBgColor,
      set: (value: string) => updateUserPrefs("notePrefs", "mainBgColor", value as "dark" | "light")
    },
  ];
  const toolBarEditorButtons = [
    { name: "heading", icon: "/heading.svg" },
    { name: "underline", icon: "/underline.svg" },
    { name: "bold", icon: "/bold.svg" },
    { name: "italic", icon: "/italic.svg" },
    { name: "bullet-list", icon: "/bulleted-list.svg" },
    { name: "toggle-tasklist", icon: "/checklist.svg" },
    { name: "split-listitem", icon: "/plus.svg"},
    { name: "sink-listitem", icon: "/indent.svg"},
    { name: "lift-listitem", icon: "/outdent.svg"},
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
    { value: "#ddd", title: ["White", "Valkoinen"]},
    { value: "rgba(113, 45, 255, 1)", title: ["Purple", "Purppura"] },
    { value: "rgba(255, 70, 70, 1)", title: ["Red", "Punainen"] },
    { value: "rgba(255, 0, 255, 1)", title: ["Pink", "Pinkki"] },
    { value: "rgba(255, 150, 72, 1)", title: ["Orange", "Oranssi"] },
    { value: "rgba(255, 220, 0, 1)", title: ["Yellow", "Keltainen"] },
    { value: "rgba(94, 255, 94, 1)", title: ["Green", "Vihreä"] },
    { value: "rgba(170, 255, 170, 1)", title: ["Mint", "Minttu"] },
    { value: "rgba(215, 255, 0, 1)", title: ["Lime", "Lime"] },
    { value: "rgba(0, 255, 240, 1)", title: ["Turquoise", "Turkoosi"] },
    { value: "rgba(0, 140, 255, 1)", title: ["Blue", "Sininen"] },
  ];

  onMount(() => {
    (async () => {
      await getTabs();
      await getNotes();
      startNoteBatchFlush();
    })();
  });

  onDestroy(() => {
    (async () => await stopNoteBatchFlush())();
  });

  beforeNavigate(({ to, cancel }) => {
    if (!to || !$isNoteUpdateBatchOngoing) return;

    cancel();
    pendingNavigation = to.url.pathname;
    sendAlert({ message: "alert.unsaved-changes", isTimer: true, buttons: false });
  });

  $effect(() => {
    if (pendingNavigation !== null && !$isNoteUpdateBatchOngoing) {
      goto(pendingNavigation);
      pendingNavigation = null;
    }
  });

  $effect(() => {
    const editor = focusedNoteControls?.focusedEditor;
    if (!editor) return;

    const onUpdate = () => updateEditorState(editor);
    const onBlur = () => resetState();

    untrack(() => updateEditorState(editor));

    editor.on("transaction", onUpdate);
    editor.on("selectionUpdate", onUpdate);
    editor.on("blur", onBlur);

    return () => {
      editor.off("transaction", onUpdate);
      editor.off("selectionUpdate", onUpdate);
      editor.off("blur", onBlur);
    };
  });

  $effect(() => {
    const regex = searchRegex;
    const ids = regex ? new SvelteSet(untrack(() => $notes).filter((n) => matches(n, regex)).map((n) => n.id)) : null;

    frozenIds = ids && ids.size <= 0
      ? untrack(() => {
        sendAlert({ message: "alert.search.nothing-found", isTimer: true, buttons: false});
        return frozenIds = null;
      })
      : ids;
  });

  // Used to collect toolbar's button references and bind the button for showing heading options to toggleHeadingOptions and bind the button for color options to toggleColorsButton,
  // and pass those to handleClickOutside to be ignored, since Svelte's bind:this doesn't allow conditional expressions.
  $effect(() => {
    if (toolBarMainButtonRefs[2]) toggleColorsButton = toolBarMainButtonRefs[2];
  });

  $effect(() => {
    if (toolBarEditorButtonRefs[0]) toggleHeadingOptions = toolBarEditorButtonRefs[0];
  });

  $effect(() => {
    return () => { hover.destroy(); };
  });


  /***********************************************************************************************************************************\
  |
  | Context, Helper & Wrapper functions
  |
  \***********************************************************************************************************************************/
  const handleOutsideClick = () => { isColorOptions = false };
  const matches = (n: Note, regex: RegExp) => { return [n.content, n.title].some(val => regex.test(val)); };

  /***********************************************************************************************************************************/

  const addNote = async () => {
    if (currentTabId === null) return;

    const result = await createNote(currentTabId, (i18n.lang === 'en' ? "Title" : "Otsikko"), (i18n.lang === 'en' ? "No content" : "Ei sisältöä"));
    if (!result.success) sendAlert({ message: "alert.add-note.fail", isTimer: true, buttons: false});
  };

  const addTab = async () => {
    const result = await createTab((i18n.lang === 'en' ? "New tab" : "Uusi välilehti"));
    if (!result.success) sendAlert({ message: "alert.add-tab.fail", isTimer: true, buttons: false });
  };

  const saveTabEdit = async () => {
    if (!editingTabId) return;
    if (editingTabTitle.trim() === '') {
      sendAlert({ message: "alert.tab.no-title", isTimer: true, buttons: false });
      return;
    }

    const result = await updateTab(editingTabId, editingTabTitle);
    if (!result.success) sendAlert({ message: "alert.update-tab.fail", isTimer: true, buttons: false });
    editingTabId = null;
  };

  const exitTabEdit = () => {
    editingTabId = null;
  };

  const handleTabDelete = async (tabId: number | null) => {
    if (!$tabs.some(t => t.id === tabId) || tabId === null) return;

    const result = await deleteTab(tabId);
    if (result.success) sendAlert({ message: "alert.delete-tab.success", isTimer: true, buttons: false });
    else sendAlert({ message: "alert.delete-tab.fail", isTimer: true, buttons: false });
    if (contextMenuTabId === currentTabId) currentTabId = null;
    isDeleteModalVisible = false;
    contextMenuTabId = null;
    isContextMenu = false;
  };

  const handleUpdateTabColor = async (color: string) => {
    if (!$tabs.some(t => t.id === currentTabId) || currentTabId === null) return;
    const result = await updateTabColor(currentTabId, color);
    if (!result.success) sendAlert({ message: "alert.tab-color-update.fail", isTimer: true, buttons: false });
    isColorOptions = false;
  };

  const changeNoteColor = (color: string) => {
    noteColor = color;
    isColorForText
    ? focusedNoteControls?.applyProperty('fore-color')
    : focusedNoteControls?.applyProperty('bg-color');
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
  };

  const handleContextMenuDelete = () => {
    isDeleteModalVisible = true;
    isContextMenu = false;
    sendAlert({
      message: "alert.delete-tab.confirmation",
      isTimer: false,
      buttons: true,
      onConfirm: async () => {
        if (contextMenuTabId !== null) {
          await handleTabDelete(contextMenuTabId);
        } else {}
      },
      onCancel: () => {
        isDeleteModalVisible = false;
        contextMenuTabId = null;
      },
      additionalText: $tabs.find(t => t.id === contextMenuTabId)?.title
    });
  };

  const handleContextMenuTabColor = async (color: string) => {
    if (!$tabs.some(t => t.id === contextMenuTabId) || contextMenuTabId === null) return;
    const result = await updateTabColor(contextMenuTabId, color);
    if (!result.success) sendAlert({ message: "alert.tab-color-update.fail", isTimer: true, buttons: false });
  };

  const handleColorMenu = () => {
    isColorOptions = !isColorOptions;
  };

  const updateEditorState = (editor: Editor) => {
    const nodeType = editor.state.selection.$anchor.node().type.name;

    editorState.isTaskListActive = editor.isActive("taskList");
    editorState.canAddNewItem = ["taskItem", "listItem"].some((option) => editor.can().splitListItem(option));
    editorState.canIndent = ["taskItem", "listItem"].some((option) => editor.can().sinkListItem(option));
    editorState.canOutdent = ["taskItem", "listItem"].some((option) => editor.can().liftListItem(option));
    editorState.isUnderline = editor.isActive("underline");
    editorState.isBold = editor.isActive("bold");
    editorState.isItalic = editor.isActive("italic");
    editorState.isBulletList = editor.isActive("bulletList");
    editorState.fontSize = editor.getAttributes('textStyle').fontSize || '16px';
    editorState.textAlignment = editor.getAttributes(nodeType).textAlign || '';
  };

  const resetState = () => {
    editorState = {
      isTaskListActive: false,
      canAddNewItem: false,
      canIndent: false,
      canOutdent: false,
      isUnderline: false,
      isBold: false,
      isItalic: false,
      isBulletList: false,
      fontSize: editorState.fontSize,
      textAlignment: '',
    };
  };
</script>

{#if isContextMenu}
  {#key contextMenuTabId}
    <ModalWrapper>
      <ContextMenu {handleContextMenuDelete} {availableColors} {handleContextMenuTabColor} {handleTabEditStart} setContextMenuVisibility={(state) => { isContextMenu = state; }} />
    </ModalWrapper>
  {/key}
{/if}

{#if isColorOptions}
  <ModalWrapper options={{ transition: { type: "fade", duration: 200, easing: "cubic-in-out" } }}>
    <div class="flex row notes-color-menu"
      use:handleClickOutside={{ onOutsideClick: handleOutsideClick, getAdditionalElements: () => [toggleColorsButton, toggleColorsEditorButton] }}
    >
      {#if isColorForNotes}
        <div class="element-wrapper-for-title flex column">
          <p class="element-paragraph-title">{i18n.t["notes.for-text-color.option"]}</p>
          <ToggleSwitch
            activeDerivedFrom={isColorForText}
            onClickCommand={() => isColorForText = !isColorForText}
            translationKey={"notes.for-text-color.option"}
            height={25}
          />
        </div>
      {/if}
      <p style="width: 100%; margin-top: 0;">{i18n.lang === 'en' ? "Dark" : "Tummat"}</p>
      {#each availableColors as color, i (i)}
        <button class="button-primary transparent" style="background-color: {color.value}; border-radius: 50%;"
          aria-label={i18n.lang === 'en' ? color.title[0] : color.title[1]}
          onclick={() => isColorForNotes ? changeNoteColor(color.value) : handleUpdateTabColor(color.value)}
          onmouseenter={() => hover.enter({ content: i18n.lang === 'en' ? color.title[0] : color.title[1] })}
          onmouseleave={(e) => hover.leave(e)}
        ></button>
        {#if i === 11}
          <p style="width: 100%;">{i18n.lang === 'en' ? "Bright" : "Kirkkaat"}</p>
        {/if}
      {/each}
    </div>
  </ModalWrapper>
{/if}

{#if zoomedNote}
  <div id="zoomed-note-container" class="flex column" transition:fade={{ duration: 250, easing: cubicInOut }}>
    <p id="zoomed-note-saving" class:opacity-breathing={$isNoteUpdateBatchOngoing} style="color: {mainBgColor === "light" ? 'black' : 'var(--color-white-primary1)'};">
      {$isNoteUpdateBatchOngoing ? i18n.t["saving.saving-in-progress"] : i18n.t["notes.zoomed-note.has-saved"]}
    </p>
    <div id="zoomed-note-wrapper" style="background-color: {mainBgColor === "light" ? 'var(--color-primary3)' : 'var(--color-primary2)'};" transition:fly={{ y: $viewport.height, duration: 250, easing: cubicInOut }}>
      <div role="note" class="note-container flex column" style="background-color: {noteBgColor === "light" ? 'var(--color-primary3)' : 'var(--color-secondary1)'}; color: {noteBgColor === "light" ? 'black' : 'var(--color-white-primary1)'};">
        <NoteComponent note={zoomedNote} fontSize={editorState.fontSize} {noteColor} {toggleHeadingOptions} {zoomedNote} isNoteUpdating={$isNoteUpdateBatchOngoing} {noteBgColor}
          onFocusChange={(controls) => { focusedNoteControls = controls; }}
          setZoomedNote={(noteId) => { zoomedNoteId = noteId; }}
          setDeleteModalVisibility={(state) => { isDeleteModalVisible = state; }}
        />
      </div>
    </div>
  </div>
{/if}

{#if hover.isHovering && hover.target?.content.trim() !== ''}
  <ModalWrapper options={{
    position: { centerElement: true, moveTop: -40 },
    transition: { type: "fade", duration: 200, easing: "cubic-in-out" },
    borderRadius: 8,
    outline: { width: 1, color: 'var(--outline-color1)'},
    }}
  >
    <p id="notes-page-hover-title-content">
      {hover.target?.content}
    </p>
  </ModalWrapper>
{/if}

<div id="notes-main-container" class="flex column">
  <div id="notes-main-toolbar" class="flex column">
    <div class="primary-toolbar flex row" use:handleHorizontalScroll={{ scrollMultiplier: 0.4 }}>
      <SearchBar options={{ sendRegexToParent: (regex) => { searchRegex = regex }, disabled: !currentTabId, searchModeIndicator: true }} />
      <div style="border-left: 1px solid var(--outline-color1); height: 100%; min-width: 0; padding-right: 2px;"></div>
      {#each toolBarMainButtons as button, i (button.titleKey)}
        <button class="button-primary transparent highlight outline default-corners"
          disabled={currentTabId === null}
          style="gap: 8px;"
          onclick={() => currentTabId !== null ? button.command() : {}}
          bind:this={toolBarMainButtonRefs[i]}
        >
          <span class="span-icon img-small" style="mask-image: url('{button.icon}');"></span>
          {i18n.t[button.titleKey]}
        </button>
      {/each}
      <div style="border-left: 1px solid var(--outline-color1); height: 100%; min-width: 0; padding-right: 2px;"></div>
      {#each toolBarSelectElements as element, idx (element.titleKey)}
        <div class="element-wrapper-for-title flex column">
          <p class="element-paragraph-title"
            onmouseenter={() => hover.enter({ content: idx === 2 ? (i18n.t["notes.note-bg-color"] as string[])[1] : idx === 3 ? (i18n.t["notes.main-bg-color"] as string[])[1] : "" })}
            onmouseleave={(e) => hover.leave(e)}
          >
            {[2, 3].includes(idx) ? element.titleKey[0] : element.titleKey}
          </p>
          <select class="primary-input" value={element.get()} onchange={(e) => element.set((e.target as HTMLSelectElement)?.value)}>
            {#each element.options as item, i (i)}
              <option style="background-color: var(--color-primary1);" value={item}>
                {[2, 3].includes(idx) ? (i18n.t["notes.bg-color-options"] as string[])[i] : item}
              </option>
            {/each}
          </select>
        </div>
      {/each}
    </div>
    <div class="primary-toolbar flex row" use:handleHorizontalScroll={{ scrollMultiplier: 0.4 }} class:note-zoomed={zoomedNote}
      style="left: {!zoomedNote ? `${$userPrefs.mainPrefs.navBarWidth + 16}px` : "0"};"
    >
      <button class="button-primary transparent highlight"
        aria-label={i18n.t["exit-zoom.button"] as string}
        disabled={!zoomedNote || $isNoteUpdateBatchOngoing}
        onclick={() => zoomedNoteId = null}
        onmouseenter={() => hover.enter({ content: i18n.t["exit-zoom.button"] as string })}
        onmouseleave={(e) => hover.leave(e)}
      >
        <span class="span-icon img-small" style="mask-image: url('/zoom-out.svg');"></span>
      </button>
      <div class="element-wrapper-for-title flex column">
        <p class="element-paragraph-title">
          {i18n.t["notes.font-size.select"]}
        </p>
        <select class="primary-input" disabled={!currentTabId} bind:value={editorState.fontSize} onchange={() => focusedNoteControls?.applyProperty('set-fontsize')}>
          {#each [...Array(40).keys()].map(i => i + 9 + "px") as option (option)}
            <option style="background-color: var(--color-primary1);" value={option}>
              {option}
            </option>
          {/each}
        </select>
      </div>
      <div style="border-right: 1px solid var(--outline-color1); height: 40px; min-width: 0; padding-left: 2px;"></div>
      <button class="button-primary transparent highlight"
        aria-label={(i18n.t["note-toolbar.button.titles"] as string[])[(i18n.t["note-toolbar.button.titles"] as string[]).length - 1]}
        disabled={!currentTabId}
        bind:this={toggleColorsEditorButton}
        onclick={() => { handleColorMenu(); isColorForNotes = true; }}
        onmouseenter={() => hover.enter({ content: (i18n.t["note-toolbar.button.titles"] as string[])[(i18n.t["note-toolbar.button.titles"] as string[]).length - 1] })}
        onmouseleave={(e) => hover.leave(e)}
      >
        <span class="span-icon img-small" style="mask-image: url('/palette.svg');"></span>
      </button>
      {#each toolBarEditorButtons as button, i (button.name)}
        {@const disabledForTitle = [0, 4, 5, 6, 7, 8, 9, 10, 11].includes(i) && focusedNoteControls?.isTitleActive}
        <button class="button-primary transparent highlight"
          aria-label={(i18n.t["note-toolbar.button.titles"] as string[])[i]}
          disabled={
            disabledForTitle ||
            !currentTabId ||
            (i === 6 && !editorState.canAddNewItem) ||
            (i === 7 && !editorState.canIndent) ||
            (i === 8 && !editorState.canOutdent)
          }
          class:toolbar-button-active={
            i === 1 && editorState.isUnderline ||
            i === 2 && editorState.isBold ||
            i === 3 && editorState.isItalic ||
            i === 4 && editorState.isBulletList ||
            i === 5 && editorState.isTaskListActive ||
            i === 9 && editorState.textAlignment === 'left' ||
            i === 10 && editorState.textAlignment === 'center' ||
            i === 11 && editorState.textAlignment === 'right'
          }
          bind:this={toolBarEditorButtonRefs[i]} onclick={() => focusedNoteControls?.applyProperty(button.name)}
          onmousedown={(e) => e.preventDefault()}
          onmouseenter={() => hover.enter({ content: (i18n.t["note-toolbar.button.titles"] as string[])[i] })}
          onmouseleave={(e) => hover.leave(e)}
        >
          <span class="span-icon img-small" style="mask-image: url('{button.icon}');"></span>
        </button>
      {/each}
    </div>
  </div>

  {#if currentTabId === null}
    <div class="flex column" style="width: 100%; height: 100%; background-color: {mainBgColor === "light" ? 'var(--color-primary3)' : 'var(--color-primary2)'};">
      <p style="color: {mainBgColor === "light" ? 'black' : 'var(--color-white-primary1)'}; font-weight: bold; user-select: none;">{i18n.t["notes.no-current-tabid"]}</p>
    </div>
  {:else}
    {#if displayNotes.length <= 0}
      <div class="flex column" style="width: 100%; height: 100%; background-color: {mainBgColor === "light" ? 'var(--color-primary3)' : 'var(--color-primary2)'};">
        <p style="font-weight: bold; color: {mainBgColor === "light" ? 'black' : 'var(--color-white-primary1)'};">{i18n.t["notes.no-notes-yet"]}</p>
        <span class="span-icon" style="mask-image: url('/notes.svg'); width: 6rem; height: 8rem; user-select: none; background-color: {mainBgColor === "light" ? 'black' : 'var(--color-white-primary1)'};"></span>
      </div>
    {:else}
      <div id="notes-container" style="grid-template-columns: repeat({noteColumns}, 1fr); grid-auto-rows: {noteGridRows}px; background-color: {mainBgColor === "light" ? 'var(--color-primary3)' : 'var(--color-primary2)'};">
        {#each displayNotes as note, i (note.id)}
          <div role="note" class="note-container flex column"
            animate:flip={{ duration: 200, easing: cubicInOut }}
            style="background-color: {noteBgColor === "light" ? 'var(--color-primary3)' : 'var(--color-secondary1)'}; color: {noteBgColor === "light" ? 'black' : 'var(--color-white-primary1)'};"
            onpointerup={() => { const res = handlePointerUp({ array: notes, arrayType: "notes", idx: i, dragIndex: noteDragIndex, currentTabId }); if (res) noteDragIndex = res.dragIndex; }}
            data-index={i}
            class:hovered-over={noteDragIndex === i}
          >
            <button aria-label="Drag handle" class="drag-handle flex row"
              disabled={isDeleteModalVisible}
              onpointermove={(e) => { const res = handlePointerMove(e, noteDragIndex, "notes"); if (res) noteDragIndex = res.dragIndex; }}
              onpointerdown={(e) => { if (!isDeleteModalVisible) { const res = handlePointerDown(e, i); if (res) noteDragIndex = res.dragIndex; }}}
            >
              <span class="span-icon img-small" style="mask-image: url('/grip-dots.svg'); mask-position: center; background-color: {noteBgColor === "light" ? 'black' : 'var(--color-white-primary1)'};"></span>
            </button>
            <NoteComponent {note} fontSize={editorState.fontSize} {noteColor} {toggleHeadingOptions} {zoomedNote} isNoteUpdating={$isNoteUpdateBatchOngoing} {noteBgColor}
              onFocusChange={(controls) => focusedNoteControls = controls}
              setZoomedNote={(noteId) => zoomedNoteId = noteId}
              setDeleteModalVisibility={(state) => isDeleteModalVisible = state}
            />
          </div>
        {/each}
      </div>
    {/if}
  {/if}

  <div id="notes-tabbar" class="flex row">
    <button id="notes-tab-add-button" class="button-primary transparent highlight" onclick={() => addTab()}>
      <span class="span-icon img-small" style="mask-image: url('/plus.svg');"></span>
      {i18n.t["notes.add-tab.button"]}
    </button>
    <div id="notes-tabs-list" class="flex row" use:handleHorizontalScroll>
      {#each displayTabs as tab, i (tab.id)}
        <div class="notes-tab-outer-container" role="tab" tabindex="0" animate:flip={{ duration: 200, easing: cubicInOut }}
          onpointerup={() => { const res = handlePointerUp({ array: tabs, arrayType: "tabs", idx: i, dragIndex: tabDragIndex }); if (res) tabDragIndex = res.dragIndex; }}
          data-index={i}
        >
          <button aria-label="Drag handle" class="drag-handle flex row"
            disabled={isDeleteModalVisible}
            onpointermove={(e) => { const res = handlePointerMove(e, tabDragIndex, "tabs"); if (res) tabDragIndex = res.dragIndex; }}
            onpointerdown={(e) => { if (!isDeleteModalVisible) { const res = handlePointerDown(e, i); if (res) tabDragIndex = res.dragIndex; }}}
          >
            <span class="span-icon img-small" style="mask-image: url('/grip-dots.svg'); mask-position: center;"></span>
          </button>
          <button class="button-primary transparent highlight" style="background-color: {tab.color}; color: {tab.color === availableColors[1].value ? 'black' : 'var(--color-white-primary1)'}"
            onclick={() => currentTabId = tab.id}
            oncontextmenu={(e) => { e.preventDefault(); handleContextMenu(tab.id); }}
            ondblclick={() => handleTabEditStart()}
            onkeydown={(e) => { if (e.key === "Enter") saveTabEdit(); if (e.key === "Escape") exitTabEdit(); }}
            class:in-editmode={tab.id === editingTabId}
            class:currentTab={tab.id === currentTabId}
            class:hovered-over={tabDragIndex === i}
            disabled={isDeleteModalVisible}
            onmouseenter={() => hover.enter({ content: tab.title })}
            onmouseleave={(e) => hover.leave(e)}
          >
            {#if editingTabId === tab.id}
              <input class="transparent-input" type="text" bind:value={editingTabTitle} bind:this={editingTabInput} onblur={() => saveTabEdit()} onclick={(e) => e.stopPropagation()} />
              {#each [editingTabInput], i (i)}
                {onMount(() => editingTabInput?.focus())}
              {/each}
            {:else}
              <span class:slideText={tab.title.length >= 18} style="user-select: none; color: {(tab.color === availableColors[2].value || tab.color === availableColors[12].value) ? "black" : "var(--color-white-primary1)"}">{tab.title}</span>
            {/if}
          </button>
        </div>
      {/each}
    </div>
  </div>
</div>

<style>
  .toolbar-button-active {
    background-color: var(--color-highlight2);
  }

  .currentTab {
    outline: 1px solid var(--color-highlight1);
  }

  #notes-page-hover-title-content {
    margin: 0;
    padding: 0.25rem 0.5rem;
    background-color: var(--color-secondary1);
  }

  #notes-main-container {
    justify-content: space-between;
    height: 100%;
    width: 100%;
  }

  #notes-main-toolbar {
    justify-content: flex-start;
    width: 100%;
    min-height: 7rem;
    height: 7rem;

    .primary-toolbar {
      overflow-x: auto;
      overflow-y: hidden;

      &:nth-of-type(2) {
        position: fixed;
        width: unset;
        top: 114px;
        right: 0.5rem;
        align-items: flex-start;
        padding: 0.5rem 0.5rem 4px 0.5rem;
        overflow-x: auto;
        overflow-y: hidden;

        button {
          margin-top: 0.25rem;
          border-radius: 0.25rem;
        }

        .element-wrapper-for-title{
          height: 39px;
        }
      }

      &.note-zoomed {
        position: fixed;
        z-index: 100;
        top: 0;
      }

      .element-wrapper-for-title {
        min-width: fit-content;
        text-wrap: nowrap;
      }
    }
  }

  .element-wrapper-for-title select {
    padding: 0 2px;
    font-size: clamp(0.75rem, 0.9cqw, 0.8rem);
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

  .note-container {
    position: relative;
    justify-content: flex-start;
    height: 100%;
    width: 100%;
    min-width: 240px;
    gap: 6px;
    padding: 0.5rem 0.5rem 1.5rem;
    border-radius: 4px;
    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.8);
    overflow: hidden;
  }

  #notes-tabbar {
    justify-content: flex-start;
    width: 100%;
    min-height: 2rem;
    height: 2rem;
    padding: 0 0.25rem 0 0;
    border-top: 1px solid var(--outline-color1);
    overflow: hidden;

    button:not(.drag-handle) {
      justify-content: flex-start;
      gap: 0.25rem;
      padding: 6px 0.5rem;
      transform: none;
      box-shadow: none;
    }

    #notes-tab-add-button {
      gap: 0.5rem;
      border-radius: 0;
      margin-right: 0.25rem;
    }
  }

  #notes-tabs-list {
    height: 100%;
    justify-content: flex-start;
    align-items: flex-start;
    gap: 0.25rem;
    padding-top: 0.25rem;
    padding-bottom: 1px;
    overflow-x: auto;
    overflow-y: hidden;

    button.button-primary.transparent.highlight {
      position: relative;
      width: 6rem;
      height: 100%;
      padding: 0;
      border-radius: 0.25rem;
      overflow: hidden;

      &:not(:disabled):hover::before {
        position: absolute;
        content: "";
        inset: 0;
        z-index: 0;
        border-radius: 0.25rem;
        background-color: rgba(255, 255, 255, 0.1) !important;
      }

      &.currentTab:not(:disabled):hover {
        outline-color: var(--color-highlight1);
      }

      > * {
        width: 100%;
        outline: none;
        padding-left: 0.25rem;
      }

      span {
        text-align: left;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;

        &.slideText:hover {
          text-overflow: unset;
          overflow: visible;
          animation: slideLeft 3s linear infinite;
        }
      }
    }
  }

  .notes-tab-outer-container {
    position: relative;
    flex-shrink: 0;
    height: 23px;
    border-right: 1px solid var(--outline-color1);
    padding-right: 28px;
  }
  .notes-tab-outer-container:first-of-type {
    border-left: 1px solid var(--outline-color1);
    padding-left: 0.25rem;
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
    top: 3.5rem;
    font-weight: bold;
    user-select: none;
  }
</style>