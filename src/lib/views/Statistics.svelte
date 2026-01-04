<script>
  import { onMount } from 'svelte';
  import * as echarts from 'echarts';
  import Card from '../components/ui/Card.svelte';
  import StatCard from '../components/StatCard.svelte';
  import Select from '../components/ui/Select.svelte';
  import Button from '../components/ui/Button.svelte';
  import IconBadge from '../components/ui/IconBadge.svelte';
  import { calculateStatistics } from '../stores/procedureStore.js';
  import { exportStatisticsToExcel } from '../utils/excelExport.js';
  import { formatNumberIT, formatPercentage } from '../utils/statistics.js';
  import { FILTER_PERIODS } from '../constants.js';

  let stats = null;
  let loading = true;
  let selectedPeriod = 'all';
  let pieChartElement;
  let barChartElement;
  let pieChart;
  let barChart;

  async function loadStats() {
    loading = true;
    const filters = selectedPeriod !== 'all' ? { period: selectedPeriod } : null;
    const result = await calculateStatistics(filters);

    if (result.success) {
      stats = result.data;
      updateCharts();
    }
    loading = false;
  }

  function updateCharts() {
    if (!stats) return;

    // Pie Chart: Distribuzione tipo valvola
    if (pieChartElement) {
      if (!pieChart) {
        pieChart = echarts.init(pieChartElement);
      }

      const pieOption = {
        title: {
          text: 'Distribuzione Tipo Valvola',
          left: 'center',
          textStyle: {
            fontSize: 16,
            fontWeight: 'normal',
            color: '#212121',
          },
        },
        tooltip: {
          trigger: 'item',
          formatter: '{b}: {c} ({d}%)',
        },
        legend: {
          orient: 'vertical',
          left: 'left',
          bottom: 20,
        },
        color: ['#2196F3', '#4CAF50'],
        series: [
          {
            type: 'pie',
            radius: '60%',
            center: ['50%', '55%'],
            data: [
              { value: stats.balloon_expandable_count, name: 'Balloon Expandable' },
              { value: stats.self_expandable_count, name: 'Self Expandable' },
            ],
            emphasis: {
              itemStyle: {
                shadowBlur: 10,
                shadowOffsetX: 0,
                shadowColor: 'rgba(0, 0, 0, 0.5)',
              },
            },
            label: {
              show: true,
              formatter: '{b}\n{c} ({d}%)',
            },
          },
        ],
      };

      pieChart.setOption(pieOption);
    }

    // Bar Chart: Top 5 modelli valvole
    if (barChartElement && stats.top_valve_models.length > 0) {
      if (!barChart) {
        barChart = echarts.init(barChartElement);
      }

      const labels = stats.top_valve_models.map((item) => item[0]);
      const values = stats.top_valve_models.map((item) => item[1]);

      const barOption = {
        title: {
          text: 'Top 5 Modelli Valvole',
          left: 'center',
          textStyle: {
            fontSize: 16,
            fontWeight: 'normal',
            color: '#212121',
          },
        },
        tooltip: {
          trigger: 'axis',
          axisPointer: {
            type: 'shadow',
          },
          formatter: '{b}: {c} procedure',
        },
        grid: {
          left: '3%',
          right: '4%',
          bottom: '3%',
          top: '15%',
          containLabel: true,
        },
        xAxis: {
          type: 'category',
          data: labels,
          axisLabel: {
            interval: 0,
            rotate: 30,
            fontSize: 10,
          },
        },
        yAxis: {
          type: 'value',
          minInterval: 1,
        },
        series: [
          {
            type: 'bar',
            data: values,
            itemStyle: {
              color: '#2196F3',
            },
            emphasis: {
              itemStyle: {
                color: '#1976D2',
              },
            },
            barWidth: '60%',
          },
        ],
      };

      barChart.setOption(barOption);
    }
  }

  async function handleExport() {
    if (!stats) return;

    const result = await exportStatisticsToExcel(stats);
    if (result.success) {
      alert('Statistiche esportate con successo!');
    } else if (!result.cancelled) {
      alert('Errore durante l\'export: ' + result.error);
    }
  }

  onMount(async () => {
    await loadStats();

    // Resize charts on window resize
    const handleResize = () => {
      pieChart?.resize();
      barChart?.resize();
    };

    window.addEventListener('resize', handleResize);

    return () => {
      window.removeEventListener('resize', handleResize);
      pieChart?.dispose();
      barChart?.dispose();
    };
  });

  // Ricarica quando cambia periodo
  $: if (selectedPeriod !== undefined) {
    loadStats();
  }
</script>

