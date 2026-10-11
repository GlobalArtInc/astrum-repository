import React, {useMemo} from 'react';

import './Sparkline.scss';
import SharedStrings from '../../constants/SharedStrings';

export interface DataPoint {
  date: string;
  value: number;
}

export interface SparklineProps {
  data: DataPoint[];
  width: number;
  height: number;
  color?: string;
  strokeWidth?: number;
  showTrend?: boolean;
}

type Trend = 'up' | 'down' | 'flat';

function trendOf(data: DataPoint[]): Trend {
  const delta = data[data.length - 1].value - data[0].value;
  if (delta > 0) {
    return 'up';
  }
  if (delta < 0) {
    return 'down';
  }
  return 'flat';
}

export function Sparkline({data, width, height, color = 'currentColor', strokeWidth = 1.5, showTrend = false}: SparklineProps) {
  const points = useMemo(() => {
    if (!data || data.length < 2) {
      return [];
    }
    const pad = strokeWidth + (showTrend ? 2 : 0);
    const values = data.map((point) => point.value);
    const min = Math.min(...values);
    const span = Math.max(...values) - min || 1;
    const stepX = (width - pad * 2) / (values.length - 1);
    return values.map((value, index) => ({
      x: pad + index * stepX,
      y: height - pad - ((value - min) / span) * (height - pad * 2)
    }));
  }, [data, width, height, strokeWidth, showTrend]);

  if (points.length < 2) {
    return null;
  }

  const trend = trendOf(data);
  const path = points.map((point, index) => `${index === 0 ? 'M' : 'L'}${point.x.toFixed(1)} ${point.y.toFixed(1)}`).join(' ');
  const last = points[points.length - 1];

  return (
    <svg
      className={`sparkline sparkline--${trend}`}
      width={width}
      height={height}
      viewBox={`0 0 ${width} ${height}`}
      role="img"
      aria-label={SharedStrings.SPARKLINE.TREND(trend)}
    >
      <path d={path} fill="none" stroke={color} strokeWidth={strokeWidth} strokeLinejoin="round" strokeLinecap="round" />
      {showTrend && <circle cx={last.x} cy={last.y} r={2} fill={color} />}
    </svg>
  );
}

export default Sparkline;
