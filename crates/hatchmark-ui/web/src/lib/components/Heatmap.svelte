<script lang="ts">
  export let dates: string[] = [];
  export let values: number[] = [];
  export let weeks = 53;
  export let cell = 12;
  export let gap = 3;
  export let accent = '#7dd3fc';

  $: max = Math.max(1, ...values);
  function intensity(v: number) {
    return v === 0 ? 0.08 : 0.15 + 0.85 * (v / max);
  }
  function col(i: number) {
    return Math.floor(i / 7);
  }
  function row(i: number) {
    return i % 7;
  }
</script>

<svg
  class="w-full"
  viewBox={`0 0 ${weeks * (cell + gap)} ${7 * (cell + gap)}`}
  preserveAspectRatio="xMinYMin meet">
  {#each values as v, i}
    <rect
      x={col(i) * (cell + gap)}
      y={row(i) * (cell + gap)}
      width={cell}
      height={cell}
      rx="3"
      ry="3"
      fill={accent}
      fill-opacity={intensity(v)}>
      <title>{dates[i]}: {v}</title>
    </rect>
  {/each}
</svg>
