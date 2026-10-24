<script lang="ts">
  import { HoverTitle } from "$lib/classes/hoverTitle.svelte";
  import { i18n } from "$lib/i18n/i18n.svelte";

  import ModalWrapper from "../ModalWrapper.svelte";

  interface Props {
    /** The isodate to render. */
    date: string;
    theme?: "default" | "lighter" | "darker";
    /** Should there be padding. Defaults to `false`. */
    noPadding?: boolean;
  }

  let {
    date,
    theme = "default",
    noPadding = false,
  }: Props = $props();

  const themes = {
    default: {
      bg: "var(--color-secondary2)",
      month: "var(--color-secondary3)",
      day: "var(--color-secondary2)",
      outline: "var(--outline-color3)",
    },
    lighter: {
      bg: "var(--color-secondary3)",
      month: "var(--color-secondary4)",
      day: "var(--color-secondary3)",
      outline: "var(--outline-color4)",
    },
    darker: {
      bg: "var(--color-primary2)",
      month: "var(--color-secondary2)",
      day: "var(--color-primary2)",
      outline: "var(--outline-color2)",
    }
  };
  const t = $derived(themes[theme] ?? themes["default"]);

  const isPadding = $derived(noPadding ? '0' : '6px');
  const height = $derived(noPadding ? '3rem' : '3.5rem');
  const width = $derived(noPadding ? '3rem' : '3.5rem');

  const hover = new HoverTitle();
  const monthIndex = $derived(new Date(date).getMonth());
  const day = $derived(date.slice(-2));
  const monthAbbrevs = $derived.by(() => {
    return (i18n.t["calendar.monthnames"] as string[]).map((month) => {
      return i18n.lang === 'en' ? month.slice(0, 3) : month.slice(0, -3);
    });
  });
</script>

{#if hover.isHovering}
  <ModalWrapper
    attributes={{ "hover-title-owner": hover.id }}
    position={{ moveTop: -36, centerElement: true }}
    transition={{ type: "fade", duration: 200, easing: "cubic-in-out" }}
    borderRadius={8}
    outline={{ width: 1, color: 'var(--outline-color2)'}}
  >
    <p id="date-box-hover-title-content">
      {date}
    </p>
  </ModalWrapper>
{/if}

<div
  id="date-box-container"
  class="flex column"
  style:--bg={t.bg}
  style:--month={t.month}
  style:--day={t.day}
  style:--outline={t.outline}
  style="
    padding: {isPadding};
    height: {height};
    width: {width};
  "  
>
  <div class="flex column">
    <p>
      {monthAbbrevs[monthIndex]}
    </p>
    <p
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
    background-color: var(--color-secondary2);
  }

  #date-box-container {
    flex-shrink: 0;
    border-radius: 0.5rem;
    user-select: none;
    background-color: var(--bg);

    > div {
      width: 100%;
      height: 100%;
      border-radius: 6px;
      outline: 1px solid var(--outline);

      > p {
        align-content: center;
        width: 100%;
        height: 100%;
        margin: 0;
        padding: 2px;
        text-align: center;
        background-color: var(--month);
        border-bottom: 1px solid var(--outline);

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
          background-color: var(--day);
        }
      }
    }
  }
</style>