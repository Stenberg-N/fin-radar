<script lang="ts">
  import { monthDifferencesMap, lastMonthMap, thisMonthMap, transactionCategoryTags } from "$lib/transactions";
  import { i18n } from "$lib/i18n/i18n.svelte";
  import { userPrefs } from "$lib/prefsStore";
  import { HoverTitle } from "$lib/actions.svelte";

  import ModalWrapper from "../ModalWrapper.svelte";

  type HoverTarget = { element: "header" | "new-tag" };

  const hover = new HoverTitle<HoverTarget>();

  $effect(() => {
    return () => { hover.destroy(); };
  })
</script>

{#if hover.isHovering}
  <ModalWrapper options={{
    transition: { type: "fade", easing: "cubic-in-out", duration: 200 },
    position: { moveTop: -60, centerElement: true },
    outline: { width: 1, color: 'var(--outline-color2)'},
    }}
  >
    <p id="home-hover-modal-content">
      {(i18n.t["transactions-feed.texts"] as string[])[hover.target?.element === 'header' ? 1 : 2]}
    </p>
  </ModalWrapper>
{/if}

<div id="transactions-feed-container" class="flex column">
  <h2 onmouseenter={() => hover.enter({ element: "header" })} onmouseleave={(e) => hover.leave(e)}>
    {i18n.t["transactions-feed.header"]}
  </h2>
  <div id="transactions-feed-content">
    {#if $monthDifferencesMap.size > 0}
      {#each $monthDifferencesMap as [ category, value ], i (i)}
        <div class="feed-item flex column">
          <div class="feed-item-topbar flex row">
            <p>
              {(() => {
                const item = transactionCategoryTags.find(k => k === category.split("-")[0]);
                return item
                  ? (i18n.t["add-transaction.categories"] as Record<string, string>)[item]
                  : 'Unknown';
              })()}
            </p>
            {#if category.endsWith("-new")}
              <p onmouseenter={() => hover.enter({ element: "new-tag" })} onmouseleave={(e) => hover.leave(e)}>
                {(i18n.t["transactions-feed.texts"] as string[])[0]}
              </p>
            {/if}
          </div>
          <div class="feed-item-content flex column">
            <div class="flex">
              <span class="span-icon img-small-medium" style="mask-image: url('/sack.svg');"></span>
              {thisMonthMap.get(category.endsWith("-new") ? category.split('-')[0] : category) + $userPrefs.mainPrefs.currency}
            </div>
            <div class="flex">
              <span class="span-icon img-medium-large" style="mask-image: url('/arrow-up.svg'); transform: rotate({value > 0 ? '' : '180deg'}); background-color: var(--color-{value > 0 ? 'negative' : 'positive'});"></span>
              {(value > 0 ? "+" : "") + value + "%"}
              {
                `${(() => {
                  const res = (thisMonthMap.get(category.endsWith("-new") ? category.split('-')[0] : category) ?? 0) - (lastMonthMap.get(category) ?? 0);
                  return `(${(res < 0 ? "-" : "+") + Math.abs(res) + $userPrefs.mainPrefs.currency})`;
                })()}`
              }
            </div>
          </div>
        </div>
      {/each}
    {:else}
      <p style="align-self: center; margin-top: 40%; user-select: none; font-weight: bold;">
        {i18n.t["transactions-feed.nothing-to-report"]}
      </p>
    {/if}
  </div>
</div>

<style>
  #home-hover-modal-content {
    max-width: 360px;
    padding: 0.25rem 0.5rem;
    background-color: var(--color-secondary2);
    margin: 0;
    text-align: center;
  }

  #transactions-feed-container {
    justify-content: flex-start;
    flex: 1 1 auto;
    height: 100%;
    max-width: 360px;
    padding: 1rem 0.25rem 0.25rem;

    h2 {
      width: 100%;
      margin: 0;
      padding-bottom: 1rem;
      border-bottom: 2px solid var(--outline-color1);
      text-align: center;
    }

    #transactions-feed-content {
      display: grid;
      grid-template-columns: 1fr;
      width: 100%;
      gap: 1rem;
      padding: 1rem 0.25rem;
      mask-image: linear-gradient(to top, rgba(0, 0, 0, 0), rgb(0, 0, 0) 2%, rgb(0, 0, 0) 98%, rgba(0, 0, 0, 0));
      overflow-y: auto;
      scrollbar-gutter: stable both-edges;

      .feed-item {
        align-items: flex-start;
        width: 100%;
        min-width: fit-content;
        padding: 0.5rem 0.5rem 1rem 1rem;
        gap: 1rem;
        border-radius: 0.5rem;
        background-color: var(--color-secondary1);
        font-size: clamp(14px, 1.2cqw, 1rem);

        .feed-item-content {
          align-items: flex-start;
          gap: 0.25rem;

          > div {
            gap: 0.25rem;

            &:first-of-type {
              span {
                margin: 0.25rem;
              }
            }
          }
        }

        .feed-item-topbar {
          justify-content: space-between;
          width: 100%;

          > p {
            align-self: flex-start;
            margin: 0;
            text-shadow: 0 4px 8px rgba(0, 0, 0, 0.8);

            &:first-of-type {
              font-weight: bold;
              color: var(--color-highlight1);
            }

            &:nth-of-type(2) {
              padding: 0.25rem 0.5rem;
              border-radius: 0.25rem;
              background-color: var(--color-secondary2);
              color: var(--color-highlight3);
            }
          }
        }
      }
    }
  }
</style>