const OCTET = /^(0|[1-9]\d{0,2})$/;
const PREFIX = /^\d+$/;
const LABEL = /^[\p{L}\p{N}_-]+$/u;

function ipv4(value: string): boolean {
  const parts = value.split('.');
  return parts.length === 4 && parts.every((part) => OCTET.test(part) && Number(part) <= 255);
}

function ipv6(value: string): boolean {
  if (!value.includes(':') || /[[\]%/\s]/.test(value)) {
    return false;
  }
  try {
    return new URL(`http://[${value}]/`).hostname.length > 2;
  } catch {
    return false;
  }
}

export function validNetwork(text: string): boolean {
  const value = text.trim();
  const slash = value.indexOf('/');
  const address = slash >= 0 ? value.slice(0, slash) : value;
  const prefix = slash >= 0 ? value.slice(slash + 1) : null;
  const bits = ipv4(address) ? 32 : ipv6(address) ? 128 : 0;
  if (bits === 0) {
    return false;
  }
  if (prefix === null) {
    return true;
  }
  return PREFIX.test(prefix) && Number(prefix) <= bits;
}

export function validDomain(text: string): boolean {
  let name = text.trim().toLowerCase();
  if (name.startsWith('*.')) {
    name = name.slice(2);
  }
  if (name.endsWith('.')) {
    name = name.slice(0, -1);
  }
  return (
    name.length > 0 &&
    name.length <= 253 &&
    name
      .split('.')
      .every(
        (label) =>
          label.length > 0 &&
          label.length <= 63 &&
          !label.startsWith('-') &&
          !label.endsWith('-') &&
          LABEL.test(label),
      )
  );
}

export function normalizeDomain(text: string): string {
  return text.trim().toLowerCase();
}
