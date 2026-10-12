export async function copyText(text: string): Promise<void> {
  if (navigator.clipboard) {
    try {
      await navigator.clipboard.writeText(text);
      return;
    } catch {}
  }
  const area = document.createElement('textarea');
  area.value = text;
  area.setAttribute('readonly', '');
  area.style.position = 'fixed';
  area.style.opacity = '0';
  document.body.appendChild(area);
  area.select();
  try {
    if (!document.execCommand('copy')) {
      throw new Error('copy refused');
    }
  } finally {
    area.remove();
  }
}
