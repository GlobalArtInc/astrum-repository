import React, { useMemo } from 'react';
import { Box, Flex, Text } from '@radix-ui/themes';
import { BarChart3 } from 'lucide-react';
import { Bar, BarChart, CartesianGrid, Rectangle, ResponsiveContainer, Tooltip, XAxis, YAxis } from 'recharts';
import UIStrings from '../../../../../constants/UIStrings';
import { formatDay, formatLongDay, formatNumber, formatWeekday, isWeekend } from './usageFormat';
import type { DownloadsOfDay } from './usageDashboard.types';

const WEEKDAY_COLOR = 'var(--accent-9)';
const WEEKEND_COLOR = 'var(--accent-7)';
const CHART_HEIGHT = 260;

export interface DownloadsChartProps {
  days: DownloadsOfDay[];
}

interface ChartRow extends DownloadsOfDay {
  weekend: boolean;
  label: string;
  peak: boolean;
}

/**
 * The smallest round number a tenth above the peak, so that the gridlines at zero, half and the top carry whole,
 * round labels.
 */
export function niceMax(peak: number): number {
  if (peak <= 0) {
    return 4;
  }
  const target = peak * 1.1;
  const magnitude = 10 ** Math.floor(Math.log10(target));
  const steps = magnitude < 10 ? [2, 4, 6, 8, 10] : [1, 1.2, 1.6, 2, 2.4, 3, 4, 5, 6, 8, 10];
  const step = steps.find((each) => each * magnitude >= target) ?? 10;
  return Math.round(step * magnitude);
}

/**
 * How many days apart the x-axis labels are: every day of a week, every fifth day of a month, every fifteenth of a
 * quarter; counted back from today so that today is always labelled.
 */
export function labelStep(days: number): number {
  if (days <= 7) {
    return 1;
  }
  return days <= 31 ? 5 : 15;
}

function ChartTooltip({ active, payload }: { active?: boolean; payload?: Array<{ payload: ChartRow }> }) {
  if (!active || !payload?.length) {
    return null;
  }
  const row = payload[0].payload;
  return (
    <div className="as-usage-chart__tooltip">
      <Text as="div" size="1" color="gray">{formatLongDay(row.day)}</Text>
      <Text as="div" size="2" weight="bold">{formatNumber(row.downloads)}</Text>
    </div>
  );
}

interface BarShapeProps {
  x?: number;
  y?: number;
  width?: number;
  height?: number;
  payload?: ChartRow;
}

/**
 * A bar of a day, its top rounded, with the count of the peak day above it.
 */
function DayBar({ x = 0, y = 0, width = 0, height = 0, payload }: BarShapeProps) {
  const fill = payload?.weekend ? WEEKEND_COLOR : WEEKDAY_COLOR;
  const bar = <Rectangle x={x} y={y} width={width} height={height} radius={[3, 3, 0, 0]} fill={fill} />;
  if (!payload?.peak) {
    return bar;
  }
  const text = formatNumber(payload.downloads);
  const labelWidth = text.length * 7 + 12;
  const centre = x + width / 2;
  return (
    <g>
      {bar}
      <g className="as-usage-chart__peak">
        <rect x={centre - labelWidth / 2} y={y - 24} width={labelWidth} height={18} rx={6} />
        <text x={centre} y={y - 11} textAnchor="middle">{text}</text>
      </g>
    </g>
  );
}

/**
 * Downloads per day as bars: weekdays in the accent color, weekends lighter, gridlines at zero, half and the top, the
 * peak day labelled. Each bar shows its day and count on hover.
 */
export function DownloadsChart({ days }: DownloadsChartProps): JSX.Element {
  const strings = UIStrings.USAGE_DASHBOARD.CHART;
  const { rows, max } = useMemo(() => {
    const peak = days.reduce((best, day, index) => (day.downloads > days[best].downloads ? index : best), 0);
    const step = labelStep(days.length);
    const result: ChartRow[] = days.map((day, index) => ({
      ...day,
      weekend: isWeekend(day.day),
      peak: index === peak && day.downloads > 0,
      label: (days.length - 1 - index) % step === 0
        ? (days.length <= 7 ? formatWeekday(day.day) : formatDay(day.day))
        : '',
    }));
    return { rows: result, max: niceMax(days[peak]?.downloads ?? 0) };
  }, [days]);

  const total = days.reduce((sum, day) => sum + day.downloads, 0);
  if (total === 0) {
    return (
      <Flex direction="column" align="center" justify="center" gap="2" className="as-usage-chart as-usage-chart--empty"
        style={{ height: CHART_HEIGHT }}>
        <BarChart3 size={32} strokeWidth={1.5} aria-hidden="true" />
        <Text size="2" weight="medium">{strings.EMPTY}</Text>
        <Text size="1" color="gray" align="center" className="as-usage-chart__hint">{strings.EMPTY_HINT}</Text>
      </Flex>
    );
  }

  return (
    <Box className="as-usage-chart" style={{ height: CHART_HEIGHT }} role="img" aria-label={strings.LABEL}>
      <ResponsiveContainer width="100%" height="100%">
        <BarChart data={rows} margin={{ top: 28, right: 4, bottom: 0, left: 0 }}
          barCategoryGap={days.length > 60 ? 1 : days.length > 14 ? 3 : 10}>
          <CartesianGrid vertical={false} stroke="var(--gray-a4)" />
          <YAxis
            domain={[0, max]}
            ticks={[0, max / 2, max]}
            tickFormatter={(value: number) => formatNumber(value)}
            axisLine={false}
            tickLine={false}
            width={48}
            tick={{ fill: 'var(--gray-11)', fontSize: 11 }}
            allowDecimals={false}
          />
          <XAxis
            dataKey="day"
            interval={0}
            tickFormatter={(_day: string, index: number) => rows[index]?.label ?? ''}
            axisLine={false}
            tickLine={false}
            tick={{ fill: 'var(--gray-11)', fontSize: 11 }}
            height={24}
          />
          <Tooltip content={<ChartTooltip />} cursor={{ fill: 'var(--gray-a3)' }} isAnimationActive={false} />
          <Bar dataKey="downloads" isAnimationActive={false} shape={<DayBar />} />
        </BarChart>
      </ResponsiveContainer>
    </Box>
  );
}

/** The legend of the chart: which color stands for weekdays and which for weekends. */
export function DownloadsChartLegend(): JSX.Element {
  const strings = UIStrings.USAGE_DASHBOARD.CHART;
  return (
    <Flex gap="4" className="as-usage-chart__legend">
      <Flex align="center" gap="2">
        <span className="as-usage-chart__swatch" style={{ background: WEEKDAY_COLOR }} aria-hidden="true" />
        <Text size="1" color="gray">{strings.WEEKDAYS}</Text>
      </Flex>
      <Flex align="center" gap="2">
        <span className="as-usage-chart__swatch" style={{ background: WEEKEND_COLOR }} aria-hidden="true" />
        <Text size="1" color="gray">{strings.WEEKENDS}</Text>
      </Flex>
    </Flex>
  );
}

export default DownloadsChart;
