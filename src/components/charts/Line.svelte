<script lang="ts">
  import { onMount } from "svelte";
  import type { Chart } from "chart.js";

  import { t, lang } from "$lib/i18n";
  import type { Transaction } from "$lib/types";
  import { handleDate } from "$lib/functions";

  let {
    transactionsData,
    searchedDate,
  }: {
    transactionsData: Transaction[];
    searchedDate: string;
  } = $props();

  let chartCanvas: HTMLCanvasElement | null = null;
  let chart: Chart;
  let displayTransactions = $state<Record<string, number>>({});
  const finnishMonthAbbrevs = ["Tammi", "Helmi", "Maalis", "Huhti", "Touko", "Kesä", "Heinä", "Elo", "Syys", "Loka", "Marras", "Joulu"] as const;
  let displayDate = $derived(handleDate(searchedDate));

  onMount(async () => {
    const { Chart, registerables } = await import('chart.js');
    Chart.register(...registerables);

    if (chartCanvas) {
      chart = new Chart(chartCanvas, {
        type: 'line',
        options: {
          scales: {
            x: {
              ticks: {
                color: 'black'
              }
            },
            y: {
              ticks: {
                color: 'black'
              }
            },
          },
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
              borderColor: "rgba(255, 70, 70, 1)",
              backgroundColor: "rgba(255, 70, 70, 0.5)"
            }
          ]
        }
      });
    }

    drawChart();
  });

  $effect(() => {
    if ($lang !== null && chart) {
      chart.data.datasets[0].label = $t["chart.amount.total"] as string;

      const sortedKeys = Object.keys(displayTransactions).sort();
      chart.data.labels = getLabelsFromKeys(sortedKeys);

      if (!chart.options.plugins?.title) return;
      chart.options.plugins.title.text = handleDate(searchedDate);

      chart.update();
    }
  });

  const getLabelsFromKeys = (keys: string[]) => {
    const monthNames = $t["calendar.monthnames"] as string[];
    return keys.map((key) => {
      const monthIdx = parseInt(key.slice(5, 7)) - 1;
      const monthName = monthNames[monthIdx];
      return $lang === "en"
        ? monthName.slice(0, 3) + ` (${monthIdx + 1})`
        : finnishMonthAbbrevs[monthIdx] + ` (${monthIdx + 1})`;
    });
  };

  const drawChart = () => {
    transactionsData.forEach((t) => {
      const monthKey = `${t.date.slice(0, 7)}`;
      displayTransactions[monthKey] = (displayTransactions[monthKey] || 0) + (t._type === "income" ? t.amount : -t.amount);
    });

    const sortedKeys = Object.keys(displayTransactions).sort();

    chart.data.datasets[0].data = sortedKeys.map((key) => displayTransactions[key]);
    chart.data.labels = getLabelsFromKeys(sortedKeys);
    chart.update();
  };

</script>

<canvas bind:this={chartCanvas}></canvas>

<style>
  canvas {
    filter: drop-shadow(0 4px 8px rgba(0, 0, 0, 0.2));
  }
</style>