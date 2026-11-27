<script lang="ts">
  import { fly } from "svelte/transition";
  import { cubicInOut } from "svelte/easing";

  import { getTransactions, transactions, getTransactionsByYear } from "#lib/transactions.svelte.ts";
  import { sendAlert } from "#lib/classes/alertStore.svelte.ts";
  import { i18n } from "#lib/i18n/i18n.svelte.ts";
  import type { Transaction } from "#lib/types.ts";
  import { viewport } from "#lib/utils/viewport.ts";
  import { formatMonthYear, toYearMonth } from "#lib/utils/helpers.svelte.ts";

  import BarChart from "../../components/charts/Bar.svelte";
  import LineChart from "../../components/charts/Line.svelte";
  import PieChart from "../../components/charts/Pie-Doughnut.svelte";
  import DoughnutChart from "../../components/charts/Pie-Doughnut.svelte";
  import ToggleSwitch from "../../components/elements/ToggleSwitch.svelte";
  import DateSelect from "../../components/elements/DateSelect.svelte";
  import ItemSelect from "../../components/elements/ItemSelect.svelte";

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

  let transactionsData = $state<Transaction[]>([])
  let chartKey = $state(0); // Used in making sure a new chart is always generated.
  let currentChart = $state<ChartType | null>(null);
  let selectChartValue = $state<ChartType>("bar");
  let isYearly = $state<boolean>(false);

  let selectedDate = $state<Date | null>(null);
  let dateToDraw = $state<string>('');
  let searchedDate = $state<string>('');

  $effect(() => {
    dateToDraw = selectedDate ? (isYearly ? String(selectedDate.getFullYear()) : formatMonthYear(selectedDate)) : '';
  });

  /***********************************************************************************************************************************\
  |
  | Context, Helper & Wrapper functions
  |
  \***********************************************************************************************************************************/
  const handleClear = () => {
    selectedDate = currentChart = null;
    isYearly = false;
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
  <div id="charts-toolbar" class="primary-toolbar flex row">
    <ItemSelect
      data={i18n.t["charts.chart-names"] as string[]}
      titleTranslationKey="charts.title.select-chart"
      onItemSelect={(item) => { if (selectChartValue !== item.toLowerCase()) selectChartValue = item.toLowerCase() as ChartType; }}
      position={{ isPositionAbsolute: false }}
    />
    <div id="toggle-switch-yearly" class="element-wrapper-for-title flex column">
      <p class="element-paragraph-title">
        {i18n.t["charts.full-year.toggle-switch"]}
      </p>
      <ToggleSwitch
        activeStateFrom={isYearly}
        onClickCommand={() => { isYearly = !isYearly; }}
        hoverContent={i18n.t["charts.toggle-switch.hover-title"] as string}
      />
    </div>
    <DateSelect
      hideMonths={isYearly ? true : false}
      overrideTitle={isYearly ? String((selectedDate ?? new Date()).getFullYear()) : selectedDate ? formatMonthYear(selectedDate) : undefined}
      onDateSelect={(date) => { selectedDate = date; }}
      hoverContent={i18n.t["charts.date-input.title"] as string}
    />
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

    #chart-container {
      width: 100%;
      height: 100%;
      background-color: var(--color-primary3);
      overflow: hidden;

      .chart-wrapper {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: flex-end;
        width: 100%;
        height: 100%;
        padding: 6px;
      }
    }
  }
</style>
