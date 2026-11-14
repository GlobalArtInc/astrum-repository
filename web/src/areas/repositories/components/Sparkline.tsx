import { sparklinePath } from '../model/sparkline';
import styles from './Sparkline.module.css';

const WIDTH = 96;
const HEIGHT = 24;

export function Sparkline({ values }: { values: readonly number[] | undefined }) {
  if (!values || values.length < 2) {
    return null;
  }
  return (
    <svg
      width={WIDTH}
      height={HEIGHT}
      viewBox={`0 0 ${WIDTH} ${HEIGHT}`}
      aria-hidden="true"
      className={styles.sparkline}
    >
      <path d={sparklinePath(values, WIDTH, HEIGHT)} className={styles.line} />
    </svg>
  );
}
