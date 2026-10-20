<script lang="ts">
  import { fly } from "svelte/transition";
  import { cubicInOut } from "svelte/easing";

  import { getTransactions, transactions, getTransactionsByYear } from "$lib/transactions";
  import { sendAlert } from "$lib/alert";
  import { i18n } from "$lib/i18n/i18n.svelte";
  import type { Transaction } from "$lib/types";
  import { viewport } from "$lib/utils/viewport";
  import { HoverTitle } from "$lib/classes/hoverTitle.svelte";
  import { formatMonthYear, toYearMonth } from "$lib/utils/helpers.svelte";

  import BarChart from "../../components/charts/Bar.svelte";
  import LineChart from "../../components/charts/Line.svelte";
  import PieChart from "../../components/charts/Pie-Doughnut.svelte";
  import DoughnutChart from "../../components/charts/Pie-Doughnut.svelte";
  import ToggleSwitch from "../../components/elements/ToggleSwitch.svelte";
  import ModalWrapper from "../../components/ModalWrapper.svelte";
  import DateSelect from "../../components/elements/DateSelect.svelte";
  import ItemSelect from "../../components/elements/ItemSelect.svelte";

  type HoverTarget = { element: "date-input" };
  type ChartType = "bar" | "line" | "pie" | "doughnut";

  const chartComponents = {
    bar: BarChart,
    line: LineChart,
    pie: PieChart,
    doughnut: DoughnutChart
  };

  const toolbarButtons = [
    {
      ariaLabel: "Clear chart",
      get title() { return i18n.t["clear.button"]; },
      icon: "/trash-can.svg",
      onClick: () => handleClear(),
    },
    {
      ariaLabel: "Draw chart",
      get title() { return i18n.t["charts.button.draw"]; },
      icon: "/edit-pen.svg",
      onClick: () => populateTransactions(),
    }
  ];

  const elementWrappers = [
    {
      get title() { return i18n.t["charts.full-year.toggle-switch"]; },
      id: "toggle-switch-yearly"
    },
    {
      get title() { return i18n.t["date-input.description"]; },
      id: "selected-date-container"
    },
  ];

  const hover = new HoverTitle<HoverTarget>();

  let transactionsData = $state<Transaction[]>([])
  let chartKey = $state(0); // Used in making sure a new chart is always generated.
  let currentChart = $state<ChartType | null>(null);
  let selectChartValue = $state<ChartType>("bar");
  let isYearly = $state<boolean>(false);

  let selectedDate = $state<Date | null>(null);
  let dateToDraw = $state<string>('');
  let searchedDate = $state<string>('');

  $effect(() => {
    return () => { hover.destroy(); };
  });

  $effect(() => {
    dateToDraw = selectedDate ? (isYearly ? String(selectedDate.getFullYear()) : formatMonthYear(selectedDate)) : '';
  });

  /***********************************************************************************************************************************\
  |
  | Context, Helper & Wrapper functions
  |
  \***********************************************************************************************************************************/
  const handleClear = () => {
    currentChart = null;
    isYearly = false;
    selectedDate = null;
    searchedDate = '';
  };

  /***********************************************************************************************************************************/

  const populateTransactions = async () => {
    currentChart = null;
    chartKey++;

    if (dateToDraw.trim().length > 0) {
      const dateParts = dateToDraw.split('-');
      if (!/^\d{4}$/.test(dateParts[0])) { sendAlert({ message: "alert.invalid-year", isTimer: true, buttons: false }); currentChart = null; return; }
      if (!/^0*([1-9]|1[0-2])$/.test(dateParts[1]) && !isYearly) { sendAlert({ message: "alert.invalid-month", isTimer: true, buttons: false }); currentChart = null; return; }

      if (isYearly) {
        const getTransactions = await getTransactionsByYear(dateToDraw);
        if (getTransactions.success) {
          transactionsData = getTransactions.data;
        }
      } else {
        await getTransactions(dateToDraw);
        transactionsData = $transactions;
      }

      searchedDate = dateToDraw;
    } else {
      if (isYearly) {
        const year = ((d) => `${String(d.getFullYear())}`)(new Date());
        const getTransactions = await getTransactionsByYear(year);
        if (getTransactions.success) {
          transactionsData = getTransactions.data;
        }
        searchedDate = year;
      } else {
        const yearMonth = toYearMonth(new Date());
        await getTransactions(yearMonth);
        transactionsData = $transactions;
        searchedDate = yearMonth;
      }
    }

    if (transactionsData.length <= 0) { sendAlert({ message: "alert.no-transaction-data", isTimer: true, buttons: false }); return; };

    transactionsData = Object.values(transactionsData).sort((a, b) => {
      const [yearA, monthA] = a.date.split("-").map(Number);
      const [yearB, monthB] = b.date.split("-").map(Number);
      return yearA !== yearB ? yearA - yearB : monthA - monthB;
    });

    currentChart = selectChartValue;
  };
