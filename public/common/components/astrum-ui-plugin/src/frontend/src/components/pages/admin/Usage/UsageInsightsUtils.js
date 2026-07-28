

export const KEY_EGRESS = 'egress';
export const KEY_STORAGE = 'storage';

/**
 * Check if an error is a permission error (403 Forbidden)
 * @param {Error} error - The error object from Axios
 * @returns {boolean} True if the error is a 403 Forbidden response
 */
export const isPermissionError = (error) => {
  return error?.response?.status === 403;
};

export function getScaleFactor(value) {
  const order = Math.floor(Math.log10(value) / 3) * 3;
  if(order > 0) {
    return Math.pow(10, order);
  }
  return 1;
}

export function getMaxValue(data) {
  const max = data?.reduce((max, item) => {
    const egress = Number(item[KEY_EGRESS]) || 0;
    const storage = Number(item[KEY_STORAGE]) || 0;
    return Math.max(max, egress + storage);
  }, 0) || 0;
  const scale = getScaleFactor(max);

  return Math.ceil(max / scale / 0.1) * scale * 0.1;
}

export function getValueTicks(maxValue, tickCount) {
  if (maxValue === 0) return []

  return [...Array(tickCount).keys()].map(i => (i + 1) * (maxValue / tickCount));
}

export function getMetricDateTicks(data) {
  if (data.length === 0) return [];

  let dateTicks = data.map(d => d.metricDate);
  const lastDate = dateTicks[dateTicks.length - 1];
  const extraTick = dateTicks.length % 2;
  const maxTicksAllowed = 4 + extraTick;

  if (dateTicks.length > 8) {
    const step = dateTicks.length / (maxTicksAllowed - 1)

    dateTicks = [...Array(maxTicksAllowed).keys()].map(i => dateTicks[Math.floor(i * step)] || lastDate);
  }

  return dateTicks
}

export function formatAsShortDate(iso) {
  const [y, m, d] = iso.toString().split('-').map(Number);
  const date = new Date(y, m - 1, d);

  // Let the browser handle locale-specific formatting
  return new Intl.DateTimeFormat(undefined, {
    month: "short",
    day: "numeric"
  }).format(date);
}

export function formatAsISODate(date) {
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, '0');
  const day = String(date.getDate()).padStart(2, '0');

  return `${year}-${month}-${day}`
}

export function getDateRange(baseDate) {
  const dateFrom = new Date(baseDate.getFullYear(), baseDate.getMonth(), 1);
  const dateTo = new Date(baseDate.getFullYear(), baseDate.getMonth() + 1, 0);

  return {
    dateFromAsDate: dateFrom,
    dateToAsDate: dateTo,
    dateFrom: formatAsISODate(dateFrom),
    dateTo: formatAsISODate(dateTo)
  }
}

export function getMonthOptions(monthsBack = 12) {
  const options = [];
  const currentDate = new Date();
  currentDate.setDate(1);

  for (let i = 0; i < monthsBack; i++) {
    const value = getDateRange(currentDate);
    const dateFromLabel = formatAsShortDate(value.dateFrom, true);
    const dateToLabel = formatAsShortDate(value.dateTo, true);
    const key = `${value.dateFrom}-${value.dateTo}`;
    const label = `${dateFromLabel} - ${dateToLabel}`;

    options.push({key, value, label});
    currentDate.setMonth(currentDate.getMonth() - 1);
  }

  return options;
}
