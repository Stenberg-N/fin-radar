<script lang="ts">
  import { i18n } from "$lib/i18n/i18n.svelte";
  import { userPrefs } from "$lib/utils/prefsStore";

  import ModalWrapper from "../ModalWrapper.svelte";

  interface Props {
    /** Specifies the key for a translation to give the items list a title. */
    titleTranslationKey?: string;
    /** The data to render as options. */
    data: string[];
    /** Triggers when an item is clicked. Returns the item's value. */
    onItemSelect: (item: string) => void;
    /** Icons paired with data items by index. If provided, each icon is rendered alongside the data item at the same position. */
    icons?: (string | null)[];
    /**
     * The initial selected item for the component is the first item in the data array. This value can be used to override that.
     * 
     * E.g. pass the value at data[3] instead, or any string.
     * 
     * After an item is selected, this value will get ignored.
     * Use this to initialize the component with the desired value.
     */
    overrideDefaultStartValue?: string;
    /**
     * A display-only string shown in the component in the place of the currently selected data item.
     * It does **not** change which item is actually selected. It only overrides what is rendered.
     * 
     * When set, this value always takes precedence over the selected item, regardless of which item is selected or how it changes.
     * 
     * Use this to pass a value when you need to show a custom label that overrides the selected item at all times.
     */
    overrideWithCustomValue?: string;
    /**
     * Used to change the display values for the data.
     */
    overrideDataValues?: string[];
    /**
     * Position data that gets passed down to ModalWrapper.
     * - `isPositionAbsolute`: determines is the item list position fixed or absolute. Defaults to `true`.
     * - `centerElement`: should the element be centered. Defaults to `true`.
     * - `left`: assign a left value.
     * - `top`: assign a top value.
     * - `usePortal`: should the ModalWrapper be spawned at document.body. Defaults to `false`.
     */
    position?: {
      isPositionAbsolute?: boolean;
      centerElement?: boolean;
      left?: number;
      top?: number;
      usePortal?: boolean;
    },
    /** A condition to disable the component */
    disabled?: boolean;
    onMouseLeave?: (e: MouseEvent) => void;
    onMouseEnter?: () => void;
  }

  let {
    titleTranslationKey = "",
    data = [],
    onItemSelect,
    icons: rawIcons,
    overrideDefaultStartValue = undefined,
    overrideWithCustomValue = undefined,
    overrideDataValues: rawOverrideValues,
    position: rawPosition,
    disabled = false,
    onMouseEnter,
    onMouseLeave,
  }: Props = $props();

  const measured: { left?: number, top?: number } = $state({});
  const position = $derived({
    isPositionAbsolute: rawPosition?.isPositionAbsolute ?? true,
    centerElement: rawPosition?.centerElement ?? true,
    left: measured.left ?? rawPosition?.left ?? 0,
    top: measured.top ?? rawPosition?.top ?? 0,
    usePortal: rawPosition?.usePortal ?? false,
  });

  const overrideDataValues = $derived(rawOverrideValues ? data.map((_, i) => rawOverrideValues[i] ?? null) : undefined);
  const icons = $derived(rawIcons ? data.map((_, i) => rawIcons[i] ?? null) : undefined);
  let selectedIndex = $state<number | null>(null);
  const selectedItem = $derived(selectedIndex !== null ? data[selectedIndex] : (overrideDefaultStartValue ?? data[0]));
  const selectedItemIcon = $derived(icons ? icons[selectedIndex ?? Math.max(data.indexOf(selectedItem), 0)] ?? null : null);
  const label = $derived.by(() => {
    if (overrideWithCustomValue) return overrideWithCustomValue;
    const i = data.indexOf(selectedItem);
    return overrideDataValues && i >= 0
      ? overrideDataValues[i] ?? selectedItem
      : selectedItem;
  });
  const MAIN_BAR = 34 + 4;
  let selectEl = $state<HTMLButtonElement | null>(null);
  let areOptionsVisible = $state<boolean>(false);

  const updatePosition = () => {
    if (!selectEl) return;
    
    const rect = selectEl.getBoundingClientRect();
    measured.left = rect.left - (position.isPositionAbsolute && position.centerElement ? ($userPrefs.mainPrefs.navBarWidth + 16) - rect.width / 2 : position.centerElement ? -rect.width / 2 : 0);
    measured.top = rect.bottom - (!position.isPositionAbsolute ? -4 : MAIN_BAR);
  };

  const toggle = () => {
    if (!areOptionsVisible) updatePosition();
    areOptionsVisible = !areOptionsVisible;
  };

  const handleItemSelect = (item: string, i: number) => {
    selectedIndex = i;
    onItemSelect(item);
    areOptionsVisible = false;
  };
</script>

<svelte:window onresize={() => areOptionsVisible && updatePosition()} />

<div class="item-selector-component-container">
  {#if areOptionsVisible}
    <ModalWrapper
      {position}
      transition={{ type: "fade", duration: 200, easing: "cubic-in-out" }}
      outline={{ width: 1, color: 'var(--outline-color2)'}}
      borderRadius={8}
      onOutsideClick={() => { areOptionsVisible = false; }}
      ignorableEls={[selectEl]}
      focus={true}
    >
      <div class="item-selector">
        {#if titleTranslationKey.trim().length > 0}
          <p class="item-selector-title">
            {i18n.t[titleTranslationKey] as string}
          </p>
        {/if}
        {#each data as option, i (i)}
          <button class="button-primary" onclick={() => handleItemSelect(option, i)}>
            {#if icons && icons.length > 0}
              <span class="span-icon img-small" style="mask-image: url({icons[i]});"></span>
            {/if}
            {overrideDataValues ? overrideDataValues[i] : option}
          </button>
        {/each}
      </div>
    </ModalWrapper>
  {/if}

  <button
    bind:this={selectEl}
    class="button-primary transparent highlight outline default-corners"
    class:toggled={areOptionsVisible}
    onclick={toggle}
    disabled={disabled}
    {...(onMouseEnter && onMouseLeave)
      ? { onmouseleave: (e) => onMouseLeave(e), onmouseenter: () => onMouseEnter() }
      : {}
    }
  >
    {#if selectedItemIcon}
      <span class="span-icon img-small" style="mask-image: url({selectedItemIcon});"></span>
    {/if}
    <span style="font-size: clamp(0.75rem, 1.1cqw, 14px);">{label}</span>
    <span class="span-icon img-extra-small" style="mask-image: url('/arrow.svg'); transform: rotate({areOptionsVisible ? '' : '-90'}deg); transition: 0.1s ease-in-out;"></span>
  </button>
</div>

<style>

</style>