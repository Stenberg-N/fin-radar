<script lang="ts">
  import { i18n } from "$lib/i18n/i18n.svelte";
  import { userPrefs } from "$lib/utils/prefsStore";

  import ModalWrapper from "../ModalWrapper.svelte";

  let {
    options,
  }: {
    options: {
      /** Specifies the key for a translation to give the items list a title. */
      titleTranslationKey?: string;
      /** The data to render as options. */
      data: string[];
      /** Triggers when an item is clicked. Returns the the item's value. */
      onItemSelect: (item: string) => void;
      /** Icons paired with items by index. If provided, each icon is rendered alongside the item at the same position. */
      icons?: (string | null)[];
      /**
       * - The initial selected item for the component is the first item in the data array.
       * - This value can be used to override that.
       *   - E.g. pass the value at data[3] instead.
       * - The value passed must be from the data array!
       */
      overrideDefaultStartValue?: string;
      /**
       * Used to give a custom, visual-only initial selected item for the component.
       */
      overrideWithCustomValue?: string;
      /**
       * Used to change the visual values for the data. 
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
      /** Used to disable the component */
      disabled?: boolean;
    }
  } = $props();

  const {
    data = [],
    titleTranslationKey = "",
    overrideDefaultStartValue = undefined,
    overrideWithCustomValue = undefined,
    overrideDataValues: rawOverrideValues = undefined,
    icons: rawIcons = undefined,
    disabled = false,
  } = $derived(options);

  const measured: { left?: number, top?: number } = $state({});
  const position = $derived({
    isPositionAbsolute: options.position?.isPositionAbsolute ?? true,
    centerElement: options.position?.centerElement ?? true,
    left: measured.left ?? options.position?.left ?? 0,
    top: measured.top ?? options.position?.top ?? 0,
    usePortal: options.position?.usePortal ?? false,
  })

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
  const MAIN_BAR = 34;
  let selectEl = $state<HTMLButtonElement | null>(null);
  let areOptionsVisible = $state<boolean>(false);

  const updatePosition = () => {
    if (!selectEl) return;
    
    const rect = selectEl.getBoundingClientRect();
    measured.left = rect.left - (position.isPositionAbsolute && position.centerElement ? ($userPrefs.mainPrefs.navBarWidth + 16) - rect.width / 2 : position.centerElement ? -rect.width / 2 : 0);
    measured.top = rect.bottom - (!position.isPositionAbsolute ? -8 : MAIN_BAR);
  };

  const toggle = () => {
    if (!areOptionsVisible) updatePosition();
    areOptionsVisible = !areOptionsVisible;
  };

  const handleItemSelect = (item: string, i: number) => {
    selectedIndex = i;
    options.onItemSelect(item);
    areOptionsVisible = false;
  };
</script>

<svelte:window onresize={() => areOptionsVisible && updatePosition()} />

<div class="item-selector-component-container">
  {#if areOptionsVisible}
    <ModalWrapper
      options={{
        position: position,
        transition: { type: "fade", duration: 200, easing: "cubic-in-out" },
        outline: { width: 1, color: 'var(--outline-color1)'},
        borderRadius: 8,
        onOutsideClick: () => { areOptionsVisible = false; },
        ignorableEls: [selectEl]
      }}
    >
      <div class="modal-default item-selector">
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
    onclick={toggle}
    disabled={disabled}
  >
    {#if selectedItemIcon}
      <span class="span-icon img-small" style="mask-image: url({selectedItemIcon});"></span>
    {/if}
    <span style="font-size: clamp(0.75rem, 1.1cqw, 14px); text-overflow: ellipsis;">{label}</span>
    <span class="span-icon img-extra-small" style="mask-image: url('/arrow.svg'); transform: rotate({areOptionsVisible ? '' : '-90'}deg); transition: 0.1s ease-in-out;"></span>
  </button>
</div>

<style>

</style>