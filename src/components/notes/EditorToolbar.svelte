<script lang="ts">
  import type { Note, EditorState, FocusedNoteControls } from "$lib/types";
  import { i18n } from "$lib/i18n/i18n.svelte";
  import { updateUserPrefs, userPrefs } from "$lib/utils/prefsStore";
  import { handleHorizontalScroll } from "$lib/utils/actions.svelte";
  import { isNoteUpdateBatchOngoing } from "$lib/notes";
  import { viewport } from "$lib/utils/viewport";

  import ItemSelect from "../elements/ItemSelect.svelte";

  interface Props {
    /** Used to null the zoomed note ID. */
    nullZoomedNoteId: () => void;
    /** Gets called when the detach/attach or lock/unlock buttons are pressed. */
    onToggle?: () => void;
    /** Used to return the mouse event and content for the hover title in the parent. */
    hoverEnter: (e: MouseEvent, content: string) => void;
    /** Used to return the mouse event for the hover title in the parent. */
    hoverLeave: (e: MouseEvent) => void;
    /** A callback to trigger handleColorMenu on the parent. */
    handleColorMenu: () => void;
    /** A callback to return headings modal visibility state. */
    setHeadings: (state: boolean) => void;
    /** Used to return the button reference that was clicked on and an ID to differentiate them. */
    setButtonRef: (btn: "colors" | "headings", ref: HTMLButtonElement | null) => void;
    /** Used to send the changed fields of editorState back to the parent. */
    patchEditorState: (patch: Partial<EditorState>) => void;
    /** The current note being zoomed. */
    zoomedNote: Note | undefined;
    editorState: EditorState;
    focusedNoteControls: FocusedNoteControls;
    currentTabId: number | null;
    isToolbarDetached: boolean;
    isToolbarUnlocked: boolean;
    isColorForNotes: boolean;
    isColorOptions: boolean;
    isHeadings: boolean;
  }

  let {
    nullZoomedNoteId,
    onToggle,
    hoverEnter,
    hoverLeave,
    handleColorMenu,
    setHeadings,
    setButtonRef,
    patchEditorState,
    zoomedNote,
    editorState,
    focusedNoteControls,
    currentTabId,
    isToolbarDetached,
    isToolbarUnlocked,
    isColorForNotes,
    isColorOptions,
    isHeadings,
  }: Props = $props();

  const toolbarEditorOthers = [
    {
      get ariaLabel() { return (i18n.t["note-toolbar.detach-attach"] as string[])[isToolbarDetached ? 0 : 1] + " " + (i18n.t["note-toolbar.detach-attach"] as string[])[2]; },
      disabled: null,
      icon: "/link.svg",
      onMount: null,
      get toggled() { return isToolbarDetached; },
      onClick: () => { updateUserPrefs("notePrefs", "editorToolbarState", { ...$userPrefs.notePrefs.editorToolbarState, detached: !isToolbarDetached }); onToggle?.(); },
      onMouseEnter: (e: MouseEvent) => hoverEnter(e, (i18n.t["note-toolbar.detach-attach"] as string[])[isToolbarDetached ? 0 : 1] + " " + (i18n.t["note-toolbar.detach-attach"] as string[])[2]),
      onMouseLeave: (e: MouseEvent) => hoverLeave(e),
    },
    {
      get ariaLabel() { return (i18n.t["note-toolbar.toggle-lock"] as string[])[isToolbarUnlocked ? 0 : 1] + " " + (i18n.t["note-toolbar.toggle-lock"] as string[])[2]; },
      get disabled() { return !isToolbarDetached; },
      icon: "/lock.svg",
      onMount: null,
      get toggled() { return isToolbarUnlocked; },
      onClick: () => { updateUserPrefs("notePrefs", "editorToolbarState", { ...$userPrefs.notePrefs.editorToolbarState, draggable: !isToolbarUnlocked }); onToggle?.(); },
      onMouseEnter: (e: MouseEvent) => hoverEnter(e, (i18n.t["note-toolbar.toggle-lock"] as string[])[isToolbarUnlocked ? 0 : 1] + " " + (i18n.t["note-toolbar.toggle-lock"] as string[])[2]),
      onMouseLeave: (e: MouseEvent) => hoverLeave(e),
    },
    {
      get ariaLabel() { return i18n.t["exit-zoom.button"] as string; },
      get disabled() { return !zoomedNote || $isNoteUpdateBatchOngoing; },
      icon: "/zoom-out.svg",
      onMount: null,
      toggled: null,
      onClick: () => { nullZoomedNoteId(); },
      onMouseEnter: (e: MouseEvent) => hoverEnter(e, i18n.t["exit-zoom.button"] as string),
      onMouseLeave: (e: MouseEvent) => hoverLeave(e),
    },
    {
      get ariaLabel() { return (i18n.t["note-toolbar.button.titles"] as string[])[(i18n.t["note-toolbar.button.titles"] as string[]).length - 1]; },
      get disabled() { return !currentTabId },
      icon: "/palette.svg",
      onMount: (el: HTMLButtonElement | null) => setButtonRef("colors", el),
      get toggled() { return isColorOptions && isColorForNotes; },
      onClick: () => { handleColorMenu(); },
      onMouseEnter: (e: MouseEvent) => hoverEnter(e, (i18n.t["note-toolbar.button.titles"] as string[])[(i18n.t["note-toolbar.button.titles"] as string[]).length - 1]),
      onMouseLeave: (e: MouseEvent) => hoverLeave(e),
    },
    {
      get ariaLabel() { return (i18n.t["note-toolbar.button.titles"] as string[])[(i18n.t["note-toolbar.button.titles"] as string[]).length - 2]; },
      get disabled() { return !currentTabId || focusedNoteControls?.isTitleActive; },
      icon: "/heading.svg",
      onMount: (el: HTMLButtonElement | null) => setButtonRef("headings", el),
      get toggled() { return isHeadings; },
      onClick: () => { setHeadings(!isHeadings); },
      onMouseEnter: (e: MouseEvent) => hoverEnter(e, (i18n.t["note-toolbar.button.titles"] as string[])[(i18n.t["note-toolbar.button.titles"] as string[]).length - 2]),
      onMouseLeave: (e: MouseEvent) => hoverLeave(e),
    }
  ];

  const toolBarEditorButtons = [
    { name: "underline", icon: "/underline.svg", get disabled() { return !currentTabId; }, get toggled() { return editorState.isUnderline; } },
    { name: "bold", icon: "/bold.svg", get disabled() { return !currentTabId; }, get toggled() { return editorState.isBold; } },
    { name: "italic", icon: "/italic.svg", get disabled() { return !currentTabId; }, get toggled() { return editorState.isItalic; } },
    { name: "bullet-list", icon: "/bulleted-list.svg", get disabled() { return !currentTabId; }, get toggled() { return editorState.isBulletList; } },
    { name: "toggle-tasklist", icon: "/checklist.svg", get disabled() { return !currentTabId; }, get toggled() { return editorState.isTaskListActive; } },
    { name: "split-listitem", icon: "/plus.svg", get disabled() { return !currentTabId || !editorState.canAddNewItem; }, toggled: null },
    { name: "sink-listitem", icon: "/indent.svg", get disabled() { return !currentTabId || !editorState.canIndent; }, toggled: null },
    { name: "lift-listitem", icon: "/outdent.svg", get disabled() { return !currentTabId || !editorState.canOutdent; }, toggled: null },
    { name: "align-left", icon: "/align-left.svg", get disabled() { return !currentTabId; }, get toggled() { return editorState.textAlignment === 'left'; } },
    { name: "align-center", icon: "/align-center.svg", get disabled() { return !currentTabId; }, get toggled() { return editorState.textAlignment === 'center'; } },
    { name: "align-right", icon: "/align-right.svg", get disabled() { return !currentTabId; }, get toggled() { return editorState.textAlignment === 'right'; } },
  ];
