<script lang="ts">
  import { onMount, tick } from "svelte";
  import type { Chart } from "chart.js";

  import { getTransactions, transactions, expenseCategories, incomeCategories } from "$lib/transactions";
  import { user } from "$lib/user";
  import { sendAlert } from "$lib/alert";
  import { t, lang } from "$lib/i18n";

  let chartCanvas = $state<HTMLCanvasElement | null>(null);
  let chart: Chart;

  let displayTransactions = $state<Record<string, number>>({});
  let combinedCategories = [... expenseCategories, ...incomeCategories];
  let dateToDraw = $state<string>('');

  const updateLegendLabels = () => {
    return (chart: Chart) => {
      return [
        {
          text: $t["transaction-table.type.income"] as string,
          fillStyle: "rgba(115, 200, 115, 0.2)",
          strokeStyle: "#73c873",
          lineWidth: 2,
          hidden: false,
          index: 0,
        },
        {
          text: $t["transaction-table.type.expense"] as string,
          fillStyle: "rgba(195, 70, 70, 0.2)",
          strokeStyle: "#c34646",
          lineWidth: 2,
          hidden: false,
          index: 1,
        }
      ];
    };
  };

  onMount(async () => {
    const { Chart, registerables } = await import('chart.js');
    Chart.register(...registerables);
    await tick();

    if (chartCanvas) {
      chart = new Chart(chartCanvas, {
        type: 'bar',
        options: {
          plugins: {
            legend: {
              labels: {
                generateLabels: updateLegendLabels()
              }
            }
          }
        },
        data: {
          labels: [],
          datasets: [
            {
              data: [],
              label: $t["chart.amount.total"] as string,
              backgroundColor: '',
              borderColor: '',
              borderWidth: 2,
            }
          ]
        }
      });
    }
  });

  $effect(() => {
    if ($lang !== null && chart) {
      chart.data.labels = Object.entries(displayTransactions).map(([key, _]) => {
        const [category, _type] = key.split("-");
        const item = combinedCategories.find(item => item.value === category);
        return item ? ($t[item.parent] as Array<Record<string, string>>)[item.index][item.key] : 'Unknown';
      });

      chart.data.datasets[0].label = $t["chart.amount.total"] as string;

      if (!chart.options.plugins?.legend?.labels?.generateLabels) return;
      chart.options.plugins.legend.labels.generateLabels = updateLegendLabels();

      chart.update();
    }
  });

  const drawChart = async () => {
    if (!$user) return;

    displayTransactions = {};

    if (dateToDraw.trim().length > 0) {
      const dateParts = dateToDraw.split('-');
      if (!/^\d{4}$/.test(dateParts[0])) { sendAlert("alert.transactions-table.date-jump.invalid-year", true, false); return; }
      if (!/^0*([1-9]|1[0-2])$/.test(dateParts[1])) { sendAlert("alert.transactions-table.date-jump.invalid-month", true, false); return; }

      await getTransactions($user.id, dateToDraw, $user.name);
    } else {
      const yearMonth = ((d) => `${String(d.getFullYear())}-${String(d.getMonth() + 1).padStart(2, '0')}`)(new Date());
      await getTransactions($user.id, "2026-04", $user.name);
    }

    $transactions.forEach((t) => {
      const key = `${t.category}-${t._type}`;
      displayTransactions[key] = (displayTransactions[key] || 0) + t.amount;
    });

    if (chart) {
      chart.data.labels = Object.entries(displayTransactions).map(([key, _]) => {
        const [category, _type] = key.split("-");
        const item = combinedCategories.find(item => item.value === category);
        return item ? ($t[item.parent] as Array<Record<string, string>>)[item.index][item.key] : 'Unknown';
      });
      chart.data.datasets[0].backgroundColor = Object.entries(displayTransactions).map(([key, _]) => {
        const [category, _type] = key.split("-");
        return _type === "expense" ? "rgba(195, 70, 70, 0.2)" : "rgba(115, 200, 115, 0.2)";
      });
      chart.data.datasets[0].borderColor = Object.entries(displayTransactions).map(([key, _]) => {
        const [category, _type] = key.split("-");
        return _type === "expense" ? "#c34646" : "#73c873";
      });
      chart.data.datasets[0].data = Object.values(displayTransactions);
      chart.update();
    }
  };
</script>

<div id="charts-main-container" class="vertical-flex-container">
  <div id="charts-toolbar" class="horizontal-flex-container">
    <div id="draw-date-input-container" class="horizontal-flex-container">
      <input class="primary-input" placeholder={$t["placeholder.year-month"] as string} bind:value={dateToDraw} />
      <button class="transparent-button-highlight" onclick={() => dateToDraw = ''}><img src="/close-x.svg" alt="Close" /></button>
    </div>
    <button class="primary-button" onclick={() => drawChart()}>Render</button>
  </div>
  <div id="chart-container" class="vertical-flex-container">
    <canvas id="chart-canvas" bind:this={chartCanvas}></canvas>
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
    justify-content: flex-end;
    background-color: rgba(200, 200, 200);
  }

  .primary-input {
    color: #f6f6f6;
  }
</style>