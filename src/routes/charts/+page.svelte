<script lang="ts">
  import { fly } from "svelte/transition";
  import { cubicInOut } from "svelte/easing";

  import { getTransactions, transactions } from "$lib/transactions";
  import { user } from "$lib/user";
  import { sendAlert } from "$lib/alert";
  import { t } from "$lib/i18n";

  import BarChart from "../../components/charts/Bar.svelte";

  let displayTransactions = $state<Record<string, number>>({});
  let dateToDraw = $state<string>('');
  let selectChartValue = $state<string>('1');
  let currentChart = $state<"bar" | null>(null);

  /***********************************************************************************************************************************
  |
  | Context, Helper & Wrapper functions
  |
  \***********************************************************************************************************************************/
  const drawChart = async () => {
    await populateTransactions();

    switch(parseInt(selectChartValue)) {
      case 1: currentChart = "bar"; break;
    }
  };

  /***********************************************************************************************************************************/

  const populateTransactions = async () => {
    if (!$user) return;

    displayTransactions = {};

    if (dateToDraw.trim().length > 0) {
      const dateParts = dateToDraw.split('-');
      if (!/^\d{4}$/.test(dateParts[0])) { sendAlert("alert.invalid-year", true, false); currentChart = null; return; }
      if (!/^0*([1-9]|1[0-2])$/.test(dateParts[1])) { sendAlert("alert.invalid-month", true, false); currentChart = null; return; }

      await getTransactions($user.id, dateToDraw, $user.name);
    } else {
      const yearMonth = ((d) => `${String(d.getFullYear())}-${String(d.getMonth() + 1).padStart(2, '0')}`)(new Date());
      await getTransactions($user.id, "2026-04", $user.name);
    }

    $transactions.forEach((t) => {
      const key = `${t.category}-${t._type}`;
      displayTransactions[key] = (displayTransactions[key] || 0) + t.amount;
    });
  };
</script>

<div id="charts-main-container" class="vertical-flex-container">
  <div id="charts-toolbar" class="horizontal-flex-container">
    <div id="draw-date-input-container" class="horizontal-flex-container">
      <input class="primary-input" placeholder={$t["placeholder.year-month"] as string} bind:value={dateToDraw} />
      <button class="transparent-button-highlight" onclick={() => dateToDraw = ''}><img src="/close-x.svg" alt="Close" /></button>
    </div>
    <select class="primary-input" bind:value={selectChartValue}>
      <option value=1>{$t["chart.bar.name"]}</option>
    </select>
    <button class="primary-button" onclick={() => currentChart = null}>{$t["clear.button"]}</button>
    <button class="primary-button" onclick={() => drawChart()}>{$t["chart.button.draw"]}</button>
  </div>
  <div id="chart-container">
    {#if currentChart === "bar"}
      <div class="chart-wrapper" in:fly={{ x: 1 * 1000, duration: 800, easing: cubicInOut }} out:fly={{ x: 1 * -1000, duration: 800, easing: cubicInOut }}>
        <BarChart displayTransactions={displayTransactions} />
      </div>
    {/if}
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