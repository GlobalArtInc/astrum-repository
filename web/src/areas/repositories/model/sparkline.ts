export function sparklinePath(values: readonly number[], width: number, height: number): string {
  if (values.length < 2) {
    return '';
  }
  const low = Math.min(...values);
  const span = Math.max(...values) - low;
  const step = width / (values.length - 1);
  return values
    .map((value, index) => {
      const y = span === 0 ? height / 2 : height - ((value - low) / span) * height;
      return `${index === 0 ? 'M' : 'L'}${(index * step).toFixed(2)},${y.toFixed(2)}`;
    })
    .join(' ');
}
