<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import uPlot from 'uplot';
  import 'uplot/dist/uPlot.min.css';
  import { colors } from '../theme';

  export let dates: string[] = [];
  export let values: number[] = [];
  export let accent: string = '#7dd3fc';
  export let height = 240;

  let el: HTMLDivElement;
  let plot: uPlot | undefined;

  $: if (plot) plot.setData([dates.map((_, i) => i), values]);

  onMount(() => {
    const opts: uPlot.Options = {
      width: el.clientWidth,
      height,
      scales: { x: { time: false } },
      axes: [
        {
          stroke: colors.axis,
          grid: { stroke: colors.grid },
          values: (_u, splits) => splits.map((s) => dates[s] ?? '')
        },
        { stroke: colors.axis, grid: { stroke: colors.grid } }
      ],
      series: [
        { value: (_u, i) => dates[i] ?? '' },
        {
          label: 'count',
          stroke: accent,
          width: 2,
          fill: accent + '22'
        }
      ]
    };
    plot = new uPlot(opts, [dates.map((_, i) => i), values], el);
    const ro = new ResizeObserver(() =>
      plot?.setSize({ width: el.clientWidth, height })
    );
    ro.observe(el);
    return () => ro.disconnect();
  });

  onDestroy(() => plot?.destroy());
</script>

<div bind:this={el} class="w-full"></div>
