<script lang="ts">
  import { fly } from "svelte/transition";
  import { cubicInOut } from "svelte/easing";

  import { getTransactions, transactions, getTransactionsByYear } from "$lib/transactions";
  import { user } from "$lib/user";
  import { sendAlert } from "$lib/alert";
  import { t } from "$lib/i18n";
  import type { Transaction } from "$lib/types";

  import BarChart from "../../components/charts/Bar.svelte";
  import LineChart from "../../components/charts/Line.svelte";
  import PieChart from "../../components/charts/Pie-Doughnut.svelte";
  import DoughnutChart from "../../components/charts/Pie-Doughnut.svelte";

  let transactionsData = $state<Transaction[]>([])
  let chartKey = $state(0); // Used in making sure a new chart is always generated.
  let currentChart = $state<"bar" | "line" | "pie" | "doughnut" | null>(null);

  let dateToDraw = $state<string>('');
  let selectChartValue = $state<number>(1);
  let isYearly = $state<boolean>(false);
  let searchedDate = $state<string>('');

  /***********************************************************************************************************************************
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
    if (!$user) return;

    currentChart = null;
    chartKey++;

    if (dateToDraw.trim().length > 0) {
      const dateParts = dateToDraw.split('-');
      if (!/^\d{4}$/.test(dateParts[0])) { sendAlert("alert.invalid-year", true, false); currentChart = null; return; }
      if (!/^0*([1-9]|1[0-2])$/.test(dateParts[1]) && !isYearly) { sendAlert("alert.invalid-month", true, false); currentChart = null; return; }

      if (isYearly) {
        const getTransactions = await getTransactionsByYear($user.id, dateToDraw, $user.name);
        if (getTransactions.success) {
          transactionsData = getTransactions.data;
        }
      } else {
        await getTransactions($user.id, dateToDraw, $user.name);
        transactionsData = $transactions;
      }

      searchedDate = dateToDraw;
    } else {
      if (isYearly) {
        const year = ((d) => `${String(d.getFullYear())}`)(new Date());
        const getTransactions = await getTransactionsByYear($user.id, year, $user.name);
        if (getTransactions.success) {
          transactionsData = getTransactions.data;
        }
        searchedDate = year;
      } else {
        const yearMonth = ((d) => `${String(d.getFullYear())}-${String(d.getMonth() + 1).padStart(2, '0')}`)(new Date());
        await getTransactions($user.id, "2026-04", $user.name);
        transactionsData = $transactions;
        searchedDate = yearMonth;
      }
    }

    if (transactionsData.length <= 0) { sendAlert("alert.no-transaction-data", true, false); return; };

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
  <div id="charts-toolbar" class="horizontal-flex-container">
    <div id="is-yearly-input-container" class="horizontal-flex-container">
      <input type="checkbox" bind:checked={isYearly} />
      <span>{$t["chart.full-year-checkbox"]}</span>
    </div>
    <div id="draw-date-input-container" class="horizontal-flex-container" title={$t["charts.date-input.title"] as string}>
      <input class="primary-input" placeholder={!isYearly ? $t["placeholder.year-month"] as string : $t["placeholder.year-month"].slice(0, 4) as string} bind:value={dateToDraw} />
      <button class="transparent-button-highlight" onclick={() => dateToDraw = ''}><img src="/close-x.svg" alt="Close" /></button>
    </div>
    <select class="primary-input" bind:value={selectChartValue}>
      {#each $t["chart.chart-names"] as option, i (i)}
        <option value={i+1}>{option}</option>
      {/each}
    </select>
    <button class="primary-button" onclick={() => handleClear()}>{$t["clear.button"]}</button>
    <button class="primary-button" onclick={() => populateTransactions()}>{$t["chart.button.draw"]}</button>
  </div>
  <div id="chart-container">
    {#key chartKey}
      {#if currentChart === "bar"}
        <div class="chart-wrapper" in:fly={{ x: 1 * 1000, duration: 800, easing: cubicInOut }} out:fly={{ x: 1 * -1000, duration: 800, easing: cubicInOut }}>
          <BarChart transactionsData={transactionsData} {searchedDate} />
        </div>
      {:else if currentChart === "line"}
        <div class="chart-wrapper" in:fly={{ x: 1 * 1000, duration: 800, easing: cubicInOut }} out:fly={{ x: 1 * -1000, duration: 800, easing: cubicInOut }}>
          <LineChart transactionsData={transactionsData} {searchedDate} />
        </div>
      {:else if currentChart === "pie"}
        <div class="chart-wrapper" in:fly={{ x: 1 * 1000, duration: 800, easing: cubicInOut }} out:fly={{ x: 1 * -1000, duration: 800, easing: cubicInOut }}>
          <PieChart transactionsData={transactionsData} type="pie" {searchedDate} />
        </div>
      {:else if currentChart === "doughnut"}
        <div class="chart-wrapper" in:fly={{ x: 1 * 1000, duration: 800, easing: cubicInOut }} out:fly={{ x: 1 * -1000, duration: 800, easing: cubicInOut }}>
          <DoughnutChart transactionsData={transactionsData} type="doughnut" {searchedDate} />
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

  #charts-toolbar {
    justify-content: flex-start;
    height: 49px;
    width: 100%;
    padding: 8px;
    gap: 12px;
    border-bottom: 1px solid #333;
  }

  #is-yearly-input-container span, #charts-toolbar select {
    font-size: clamp(0.75rem, 0.9cqw, 1rem);
  }

  #is-yearly-input-container input {
    height: 20px;
    width: 20px;
  }
  #is-yearly-input-container input:hover {
    cursor: pointer;
  }

  #charts-toolbar select {
    max-width: 120px;
  }
  #charts-toolbar select:hover {
    cursor: pointer;
    background: #222;
  }

  #charts-toolbar select option {
    background-color: #0f0f0f;
  }

  #draw-date-input-container {
    position: relative;
    height: 32px;
  }

  #draw-date-input-container input {
    width: 110px;
    padding-right: 32px;
    font-size: clamp(0.75rem, 0.9cqw, 1rem);
  }

  #draw-date-input-container button {
    position: absolute;
    right: 6px;
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

  .primary-input {
    color: #f6f6f6;
  }
</style>