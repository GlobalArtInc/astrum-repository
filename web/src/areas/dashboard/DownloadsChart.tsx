import { Text } from '@radix-ui/themes';
import { type PointerEvent, useLayoutEffect, useRef, useState } from 'react';
import { formatNumber, type Language, useI18n, useT } from '../../i18n';
import styles from './Dashboard.module.css';
import { axisMax, type DayDownloads, isWeekend, parseDay, peakOf } from './types';

export const CHART_HEIGHT = 260;
const LEFT = 48;
const RIGHT = 4;
const TOP = 28;
const BOTTOM = 24;
const TOOLTIP_WIDTH = 180;
const TOOLTIP_HEIGHT = 52;
const BAR_GAPS: Record<number, number> = { 7: 20, 30: 6, 90: 2 };

function barGap(count: number): number {
  return BAR_GAPS[count] ?? Math.max(2, Math.round(180 / Math.max(1, count)));
}

export function labelStep(count: number, width: number): number {
  const labels = width < 560 ? 3 : 6;
  return Math.max(1, Math.round(count / labels));
}

function bar(x: number, y: number, width: number, bottom: number): string {
  const radius = Math.min(3, width / 2, bottom - y);
  return [
    `M${x},${bottom}`,
    `L${x},${y + radius}`,
    `A${radius},${radius} 0 0 1 ${x + radius},${y}`,
    `L${x + width - radius},${y}`,
    `A${radius},${radius} 0 0 1 ${x + width},${y + radius}`,
    `L${x + width},${bottom}`,
    'Z',
  ].join(' ');
}

export function dayLabel(language: Language, day: string, count: number): string {
  const date = parseDay(day);
  if (count <= 7) {
    const parts = new Intl.DateTimeFormat(language, {
      weekday: 'short',
      day: 'numeric',
      timeZone: 'UTC',
    }).formatToParts(date);
    const weekday = parts.find((part) => part.type === 'weekday')?.value ?? '';
    const number = parts.find((part) => part.type === 'day')?.value ?? '';
    return `${weekday} ${number}`;
  }
  return shortDay(language, day);
}

export function shortDay(language: Language, day: string): string {
  return new Intl.DateTimeFormat(language, { month: 'short', day: 'numeric', timeZone: 'UTC' }).format(
    parseDay(day),
  );
}

function longDay(language: Language, day: string): string {
  return new Intl.DateTimeFormat(language, {
    weekday: 'long',
    month: 'long',
    day: 'numeric',
    timeZone: 'UTC',
  }).format(parseDay(day));
}

function useWidth() {
  const ref = useRef<HTMLDivElement>(null);
  const [width, setWidth] = useState(0);
  useLayoutEffect(() => {
    const element = ref.current;
    if (!element) {
      return;
    }
    setWidth(element.getBoundingClientRect().width);
    if (typeof ResizeObserver === 'undefined') {
      return;
    }
    const observer = new ResizeObserver((entries) => {
      const entry = entries[0];
      if (entry) {
        setWidth(entry.contentRect.width);
      }
    });
    observer.observe(element);
    return () => observer.disconnect();
  }, []);
  return [ref, width] as const;
}

interface Hover {
  index: number;
  x: number;
  y: number;
}