<div class="max-w-7xl mx-auto">
  <!-- Header -->
  <div class="flex items-center justify-between mb-6">
    <div>
      <h2 class="text-2xl font-bold text-textPrimary">Statistiche TAVI</h2>
      <p class="text-sm text-textSecondary mt-1">Analisi aggregata delle procedure</p>
    </div>

    <div class="flex gap-3 items-center">
      <Select
        bind:value={selectedPeriod}
        options={FILTER_PERIODS}
        placeholder="Periodo"
      />

      <Button variant="secondary" on:click={handleExport} disabled={!stats}>
        <IconBadge icon="download" size="sm" tone="secondary" class="mr-2" /> Esporta Excel
      </Button>
    </div>
  </div>

  {#if loading}
    <div class="text-center py-12">
      <div class="inline-block animate-spin rounded-full h-12 w-12 border-b-2 border-primary"></div>
      <p class="mt-4 text-textSecondary">Caricamento statistiche...</p>
    </div>
  {:else if stats}
    <!-- Statistiche Principali -->
    <div class="grid grid-cols-1 md:grid-cols-4 gap-4 mb-8">
      <StatCard
        icon="stats"
        title="Totale Procedure"
        value={stats.total_procedures.toString()}
        color="primary"
      />

      <StatCard
        icon="timer"
        title="Durata Media"
        value="{Math.round(stats.average_duration_minutes)} min"
        color="secondary"
      />

      <StatCard
        icon="balloon"
        title="Pre-dilatazione"
        value={formatPercentage(stats.pre_dilatazione_percentage)}
        color="success"
      />

      <StatCard
        icon="balloon"
        title="Post-dilatazione"
        value={formatPercentage(stats.post_dilatazione_percentage)}
        color="success"
      />
    </div>

    <!-- Grafici -->
    <div class="grid grid-cols-1 lg:grid-cols-2 gap-6 mb-8">
      <!-- Pie Chart -->
      <Card padding="md">
        <div bind:this={pieChartElement} class="w-full h-80"></div>
      </Card>

      <!-- Bar Chart -->
      <Card padding="md">
        <div bind:this={barChartElement} class="w-full h-80"></div>
      </Card>
    </div>

    <!-- Parametri Emodinamici Medi -->
    <Card padding="lg">
      <h3 class="text-lg font-semibold text-textPrimary mb-4 flex items-center gap-2">
        <IconBadge icon="heart" size="sm" tone="secondary" /> Parametri Emodinamici Medi
      </h3>

      <div class="grid grid-cols-2 md:grid-cols-5 gap-6">
        <!-- FE -->
        <div class="text-center">
          <div class="text-3xl font-bold text-primary mb-2">
            {stats.average_fe !== null ? formatNumberIT(stats.average_fe, 1) : '-'}
          </div>
          <div class="text-sm text-textSecondary font-medium">
            FE (%)
          </div>
        </div>

        <!-- Vmax -->
        <div class="text-center">
          <div class="text-3xl font-bold text-primary mb-2">
            {stats.average_vmax !== null ? formatNumberIT(stats.average_vmax, 2) : '-'}
          </div>
          <div class="text-sm text-textSecondary font-medium">
            Vmax (m/s)
          </div>
        </div>

        <!-- Gmax -->
        <div class="text-center">
          <div class="text-3xl font-bold text-primary mb-2">
            {stats.average_gmax !== null ? formatNumberIT(stats.average_gmax, 1) : '-'}
          </div>
          <div class="text-sm text-textSecondary font-medium">
            Gmax (mmHg)
          </div>
        </div>

        <!-- Gmed -->
        <div class="text-center">
          <div class="text-3xl font-bold text-primary mb-2">
            {stats.average_gmed !== null ? formatNumberIT(stats.average_gmed, 1) : '-'}
          </div>
          <div class="text-sm text-textSecondary font-medium">
            Gmed (mmHg)
          </div>
        </div>

        <!-- AVA -->
        <div class="text-center">
          <div class="text-3xl font-bold text-primary mb-2">
            {stats.average_ava !== null ? formatNumberIT(stats.average_ava, 2) : '-'}
          </div>
          <div class="text-sm text-textSecondary font-medium">
            AVA (cm²)
          </div>
        </div>
      </div>
    </Card>
  {:else}
    <Card padding="lg">
      <div class="text-center py-12">
        <IconBadge icon="stats" size="lg" class="mx-auto mb-4" />
        <h3 class="text-xl font-semibold text-textPrimary mb-2">Nessun dato disponibile</h3>
        <p class="text-textSecondary">
          Aggiungi alcune procedure per visualizzare le statistiche
        </p>
      </div>
    </Card>
  {/if}
</div>