</script>

<div
  id="detachable-toolbar"
  class="primary-toolbar flex row"
  class:note-zoomed={zoomedNote}
  class:detached={isToolbarDetached}
  style:--toolbar-left={!zoomedNote ? `${$userPrefs.mainPrefs.gutterAt + 16}px` : "0"}
  style:--max-width={isToolbarDetached ? `${$viewport.width - (($userPrefs.mainPrefs.gutterAt + 24 + 16))}px` : "unset"}
  use:handleHorizontalScroll={{ scrollMultiplier: 0.4 }}
>
  {#each toolbarEditorOthers as btn, i (btn.ariaLabel)}
    <button
      {@attach (el) => { btn.onMount?.(el); return () => btn.onMount?.(null); }}
      class="button-primary transparent highlight"
      class:toggled={btn.toggled}
      aria-label={btn.ariaLabel}
      disabled={btn.disabled}
      onclick={btn.onClick}
      onmouseenter={(e) => btn.onMouseEnter(e)}
      onmouseleave={(e) => btn.onMouseLeave(e)}
    >
      <span class="span-icon img-small" style="mask-image: url({btn.icon});"></span>
    </button>
    {#if i === 2}
      <ItemSelect
        data={[...Array(40).keys()].map(i => i + 9 + "px")}
        onItemSelect={(item) => { patchEditorState({ fontSize: item }); focusedNoteControls?.applyProperty('set-fontsize'); }}
        overrideWithCustomValue={!editorState.fontSize ? i18n.t["notes.font-size.select"] as string : editorState.fontSize}
        position={{ isPositionAbsolute: false, usePortal: true, }}
        disabled={!currentTabId}
      />
      <div class="divider"></div>
    {/if}
  {/each}
  {#each toolBarEditorButtons as button, i (button.name)}
    {@const disabledForTitle = [3, 4, 5, 6, 7, 8, 9, 10].includes(i) && focusedNoteControls?.isTitleActive}
    <button class="button-primary transparent highlight"
      aria-label={(i18n.t["note-toolbar.button.titles"] as string[])[i]}
      disabled={disabledForTitle || button.disabled}
      class:toggled={button.toggled}
      onclick={() => focusedNoteControls?.applyProperty(button.name)}
      onmousedown={(e) => e.preventDefault()}
      onmouseenter={(e) => hoverEnter(e, (i18n.t["note-toolbar.button.titles"] as string[])[i])}
      onmouseleave={(e) => hoverLeave(e)}
    >
      <span class="span-icon img-small" style="mask-image: url('{button.icon}');"></span>
    </button>
  {/each}
</div>

<style>
  #detachable-toolbar {
    left: var(--toolbar-left);
    align-items: flex-start;
    padding: 0.5rem 0.5rem 4px 0.5rem;
    overflow-x: auto;
    overflow-y: hidden;

    button {
      margin-top: 0.25rem;
      border-radius: 0.25rem;
    }

    :global(div.item-selector-component-container) {
      margin-top: 0.25rem;
    }

    .divider {
      height: 100%;
      min-width: 0;
      border-left: 1px solid var(--outline-color2);
      transform: translateX(1px);
    }

    &.detached {
      max-width: var(--max-width);
      border: none;
    }

    &:not(.detached).note-zoomed {
      position: fixed;
      z-index: 100;
      top: 0;
      width: 100%;
    }
  }
</style>