export function DownloadsChart({ days }: { days: DayDownloads[] }) {
  const t = useT();
  const { language } = useI18n();
  const [ref, measured] = useWidth();
  const [hover, setHover] = useState<Hover | null>(null);
  const width = measured > 0 ? measured : 600;
  const count = days.length;
  const plot = Math.max(1, width - LEFT - RIGHT);
  const bottom = CHART_HEIGHT - BOTTOM;
  const slot = plot / Math.max(1, count);
  const gap = Math.min(barGap(count), slot / 2);
  const barWidth = Math.max(1, Math.floor(slot - gap));
  const peak = peakOf(days);
  const max = axisMax(peak?.downloads ?? 0);
  const scale = (value: number) => bottom - (value / max) * (bottom - TOP);
  const ticks = [0, max / 2, max];
  const step = labelStep(count, width);

  const onMove = (event: PointerEvent<HTMLDivElement>) => {
    const box = event.currentTarget.getBoundingClientRect();
    const x = event.clientX - box.left;
    const y = event.clientY - box.top;
    const index = Math.floor((x - LEFT) / slot);
    if (x < LEFT || index < 0 || index >= count || y < TOP || y > bottom) {
      setHover(null);
      return;
    }
    setHover({ index, x, y });
  };

  const hovered = hover ? days[hover.index] : undefined;
  return (
    <div
      ref={ref}
      className={styles.chart}
      style={{ height: CHART_HEIGHT }}
      onPointerMove={onMove}
      onPointerLeave={() => setHover(null)}
    >
      <svg width={width} height={CHART_HEIGHT} role="img" aria-label={t('dashboard.chart.label')}>
        {ticks.map((tick) => (
          <line
            key={`grid-${tick}`}
            x1={LEFT}
            x2={width - RIGHT}
            y1={scale(tick)}
            y2={scale(tick)}
            className={styles.gridLine}
          />
        ))}
        {hover && (
          <rect
            x={LEFT + hover.index * slot}
            y={TOP + 0.5}
            width={slot}
            height={bottom - TOP - 1}
            className={styles.cursor}
          />
        )}
        {days.map((entry, index) => {
          if (entry.downloads <= 0) {
            return null;
          }
          const x = LEFT + index * slot + gap / 2;
          return (
            <path
              key={entry.day}
              d={bar(x, scale(entry.downloads), barWidth, bottom)}
              className={isWeekend(entry.day) ? styles.weekendBar : styles.weekdayBar}
              data-day={entry.day}
            />
          );
        })}
        {ticks.map((tick) => (
          <text
            key={`tick-${tick}`}
            x={LEFT - 8}
            y={scale(tick)}
            dy="0.355em"
            textAnchor="end"
            className={styles.axisText}
          >
            {formatNumber(language, tick)}
          </text>
        ))}
        {days.map((entry, index) => {
          if ((count - 1 - index) % step !== 0) {
            return null;
          }
          const center = LEFT + index * slot + slot / 2;
          const last = index === count - 1 && center + 18 > width;
          return (
            <text
              key={`label-${entry.day}`}
              x={last ? width : center}
              y={bottom + 8}
              dy="0.71em"
              textAnchor={last ? 'end' : 'middle'}
              className={styles.axisText}
            >
              {dayLabel(language, entry.day, count)}
            </text>
          );
        })}
        {peak &&
          (() => {
            const index = days.indexOf(peak);
            const center = LEFT + index * slot + slot / 2;
            const label = formatNumber(language, peak.downloads);
            const bubble = Math.round(label.length * 6.6 + 12);
            const top = scale(peak.downloads) - 24;
            return (
              <g className={styles.peak}>
                <rect x={center - bubble / 2} y={top} width={bubble} height={18} rx={6} />
                <text x={center} y={top + 13} textAnchor="middle">
                  {label}
                </text>
              </g>
            );
          })()}
      </svg>
      {hover && hovered && (
        <div
          className={styles.tooltip}
          style={{
            left: hover.x + 10 + TOOLTIP_WIDTH > width ? hover.x - 10 : hover.x + 10,
            top: hover.y + 10 + TOOLTIP_HEIGHT > CHART_HEIGHT ? hover.y - 10 : hover.y + 10,
            transform: `translate(${hover.x + 10 + TOOLTIP_WIDTH > width ? '-100%' : '0'}, ${
              hover.y + 10 + TOOLTIP_HEIGHT > CHART_HEIGHT ? '-100%' : '0'
            })`,
          }}
        >
          <Text as="div" size="1" color="gray">
            {longDay(language, hovered.day)}
          </Text>
          <Text as="div" size="2" weight="bold">
            {formatNumber(language, hovered.downloads)}
          </Text>
        </div>
      )}
    </div>
  );
}
