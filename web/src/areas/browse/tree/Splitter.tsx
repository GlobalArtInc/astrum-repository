import { type KeyboardEvent, type PointerEvent, useRef } from 'react';
import styles from './Splitter.module.css';

export interface SplitterProps {
  value: number;
  min: number;
  max: number;
  defaultValue: number;
  label: string;
  controls: string;
  onChange(value: number): void;
}

export function clamp(value: number, min: number, max: number): number {
  return Math.round(Math.min(Math.max(value, min), Math.max(min, max)));
}

export function Splitter({ value, min, max, defaultValue, label, controls, onChange }: SplitterProps) {
  const start = useRef<{ x: number; width: number } | null>(null);

  const onPointerDown = (event: PointerEvent<HTMLDivElement>) => {
    event.preventDefault();
    event.currentTarget.setPointerCapture?.(event.pointerId);
    start.current = { x: event.clientX, width: value };
  };

  const onPointerMove = (event: PointerEvent<HTMLDivElement>) => {
    if (start.current) {
      onChange(clamp(start.current.width + event.clientX - start.current.x, min, max));
    }
  };

  const onPointerUp = (event: PointerEvent<HTMLDivElement>) => {
    start.current = null;
    event.currentTarget.releasePointerCapture?.(event.pointerId);
  };

  const onKeyDown = (event: KeyboardEvent<HTMLDivElement>) => {
    const step = event.shiftKey ? 64 : 16;
    const next =
      event.key === 'ArrowLeft'
        ? value - step
        : event.key === 'ArrowRight'
          ? value + step
          : event.key === 'Home'
            ? min
            : event.key === 'End'
              ? max
              : event.key === 'Enter'
                ? defaultValue
                : null;
    if (next !== null) {
      event.preventDefault();
      onChange(clamp(next, min, max));
    }
  };

  return (
    <div
      role="separator"
      aria-orientation="vertical"
      aria-label={label}
      aria-controls={controls}
      aria-valuenow={value}
      aria-valuemin={min}
      aria-valuemax={Math.max(min, max)}
      tabIndex={0}
      className={styles.splitter}
      onPointerDown={onPointerDown}
      onPointerMove={onPointerMove}
      onPointerUp={onPointerUp}
      onPointerCancel={onPointerUp}
      onDoubleClick={() => onChange(clamp(defaultValue, min, max))}
      onKeyDown={onKeyDown}
    >
      <span className={styles.grip} aria-hidden="true" />
    </div>
  );
}
