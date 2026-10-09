<script lang="ts">
  import { HoverTitle } from "$lib/actions.svelte";
  import { i18n } from "$lib/i18n/i18n.svelte";

  import ModalWrapper from "../ModalWrapper.svelte";



  let {
    options,
  }: {
    options: {
      date: string;
      bgColor?: "lighter" | "darker";
      noPadding?: boolean;
    }
  } = $props();

  const hover = new HoverTitle();

  const bgColor = $derived(options?.bgColor === "lighter" ? 'var(--color-secondary2)' : options?.bgColor === "darker" ? 'var(--color-primary2)' : 'var(--color-secondary1)');
  const monthColor = $derived(options?.bgColor === "lighter" ? 'var(--color-secondary3)' : options?.bgColor === "darker" ? 'var(--color-secondary1)' : 'var(--color-secondary2)');
  const dayColor = $derived(options?.bgColor === "lighter" ? 'var(--color-secondary2)' : options?.bgColor === "darker" ? 'var(--color-primary2)' : 'var(--color-secondary1)');
  const outlineColor = $derived(options?.bgColor === "lighter" ? 'var(--outline-color3)' : options?.bgColor === "darker" ? 'var(--outline-color1)' : 'var(--outline-color2)');
  const isPadding = $derived(options?.noPadding ? '0' : '6px');
  const height = $derived(options?.noPadding ? '3rem' : '3.5rem');
  const width = $derived(options?.noPadding ? '3rem' : '3.5rem');

  const monthIndex = $derived(new Date(options.date).getMonth());
  const day = $derived(options.date.slice(-2));
  const monthAbbrevs = $derived.by(() => {
    return (i18n.t["calendar.monthnames"] as string[]).map((month) => {
      return i18n.lang === 'en' ? month.slice(0, 3) : month.slice(0, -3);
    });
  });
</script>

{#if hover.isHovering}
  <ModalWrapper
    attributes={{ "hover-title-owner": hover.id }}
    options={{
      position: { moveTop: -36, centerElement: true },
      transition: { type: "fade", duration: 200, easing: "cubic-in-out" },
      borderRadius: 8,
      outline: { width: 1, color: 'var(--outline-color1)'},
    }}
  >
    <p id="date-box-hover-title-content">
      {options.date}
    </p>
  </ModalWrapper>
{/if}

<div
  id="date-box-container"
  class="flex column"
  style="
    background-color: {bgColor};
    padding: {isPadding};
    height: {height};
    width: {width};
  "  
>
  <div class="flex column"
    style="outline: 1px solid {outlineColor};"
  >
    <p
      style="
        background-color: {monthColor};
        border-bottom: 1px solid {outlineColor};
      "
    >
      {monthAbbrevs[monthIndex]}
    </p>
    <p
      style="background-color: {dayColor};"
      onmouseenter={() => hover.enter()}
      onmouseleave={(e) => hover.leave(e)}
    >
      {day}
    </p>
  </div>
</div>

<style>
  #date-box-hover-title-content {
    margin: 0;
    padding: 0.25rem 0.5rem;
    background-color: var(--color-secondary1);
  }

  #date-box-container {
    flex-shrink: 0;
    border-radius: 0.5rem;
    user-select: none;

    > div {
      width: 100%;
      height: 100%;
      border-radius: 6px;

      > p {
        align-content: center;
        width: 100%;
        height: 100%;
        margin: 0;
        padding: 2px;
        text-align: center;

        &:first-of-type {
          height: unset;
          border-radius: 6px 6px 0 0;
          font-size: 10px;
          line-height: 10px;
        }

        &:nth-of-type(2) {
          border-radius: 0 0 6px 6px;
          font-weight: bold;
          font-size: 12px;
          line-height: 12px;
        }
      }
    }
  }
</style>