<script lang="ts">
  import { onMount } from "svelte";
  import type { Chart } from "chart.js";

  import { expenseCategories, incomeCategories } from "$lib/transactions";
  import { t, lang } from "$lib/i18n";
  import type { Transaction } from "$lib/types";

  let {
    transactionsData,
  }: {
    transactionsData: Transaction[];
  } = $props();

  let chartCanvas = $state<HTMLCanvasElement | null>(null);
  let chart: Chart;
  let combinedCategories = [...expenseCategories, ...incomeCategories];
  let displayTransactions = $state<Record<string, number>>({});

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

      drawChart();
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
    if (chart) {
      transactionsData.forEach((t) => {
        const key = `${t.category}-${t._type}`;
        displayTransactions[key] = (displayTransactions[key] || 0) + t.amount;
      });

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

<canvas id="chart-canvas" bind:this={chartCanvas}></canvas>