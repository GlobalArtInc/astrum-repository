import { Badge } from '@radix-ui/themes';
import { DateTime } from '../../ui';
import { type ExpiryTone, expiryTone } from './model';

const COLORS: Record<ExpiryTone, 'amber' | 'red'> = {
  warning: 'amber',
  danger: 'red',
};

export interface ExpiryProps {
  at: string | null;
}

export function Expiry({ at }: ExpiryProps) {
  if (at === null) {
    return null;
  }
  const tone = expiryTone(at, Date.now());
  if (tone === undefined) {
    return <DateTime value={at} />;
  }
  return (
    <Badge color={COLORS[tone]} variant="soft">
      <DateTime value={at} />
    </Badge>
  );
}
