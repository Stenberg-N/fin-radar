<script lang="ts">
  import { HoverTitle } from "$lib/classes/hoverTitle.svelte";
  import { i18n } from "$lib/i18n/i18n.svelte";

  import ModalWrapper from "../ModalWrapper.svelte";
  import ToggleSwitch from "../elements/ToggleSwitch.svelte";

  type HoverContent = { content: string };

  interface Props {
    /** Used to return a boolean to hide the component on outside click. */
    onOutsideClick: (state: boolean) => void;
    /** Used to return the color that was clicked on. */
    selectColor: (color: string) => void;
    /**
     * An array of colors and their hover titles.
     * 
     * @example
     * [
     *  { value: '#fff', title: ["White", "Valkoinen"] },
     *  { value: 'black', title: ["Black", "Musta"] },
     *  ...
     * ]
     */
    availableColors: { value: string; title: string[]; }[];
    /** Elements for the outside click handler to ignore. */
    ignorableEls?: (HTMLElement | null)[];
    /** Pass this when used for modifying notes' background color. */
    isColorForNotes?: boolean;
    /** Pass this when notes' text can be recolored. */
    isColorForText?: boolean;
    theme?: "default" | "lighter" | "light";
  }

  let {
    onOutsideClick,
    selectColor,
    availableColors,
    ignorableEls,
    isColorForNotes,
    isColorForText = $bindable(),
    theme = "default"
  }: Props = $props();

  const themes = {
  default: {
    bg: "var(--color-secondary2)",
    outline: "var(--outline-color2)"
  },
  lighter: {
    bg: "var(--color-secondary3)",
    outline: "var(--outline-color3)"
  },
  light: {
    bg: "var(--color-secondary4)",
    outline: "var(--outline-color4)"
  }
} as const;
const t = $derived(themes[theme] ?? themes["default"]);

const ht = $derived.by(() => {
  const obj: Record<"bg" | "outline", string> = { bg: '', outline: '' };
  for (const prop in t) {
    const parts = t[prop as "bg" | "outline"].split(/(\d)/);
    obj[prop as "bg" | "outline"] = parts[0] + String(Number(parts[1]) + 1) + parts[2]
  }
  return { obj };
});

const colorMenuSections = $derived([
    availableColors.slice(0, 12),
    availableColors.slice(12),
]);

const hover = new HoverTitle<HoverContent>();

</script>

<ModalWrapper
  position={{
    isDraggable: true,
  }}
  transition={{ type: "fade", duration: 200, easing: "cubic-in-out" }}
  onOutsideClick={() => { onOutsideClick(false) }}
  ignorableEls={ignorableEls}
  dragHandleColor={isColorForNotes !== undefined ? "lighter" : "lighter2"}
  outline={{ width: 1, color: ht.obj.outline }}
>
  {#if hover.isHovering}
    <ModalWrapper
      attributes={{ "hover-title-owner": hover.id }}
      position={{ centerElement: true, moveTop: 24 }}
      transition={{ type: "fade", duration: 200, easing: "cubic-in-out" }}
      borderRadius={8}
      outline={{ width: 1, color: ht.obj.outline }}
      focus={false}
    >
      <p id="color-menu-hover-title-content" style="background-color: {ht.obj.bg}">
        {hover.target?.content}
      </p>
    </ModalWrapper>
  {/if}

  <div
    class="flex column notes-color-menu"
    style:--bg={t.bg}
    style:--secondary-bg={ht.obj.bg}
  >
    <div id="colors-wrapper" class="flex row">
      {#each colorMenuSections as colors (colors)}
        <div class="color-menu-grid">
          {#each colors as color (color)}
            <button class="button-primary" style="background-color: {color.value};"
              aria-label={i18n.lang === 'en' ? color.title[0] : color.title[1]}
              onclick={() => selectColor(color.value)}
              onmouseenter={() => hover.enter({ content: i18n.lang === 'en' ? color.title[0] : color.title[1] })}
              onmouseleave={(e) => hover.leave(e)}
            ></button>
          {/each}
        </div>
      {/each}
    </div>

    {#if isColorForNotes}
      <div class="element-wrapper-for-title flex column">
        <p class="element-paragraph-title">{i18n.t["notes.for-text-color.option"]}</p>
        <ToggleSwitch
          activeStateFrom={isColorForText ?? false}
          onClickCommand={() => { isColorForText = !isColorForText; }}
          translationKey="notes.for-text-color.option"
        />
      </div>
    {:else}
      <button
        id="color-reset-button"
        class="button-primary transparent highlight default-corners lower-padding"
        onclick={() => selectColor(isColorForNotes !== undefined ? '#1c1c1c' : 'transparent')}
        onmouseenter={() => hover.enter({ content: (i18n.t["notes.reset-color"] as string[])[0] + (i18n.t["notes.reset-color"] as string[])[isColorForNotes !== undefined ? 2 : 1] })}
        onmouseleave={(e) => hover.leave(e)}
      >
        {i18n.t["reset.button"]}
      </button>
    {/if}
  </div>
</ModalWrapper>

<style>
  #color-menu-hover-title-content {
    margin: 0;
    padding: 0.25rem 0.5rem;
  }

  .notes-color-menu {
    align-items: flex-start;
    flex-wrap: wrap;
    padding: 1rem;
    gap: 1rem;
    background-color: var(--bg);
    box-shadow: 0 8px 16px rgba(0, 0, 0, 0.8);

    #colors-wrapper {
      gap: inherit;
    }

    p:not(.element-paragraph-title) {
      font-size: clamp(0.75rem, 0.9cqw, 1rem);
    }

    .color-menu-grid {
      display: grid;
      grid-template-columns: repeat(4, 1fr);
      gap: 0.5rem;
      padding: 1rem;
      border-radius: 0.5rem;
      background-color: var(--secondary-bg);
    }

    .element-wrapper-for-title, #color-reset-button {
      align-self: flex-end;
    }

    button {
      &:not(#toggle-track, #color-reset-button) {
        position: relative;
        width: 2rem;
        height: 2rem;
        box-shadow: 0 4px 8px rgba(0, 0, 0, 0.8);
        transition: transform 0.2s, box-shadow 0.2s;
      }

      &:not(#toggle-track, #color-reset-button):hover {
        transform: scale(1.1);
        box-shadow: 0 8px 16px rgba(0, 0, 0, 0.8);
      }

      &:not(#toggle-track, #color-reset-button)::before {
        position: absolute;
        content: "";
        inset: 0;
        width: 2rem;
        height: 2rem;
        border-radius: inherit;
        background-color: #fff;
        opacity: 0.05;
      }
    }
  }
</style>