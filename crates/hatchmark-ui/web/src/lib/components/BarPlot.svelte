<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import uPlot from 'uplot';
  import 'uplot/dist/uPlot.min.css';
  import { colors } from '../theme';

  export let labels: string[] = [];
  export let values: number[] = [];
  export let accent = '#7dd3fc';
  export let height = 240;

  let el: HTMLDivElement;
  let plot: uPlot | undefined;

  $: if (plot) plot.setData([labels.map((_, i) => i), values]);

  onMount(() => {
    const bars = (uPlot.paths as any).bars;
    const opts: uPlot.Options = {
      width: el.clientWidth,
      height,
      scales: { x: { time: false } },
      axes: [
        {
          stroke: colors.axis,
          values: (_u, splits) => splits.map((s) => labels[s] ?? '')
        },
        { stroke: colors.axis, grid: { stroke: colors.grid } }
      ],
      series: [
        { value: (_u, i) => labels[i] ?? '' },
        {
          label: 'count',
          stroke: accent,
          fill: accent,
          paths: bars ? bars({ size: [0.7, 60] }) : undefined
        }
      ]
    };
    plot = new uPlot(opts, [labels.map((_, i) => i), values], el);
    const ro = new ResizeObserver(() =>
      plot?.setSize({ width: el.clientWidth, height })
    );
    ro.observe(el);
    return () => ro.disconnect();
  });

  onDestroy(() => plot?.destroy());
</script>

<div bind:this={el} class="w-full"></div>
