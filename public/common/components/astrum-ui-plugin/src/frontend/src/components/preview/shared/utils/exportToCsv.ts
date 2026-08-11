

function escapeField(value: unknown): string {
  const str = value == null ? '' : String(value);
  if (str.includes(',') || str.includes('"') || str.includes('\n') || str.includes('\r') || str.includes('\t')) {
    return `"${str.replace(/"/g, '""')}"`;
  }
  return str;
}

export function exportToCsv(
  data: Record<string, unknown>[],
  filename: string,
  columns: string[],
): void {
  const header = columns.join(',');
  const rows = data.map((row) => columns.map((col) => escapeField(row[col])).join(','));
  const csv = [header, ...rows].join('\n');

  // BOM ensures Excel opens UTF-8 CSV correctly without character encoding issues
  const blob = new Blob(['\uFEFF' + csv], { type: 'text/csv;charset=utf-8;' });
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement('a');
  anchor.href = url;
  anchor.download = filename;
  document.body.appendChild(anchor);
  anchor.click();
  document.body.removeChild(anchor);
  URL.revokeObjectURL(url);
}
