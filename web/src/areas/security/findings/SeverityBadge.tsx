import { Badge } from '@radix-ui/themes';
import { useT } from '../../../i18n';
import { type Severity, severityOf } from './model';

interface SeverityBadgeProps {
  severity: string;
}

const COLORS = {
  critical: 'red',
  high: 'orange',
  medium: 'amber',
  low: 'blue',
  unknown: 'gray',
} as const;

const LABELS = {
  critical: 'security.findings.severity.critical',
  high: 'security.findings.severity.high',
  medium: 'security.findings.severity.medium',
  low: 'security.findings.severity.low',
  unknown: 'security.findings.severity.unknown',
} as const;

export function SeverityBadge({ severity }: SeverityBadgeProps) {
  const t = useT();
  const known: Severity = severityOf(severity);
  return (
    <Badge color={COLORS[known]} variant="soft" radius="full">
      {t(LABELS[known])}
    </Badge>
  );
}
