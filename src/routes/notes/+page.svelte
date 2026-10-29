<script lang="ts">
  import { onMount, onDestroy, untrack } from "svelte";
  import { fade, fly, slide } from "svelte/transition";
  import { flip } from "svelte/animate";
  import { cubicInOut } from "svelte/easing";
  import { goto, beforeNavigate } from "$app/navigation";
  import type { Editor } from "@tiptap/core";

  import { i18n } from "$lib/i18n/i18n.svelte";
  import type { Note } from "$lib/types";
  import { createNote, createTab, getNotes, getTabs, notes, tabs, updateTab, deleteTab, updateTabColor, stopNoteBatchFlush, startNoteBatchFlush, isNoteUpdateBatchOngoing, queueNoteUpdate } from "$lib/notes";
  import { sendAlert } from "$lib/classes/alertStore.svelte";
  import { handleHorizontalScroll } from "$lib/utils/actions.svelte";
  import { HoverTitle } from "$lib/classes/hoverTitle.svelte";
  import { viewport } from "$lib/utils/viewport";
  import { handlePointerDown, handlePointerMove, handlePointerUp } from "$lib/utils/dragAndDrop";
  import { userPrefs, updateUserPrefs } from "$lib/utils/prefsStore";

  import NoteComponent from "../../components/notes/Note.svelte";
  import ContextMenu from "../../components/notes/ContextMenu.svelte";
  import ModalWrapper from "../../components/ModalWrapper.svelte";
  import SearchBar from "../../components/elements/SearchBar.svelte";
  import ItemSelect from "../../components/elements/ItemSelect.svelte";
  import ColorList from "../../components/notes/ColorList.svelte";

  type HoverContent = { content: string };

  // MAIN
  let searchRegex = $state<RegExp | null>(null);
  let frozenIds = $state<Set<number> | null>(null);
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
    id: number;
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
  let lastFocusedNoteId = $state<number | null>(null);
  const hover = new HoverTitle<HoverContent>();

  // STORE
  const {
    noteColumns,
    noteHeight,
    mainBgColor
  } = $derived($userPrefs.notePrefs);
  let mainContainerHeight = $state<number>(0);
  const noteGridRows = $derived(noteHeight === "100%" ? mainContainerHeight - 40 : (mainContainerHeight - 60) / 2); 

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
  let isToolbarVisible = $derived($userPrefs.notePrefs.toolbarVisibility);
  const toolBarMainButtons = [
    {
      get title() { return i18n.t["add.button"] as string; },
      get hover() { return i18n.t["notes.add-note.title"] as string; },
      icon: "/plus.svg",
      command: async () => await addNote(),
      toggled: null,
    },
    {
      get title() { return i18n.t["delete.button"] as string; },
      get hover() { return i18n.t["notes.delete-note.title"] as string; },
      icon: "/trash-can.svg",
      command: () => {
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
      },
      toggled: null,
    },
    {
      get title() { return i18n.t["color.button"] as string; },
      get hover() { return i18n.t["notes.note-bg-color"] as string; },
      icon: "/palette.svg",
      command: () => { handleColorMenu(); isColorForNotes = false; },
      get toggled() { return (isColorOptions && !isColorForNotes); },
    },
  ];
  const toolbarSelectElements = [
    {
      get title() { return i18n.t["notes.columns-amount"] as string; },
      options: ["1", "2", "3", "4", "5"],
      get: () => String(noteColumns),
      set: (value: string) => updateUserPrefs("notePrefs", "noteColumns", Number(value))
    },
    {
      get title() { return i18n.t["notes.note-height"] as string; },
      options: ["100%", "50%"],
      get: () => noteHeight,
      set: (value: string) => updateUserPrefs("notePrefs", "noteHeight", value as "100%" | "50%")
    },
    {
      get title() { return i18n.t["notes.main-bg-color"] as string; },
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
    { value: "#c8c8c8", title: ["White", "Valkoinen"]},
    { value: "rgba(113, 45, 255, 0.25)", title: ["Dark purple", "Tumma purppura"] },
    { value: "rgba(255, 70, 70, 0.25)", title: ["Dark red", "Tumma punainen"] },
    { value: "rgba(255, 0, 255, 0.25)", title: ["Dark pink", "Tumma pinkki"] },
    { value: "rgba(255, 150, 72, 0.25)", title: ["Dark orange", "Tumma oranssi"] },
    { value: "rgba(255, 220, 0, 0.25)", title: ["Dark yellow", "Tumma keltainen"] },
    { value: "rgba(94, 255, 94, 0.25)", title: ["Dark green", "Tumma vihreä"] },
    { value: "rgba(215, 255, 0, 0.25)", title: ["Dark lime", "Tumma lime"] },
    { value: "rgba(0, 255, 240, 0.25)", title: ["Dark turquoise", "Tumma turkoosi"] },
    { value: "rgba(0, 140, 255, 0.25)", title: ["Dark blue", "Tumma sininen"] },

    // BRIGHTER
    { value: "#ddd", title: ["Bright White", "Kirkas valkoinen"]},
    { value: "rgba(113, 45, 255, 1)", title: ["Bright purple", "Kirkas purppura"] },
    { value: "rgba(255, 70, 70, 1)", title: ["Bright red", "Kirkas punainen"] },
    { value: "rgba(255, 0, 255, 1)", title: ["Bright pink", "Kirkas pinkki"] },
    { value: "rgba(255, 150, 72, 1)", title: ["Bright orange", "Kirkas oranssi"] },
    { value: "rgba(255, 220, 0, 1)", title: ["Bright yellow", "Kirkas keltainen"] },
    { value: "rgba(94, 255, 94, 1)", title: ["Bright green", "Kirkas vihreä"] },
    { value: "rgba(170, 255, 170, 1)", title: ["Bright mint", "Kirkas minttu"] },
    { value: "rgba(215, 255, 0, 1)", title: ["Bright lime", "Kirkas lime"] },
    { value: "rgba(0, 255, 240, 1)", title: ["Bright turquoise", "Kirkas turkoosi"] },
    { value: "rgba(0, 140, 255, 1)", title: ["Bright blue", "Kirkas sininen"] },
  ];
  const blackTextColors = $derived(Array.from([availableColors[2], availableColors[12], ...availableColors.slice(16, -1)]));

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
    const ids = regex ? new Set(untrack(() => $notes).filter((n) => matches(n, regex)).map((n) => n.id)) : null;

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
  const matches = (n: Note, regex: RegExp) => { return [n.content, n.title].some(val => regex.test(val)); };

  /***********************************************************************************************************************************/

  const addNote = async () => {
    if (currentTabId === null) return;
    await createNote(currentTabId);
  };

  const addTab = async () => {
    await createTab();
  };

  const saveTabEdit = async () => {
    await updateTab(editingTabId, editingTabTitle);
    editingTabId = null;
  };

  const exitTabEdit = () => {
    editingTabId = null;
  };

  const handleTabDelete = async (tabId: number | null) => {
    await deleteTab(tabId);
    if (contextMenuTabId === currentTabId) currentTabId = null;
    isDeleteModalVisible = isContextMenu = false;
    contextMenuTabId = null;
  };

  const changeNoteBgColor = async (color: string) => {
    if (!lastFocusedNoteId) return;

    isNoteUpdateBatchOngoing.update(() => true);
    const id = lastFocusedNoteId;
    queueNoteUpdate({ id, bg_color: color });
  };

  const changeNoteTextColor = (color: string) => {
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
      onConfirm: async () => { await handleTabDelete(contextMenuTabId); },
      onCancel: () => {
        isDeleteModalVisible = false;
        contextMenuTabId = null;
      },
      additionalText: $tabs.find(t => t.id === contextMenuTabId)?.title
    });
  };

  const handleContextMenuTabColor = async (color: string) => {
    await updateTabColor(contextMenuTabId, color);
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

  const toggleToolbar = () => {
    isToolbarVisible = !isToolbarVisible;
    updateUserPrefs("notePrefs", "toolbarVisibility", isToolbarVisible);
  };
</script>

{#if isContextMenu}
  {#key contextMenuTabId}
    <ContextMenu
      {availableColors}
      {handleContextMenuDelete}
      {handleContextMenuTabColor}
      {handleTabEditStart}
      setContextMenuVisibility={(state) => { isContextMenu = state; }}
    />
  {/key}
{/if}

{#if isColorOptions}
  <ColorList
    {availableColors}
    onOutsideClick={(state) => { isColorOptions = state; }}
    selectColor={(color) => { isColorForNotes ? changeNoteTextColor(color) : changeNoteBgColor(color); }}
    {isColorForNotes}
    bind:isColorForText={isColorForText}
    ignorableEls={[toggleColorsButton, toggleColorsEditorButton]}
  />
{/if}

{#if zoomedNote}
  <div role="none" id="zoomed-note-container" class="flex column" transition:fade={{ duration: 200, easing: cubicInOut }}
    onkeydown={(e) => { if (e.key === "Escape") zoomedNoteId = null}}
  >
    <button
      aria-label="Toggle toolbar"
      class="toolbar-toggle button-primary dark"
      class:hidden={!isToolbarVisible}
      onclick={toggleToolbar}
    >
      <span class="span-icon img-extra-small" style="mask-image: url('/arrow.svg'); transform: rotate({isToolbarVisible ? "180" : ""}deg);"></span>
    </button>
    <p id="zoomed-note-saving" class:opacity-breathing={$isNoteUpdateBatchOngoing} style="color: {mainBgColor === "light" ? 'black' : 'var(--color-white-primary1)'};">
      {$isNoteUpdateBatchOngoing ? i18n.t["saving.saving-in-progress"] : i18n.t["notes.zoomed-note.has-saved"]}
    </p>
    <div
      id="zoomed-note-wrapper"
      style="background-color: {mainBgColor === "light" ? 'var(--color-primary3)' : 'var(--color-primary2)'};"
      transition:fly={{ y: $viewport.height, duration: 250, easing: cubicInOut }}
    >
      <div role="note" class="note-container flex column" style="background-color: {zoomedNote.bg_color}">
        <NoteComponent note={zoomedNote} fontSize={editorState.fontSize} {noteColor} {toggleHeadingOptions} {zoomedNote} isNoteUpdating={$isNoteUpdateBatchOngoing}
          onFocusChange={(controls) => { focusedNoteControls = controls; }}
          setZoomedNote={(noteId) => { zoomedNoteId = noteId; }}
          setDeleteModalVisibility={(state) => { isDeleteModalVisible = state; }}
        />
      </div>
    </div>
  </div>
{/if}

{#if hover.isHovering && hover.target?.content.trim() !== ''}
  <ModalWrapper
    attributes={{ "hover-title-owner": hover.id }}
    position={{ centerElement: true, ...(zoomedNote ? {moveTop: 24} : {moveTop: -40})}}
    transition={{ type: "fade", duration: 200, easing: "cubic-in-out" }}
    borderRadius={8}
    outline={{ width: 1, color: 'var(--outline-color2)'}}
  >
    <p id="notes-page-hover-title-content">
      {hover.target?.content}
    </p>
  </ModalWrapper>
{/if}

<div id="notes-main-container" class="flex column">
  {#if isToolbarVisible}
    <div
      id="notes-main-toolbar"
      class="flex column"
      class:hidden={!isToolbarVisible}
      transition:slide={{ axis: "y", duration: 200, easing: cubicInOut }}
    >
      <div class="primary-toolbar flex row" use:handleHorizontalScroll={{ scrollMultiplier: 0.4 }}>
        <SearchBar
          sendRegexToParent={(regex) => { searchRegex = regex }}
          disabled={!currentTabId}
          searchModeIndicator={true}
        />
        <div style="border-left: 1px solid var(--outline-color2); height: 100%; min-width: 0; padding-right: 2px;"></div>
        {#each toolBarMainButtons as button, i (i)}
          <button
            class="button-primary transparent highlight outline default-corners"
            class:toggled={button.toggled}
            disabled={currentTabId === null || i === 2 && !lastFocusedNoteId}
            style="gap: 8px;"
            onclick={() => currentTabId !== null ? button.command() : {}}
            bind:this={toolBarMainButtonRefs[i]}
            onmouseenter={() => hover.enter({ content: button.hover })}
            onmouseleave={(e) => hover.leave(e)}
          >
            <span class="span-icon img-small" style="mask-image: url('{button.icon}');"></span>
            {button.title}
          </button>
        {/each}
        <div style="border-left: 1px solid var(--outline-color2); height: 100%; min-width: 0; padding-right: 2px;"></div>
        {#each toolbarSelectElements as el, i (i)}
          <ItemSelect
            data={el.options}
            onItemSelect={(item) => { if (el.get() !== item) el.set(item); }}
            overrideDefaultStartValue={el.get()}
            overrideDataValues={[2, 3].includes(i) ? (i18n.t["notes.bg-color-options"] as string[]) : undefined}
            onMouseLeave={(e) => hover.leave(e)}
            onMouseEnter={() => hover.enter({ content: el.title })}
          />
        {/each}
      </div>
      <div class="primary-toolbar flex row" use:handleHorizontalScroll={{ scrollMultiplier: 0.4 }} class:note-zoomed={zoomedNote}
        style="left: {!zoomedNote ? `${$userPrefs.mainPrefs.gutterAt + 16}px` : "0"};"
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
        <ItemSelect
          data={[...Array(40).keys()].map(i => i + 9 + "px")}
          onItemSelect={(item) => { editorState.fontSize = item; focusedNoteControls?.applyProperty('set-fontsize'); }}
          overrideWithCustomValue={!editorState.fontSize ? i18n.t["notes.font-size.select"] as string : editorState.fontSize}
          position={{ isPositionAbsolute: false, usePortal: true, }}
          disabled={!currentTabId}
        />
        <div style="border-right: 1px solid var(--outline-color2); height: 40px; min-width: 0; padding-left: 2px;"></div>
        <button
          class="button-primary transparent highlight"
          class:toggled={isColorOptions && isColorForNotes}
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
            class:toggled={
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
  {/if}
  <button
    aria-label="Toggle toolbar"
    class="toolbar-toggle button-primary dark"
    class:hidden={!isToolbarVisible}
    onclick={toggleToolbar}
  >
    <span class="span-icon img-extra-small" style="mask-image: url('/arrow.svg'); transform: rotate({isToolbarVisible ? "180" : ""}deg);"></span>
  </button>

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
      <div
        bind:clientHeight={mainContainerHeight}
        id="notes-container"
        style="
          grid-template-columns: repeat({noteColumns}, 1fr);
          grid-auto-rows: {noteGridRows}px;
          background-color: {mainBgColor === "light" ? 'var(--color-primary3)' : 'var(--color-primary2)'};
        "
      >
        {#each displayNotes as note, i (note.id)}
          <div role="note" class="note-container flex column"
            animate:flip={{ duration: 200, easing: cubicInOut }}
            style="background-color: {note.bg_color};"
            onpointerup={() => { const res = handlePointerUp({ array: notes, arrayType: "notes", idx: i, dragIndex: noteDragIndex, currentTabId }); if (res) noteDragIndex = res.dragIndex; }}
            data-index={i}
            class:hovered-over={noteDragIndex === i}
          >
            <button aria-label="Drag handle" class="drag-handle flex row"
              disabled={isDeleteModalVisible}
              onpointermove={(e) => { const res = handlePointerMove(e, noteDragIndex, "notes"); if (res) noteDragIndex = res.dragIndex; }}
              onpointerdown={(e) => { if (!isDeleteModalVisible) { const res = handlePointerDown(e, i); if (res) noteDragIndex = res.dragIndex; }}}
            >
              <span class="span-icon img-small" style="mask-image: url('/grip-dots.svg'); mask-position: center;"></span>
            </button>
            <NoteComponent {note} fontSize={editorState.fontSize} {noteColor} {toggleHeadingOptions} {zoomedNote} isNoteUpdating={$isNoteUpdateBatchOngoing}
              onFocusChange={(controls) => { focusedNoteControls = controls; lastFocusedNoteId = controls.id; }}
              setZoomedNote={(noteId) => { zoomedNoteId = noteId; }}
              setDeleteModalVisibility={(state) => { isDeleteModalVisible = state; }}
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
        <div
          class="notes-tab-outer-container flex row"
          class:currentTab={tab.id === currentTabId}
          class:in-editmode={tab.id === editingTabId}
          class:hovered-over={tabDragIndex === i}
          class:disabled={isDeleteModalVisible}
          style="background-color: {tab.color};"
          role="tab"
          tabindex="0"
          onpointerup={() => { const res = handlePointerUp({ array: tabs, arrayType: "tabs", idx: i, dragIndex: tabDragIndex }); if (res) tabDragIndex = res.dragIndex; }}
          data-index={i}
          animate:flip={{ duration: 200, easing: cubicInOut }}
        >
          <button aria-label="Drag handle" class="drag-handle flex row"
            disabled={isDeleteModalVisible}
            onpointermove={(e) => { const res = handlePointerMove(e, tabDragIndex, "tabs"); if (res) tabDragIndex = res.dragIndex; }}
            onpointerdown={(e) => { if (!isDeleteModalVisible) { const res = handlePointerDown(e, i); if (res) tabDragIndex = res.dragIndex; }}}
          >
            <span class="span-icon img-small" style="mask-image: url('/grip-dots.svg'); mask-position: center;"></span>
          </button>
          {#if editingTabId === tab.id}
            <input
              bind:value={editingTabTitle}
              bind:this={editingTabInput}
              class="transparent-input"
              type="text"
              onblur={() => saveTabEdit()}
              onkeydown={(e) => { if (e.key === "Enter") saveTabEdit(); if (e.key === "Escape") exitTabEdit(); }}
            />
            {#each [editingTabInput], i (i)}
              {onMount(() => editingTabInput?.focus())}
            {/each}
          {:else}
            <button
              class="button-primary transparent"
              style="color: {tab.color === "black" ? 'black' : 'var(--color-white-primary1)'}"
              onclick={() => currentTabId = tab.id}
              oncontextmenu={(e) => { e.preventDefault(); handleContextMenu(tab.id); }}
              ondblclick={() => handleTabEditStart()}
              disabled={isDeleteModalVisible}
              onmouseenter={() => hover.enter({ content: tab.title })}
              onmouseleave={(e) => hover.leave(e)}
            >
              <span style="user-select: none; color: {blackTextColors.some((color) => color.value === tab.color) ? "black" : "var(--color-white-primary1)"};">
                {tab.title}
              </span>
            </button>
          {/if}
        </div>
      {/each}
    </div>
  </div>
</div>

<style>
  #notes-page-hover-title-content {
    margin: 0;
    padding: 0.25rem 0.5rem;
    background-color: var(--color-secondary2);
  }

  .toolbar-toggle {
    position: absolute;
    padding: 2px 0.5rem;
  }

  #notes-main-container {
    position: relative;
    justify-content: space-between;
    height: 100%;
    width: 100%;

    .toolbar-toggle {
      top: 7rem;
      transform: translateY(-50%);

      &.hidden {
        transform: translateY(calc(-7rem - 50%));
      }
    }
  }

  #notes-main-toolbar {
    justify-content: flex-start;
    flex-shrink: 0;
    width: 100%;
    height: 7rem;

    &.hidden {
      height: 0;
    }

    .primary-toolbar {
      overflow-x: auto;
      overflow-y: hidden;

      &:nth-of-type(2) {
        align-items: flex-start;
        padding: 0.5rem 0.5rem 4px 0.5rem;

        button {
          margin-top: 0.25rem;
          border-radius: 0.25rem;
        }

        :global(div.item-selector-component-container) {
          margin-top: 0.25rem;
        }
      }

      &.note-zoomed {
        position: fixed;
        z-index: 100;
        top: 0;
        width: 100%;
      }
    }
  }

  #notes-container {
    display: grid;
    gap: 20px;
    padding: 20px 12px;
    width: 100%;
    height: 100%;
    overflow-y: auto;
    scrollbar-gutter: stable both-edges;
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
    flex-shrink: 0;
    width: 100%;
    height: 2rem;
    padding: 0 0.25rem 0 0;
    border-top: 1px solid var(--outline-color2);
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
      height: 100%;
      border-radius: 0;
      margin-right: 0.25rem;
      white-space: nowrap;
    }
  }

  #notes-tabs-list {
    height: 100%;
    justify-content: flex-start;
    align-items: flex-start;
    padding: 1px 0;
    overflow-x: auto;
    overflow-y: hidden;

    button.button-primary.transparent {
      position: relative;
      width: 6rem;
      height: 100%;
      padding: 0;
      outline: none;
      overflow: hidden;

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
      }
    }

    .transparent-input {
      align-self: flex-start;
      max-width: 6rem;
      z-index: 1;
      outline: none;
      font: inherit;
      font-size: 14px;
    }
  }

  .notes-tab-outer-container {
    position: relative;
    flex-shrink: 0;
    height: 100%;
    border-right: 1px solid var(--outline-color2);
    padding-right: 24px;

    &.currentTab {
      outline: 1px solid var(--color-highlight1);
    }

    &:first-of-type {
      border-left: 1px solid var(--outline-color2);
      margin-left: 1px;
    }

    &:not(.disabled):hover::before {
      position: absolute;
      content: "";
      inset: 0;
      z-index: 0;
      background-color: var(--hover-color-transparent) !important;
    }
  }

  #zoomed-note-container {
    position: fixed;
    inset: 0;
    z-index: 100;
    background-color: var(--color-primary2);

    .toolbar-toggle {
      top: 3.5rem;

      &.hidden {
        transform: translateY(-3.5rem);
      }
    }
  }

  #zoomed-note-wrapper {
    width: 100%;
    height: 100%;
    padding: 120px 25%;
  }

  #zoomed-note-saving {
    position: fixed;
    top: 3.5rem;
    margin-top: 1.5rem;
    font-weight: bold;
    user-select: none;
  }
</style>