</script>

<div id="charts-main-container" class="flex column">
  {#if hover.isHovering}
    {@const content = (() => {
      switch (hover.target?.element) {
        case "date-input": return i18n.t["charts.date-input.title"] as string;
      }
    })()}

    <ModalWrapper
      attributes={{ "hover-title-owner": hover.id }}
      position={{ moveTop: -30 }}
      transition={{ type: "fade", duration: 200, easing: "cubic-in-out" }}
      borderRadius={8}
      outline={{ width: 1, color: 'var(--outline-color1)'}}
      focus={false}
    >
      <p id="charts-page-hover-title-content">
        {content}
      </p>
    </ModalWrapper>
  {/if}

  <div id="charts-toolbar" class="primary-toolbar flex row">
    <ItemSelect
      data={i18n.t["charts.chart-names"] as string[]}
      titleTranslationKey="charts.title.select-chart"
      onItemSelect={(item) => { if (selectChartValue !== item.toLowerCase()) selectChartValue = item.toLowerCase() as ChartType; }}
      position={{ isPositionAbsolute: false, }}
    />
    {#each elementWrappers as el, i (el.id)}
      <div id={el.id} class="element-wrapper-for-title flex column">
        <p class="element-paragraph-title">
          {el.title}
        </p>
        {#if i === 0}
          <ToggleSwitch
            activeStateFrom={isYearly}
            onClickCommand={() => { isYearly = !isYearly; }}
            translationKey="charts.full-year.toggle-switch"
          />
        {:else}
          <input bind:value={dateToDraw} class="primary-input" placeholder={!isYearly ? (i18n.t["placeholder.isodate"] as string).slice(0, 7) : (i18n.t["placeholder.isodate"] as string).slice(0, 4)} readonly
            onmouseenter={() => hover.enter({ element: "date-input" })}
            onmouseleave={(e) => hover.leave(e)}
          />
        {/if}
      </div>
      {#if i === 0}
        <DateSelect
          hideRefresh={true}
          hideMonths={isYearly ? true : false}
          overrideTitle={i18n.t["select-date.button"] as string}
          onDateSelect={(date) => { selectedDate = date; }}
        />
      {/if}
    {/each}
    {#each toolbarButtons as btn, i (i)}
      <button aria-label={btn.ariaLabel} class="button-primary {i === 0 ? 'transparent highlight outline default-corners' : 'white-bg'}" onclick={btn.onClick}>
        <span class="span-icon img-small" style="mask-image: url({btn.icon});"></span>
        {btn.title}
      </button>
    {/each}
  </div>

  <div id="chart-container">
    {#key chartKey}
      {@const chartType = currentChart}
      {#if chartType !== null && chartComponents[chartType]}
        {@const chartProps = { transactionsData, searchedDate, ...(chartType === "pie" || chartType === "doughnut" ? { type: chartType } : {}) }}
        {@const ChartComponent = chartComponents[chartType]}
        <div class="chart-wrapper" in:fly={{ x: $viewport.width, duration: 800, easing: cubicInOut }} out:fly={{ x: -$viewport.width, duration: 800, easing: cubicInOut }}>
          <ChartComponent {...chartProps} />
        </div>
      {/if}
    {/key}
  </div>
</div>

<style>
  #charts-main-container {
    width: 100%;
    height: 100%;
    justify-content: flex-start;

    #charts-page-hover-title-content {
      margin: 0;
      padding: 0.25rem 0.5rem;
      background-color: var(--color-secondary1);
    }

    #charts-toolbar {
      #selected-date-container {
        input {
          field-sizing: content;
          cursor: default;
        }
      }
    }

    #chart-container {
      width: 100%;
      height: 100%;
      background-color: var(--color-primary3);
      overflow: hidden;
    }
  }
</style>