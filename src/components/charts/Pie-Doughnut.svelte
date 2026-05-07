<script lang="ts">
  import type { Chart } from 'chart.js';
  import { onMount } from 'svelte';

  import { t, lang } from '$lib/i18n';
  import type { Transaction } from '$lib/types';
  import { expenseCategories, incomeCategories } from '$lib/transactions';
  import { handleDate } from '$lib/functions';

  let {
    transactionsData,
    type,
    searchedDate,
  }: {
    transactionsData: Transaction[];
    type?: "pie" | "doughnut";
    searchedDate: string;
  } = $props();

  let chartCanvas: HTMLCanvasElement | null = null;
  let chart: Chart;
  let displayTransactions = $state<Record<string, number>>({});
  const combinedCategories = [...expenseCategories, ...incomeCategories];
  let displayDate = $derived(handleDate(searchedDate));

  onMount(async () => {
    const { Chart, registerables } = await import('chart.js');
    Chart.register(...registerables);
    if (chartCanvas) {
      chart = new Chart(chartCanvas, {
        type: type ?? "pie",
        options: {
          plugins: {
            legend: {
              labels: {
                color: 'black',
                font: {
                  weight: 'bold'
                }
              }
            },
            title: {
              display: true,
              text: displayDate,
              color: 'black',
              font: {
                weight: 'bold',
                size: 18
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
              backgroundColor: ["#4F46E5", "#F59E0B", "#10B981", "#EF4444", "#8B5CF6", "#06B6D4", "#84CC16", "#F97316", "#A855F7", "#0EA5E9", "#22C55E", "#EAB308", "#EC4899", "#3B82F6", "#14B8A6"],
              borderWidth: 3,
              borderColor: "rgba(180, 180, 180)",
              hoverBorderColor: "rgba(255, 70, 70, 1)"
            }
          ]
        }
      });
    }

    drawChart();
  });

  $effect(() => {
    if ($lang !== null && chart) {
      chart.data.labels = Object.entries(displayTransactions).map(([key, amount]) => {
        const item = combinedCategories.find(c => c.value === key);
        return item ? ($t[item.parent] as Array<Record<string, string>>)[item.index][item.key] : "Unknown";
      });

      chart.data.datasets[0].label = $t["chart.amount.total"] as string;

      if (!chart.options.plugins?.title) return;
      chart.options.plugins.title.text = handleDate(searchedDate);

      chart.update();
    }
  });

  const drawChart = () => {
    transactionsData.forEach((t) => {
      const key = t.category;
      displayTransactions[key] = (displayTransactions[key] || 0) + t.amount;
    });

    chart.data.labels = Object.entries(displayTransactions).map(([key, amount]) => {
      const item = combinedCategories.find(c => c.value === key);
      return item ? ($t[item.parent] as Array<Record<string, string>>)[item.index][item.key] : "Unknown";
    });

    chart.data.datasets[0].data = Object.values(displayTransactions);

    chart.update();
  };
</script>

<canvas bind:this={chartCanvas}></canvas>

<style>
  canvas {
    filter: drop-shadow(0 4px 8px rgba(0, 0, 0, 0.2));
  }
</style>