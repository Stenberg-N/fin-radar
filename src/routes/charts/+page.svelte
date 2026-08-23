<script lang="ts">
  import { fly } from "svelte/transition";
  import { cubicInOut } from "svelte/easing";

  import { getTransactions, transactions, getTransactionsByYear } from "$lib/transactions";
  import { sendAlert } from "$lib/alert";
  import { t } from "$lib/i18n";
  import type { Transaction } from "$lib/types";
  import { viewport } from "$lib/viewport";

  import BarChart from "../../components/charts/Bar.svelte";
  import LineChart from "../../components/charts/Line.svelte";
  import PieChart from "../../components/charts/Pie-Doughnut.svelte";
  import DoughnutChart from "../../components/charts/Pie-Doughnut.svelte";
  import ToggleSwitch from "../../components/ToggleSwitch.svelte";

  let transactionsData = $state<Transaction[]>([])
  let chartKey = $state(0); // Used in making sure a new chart is always generated.
  let currentChart = $state<"bar" | "line" | "pie" | "doughnut" | null>(null);

  let dateToDraw = $state<string>('');
  let selectChartValue = $state<number>(1);
  let isYearly = $state<boolean>(false);
  let searchedDate = $state<string>('');

  const chartComponents = {
    bar: BarChart,
    line: LineChart,
    pie: PieChart,
    doughnut: DoughnutChart
  };

  /***********************************************************************************************************************************\
  |
  | Context, Helper & Wrapper functions
  |
  \***********************************************************************************************************************************/
  const handleClear = () => {
    currentChart = null;
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
        const yearMonth = ((d) => `${String(d.getFullYear())}-${String(d.getMonth() + 1).padStart(2, '0')}`)(new Date());
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

    switch(selectChartValue) {
      case 1: currentChart = "bar"; break;
      case 2: currentChart = "line"; break;
      case 3: currentChart = "pie"; break;
      case 4: currentChart = "doughnut"; break;
    }
  };
</script>

<div id="charts-main-container" class="vertical-flex-container">
  <div id="charts-toolbar" class="primary-toolbar horizontal-flex-container">
    <div class="element-wrapper-for-title vertical-flex-container">
      <p class="element-paragraph-title">{$t["charts.full-year.toggle-switch"]}</p>
      <ToggleSwitch
        activeDerivedFrom={isYearly}
        onClickCommand={() => isYearly = !isYearly}
        translationKey={"charts.full-year.toggle-switch"}
        height={25}
      />
    </div>
    <div class="element-wrapper-for-title vertical-flex-container">
      <p class="element-paragraph-title">{$t["date-input.description"]}</p>
      <div id="draw-date-input-container" class="horizontal-flex-container" style="position: relative;" title={$t["charts.date-input.title"] as string}>
        <input class="primary-input" placeholder={!isYearly ? $t["placeholder.isodate"].slice(0, 7) as string : $t["placeholder.isodate"].slice(0, 4) as string} bind:value={dateToDraw} />
        <button class="transparent-button-highlight" onclick={() => dateToDraw = ''}><img src="/close-x.svg" alt="Close" /></button>
      </div>
    </div>
    <div class="element-wrapper-for-title vertical-flex-container">
      <p class="element-paragraph-title">{$t["charts.chart-type.select"]}</p>
      <select class="primary-input" bind:value={selectChartValue}>
        {#each $t["charts.chart-names"] as option, i (i)}
          <option value={i+1}>{option}</option>
        {/each}
      </select>
    </div>
    <button class="primary-button" onclick={() => handleClear()}>{$t["clear.button"]}</button>
    <button class="primary-button" onclick={() => populateTransactions()}>{$t["charts.button.draw"]}</button>
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
  }

  #charts-toolbar select {
    max-width: 120px;
    font-size: clamp(0.75rem, 0.9cqw, 1rem);
  }
  #charts-toolbar select:hover {
    cursor: pointer;
    background: #222;
  }

  #charts-toolbar select option {
    background-color: #0f0f0f;
  }

  #draw-date-input-container input {
    max-width: 110px;
    min-width: 95px;
    padding-right: 32px;
  }

  #draw-date-input-container button {
    position: absolute;
    right: 6px;
    flex-shrink: 0;
    height: 20px;
    width: 20px;
  }

  #draw-date-input-container button img {
    height: 10px;
    width: 10px;
  }

  #chart-container {
    width: 100%;
    height: 100%;
    background-color: rgba(200, 200, 200);
    overflow: hidden;
  }
</style>