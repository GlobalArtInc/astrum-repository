

import React from 'react';
import { useMachine } from '@xstate/react';
import { Box, Flex, Text } from '@radix-ui/themes';
import { Loader2, AlertCircle, AlertTriangle } from 'lucide-react';
import { planInformationMachine } from './planInformationMachine';

const EMPTY = 'N/A';

/** Format a stored plan date for display; falls back to N/A when blank/unparseable. */
function formatLongDateTime(value: string | null): string {
  if (!value) return EMPTY;
  const d = new Date(value);
  if (Number.isNaN(d.getTime())) return EMPTY;
  // Intentional improvement over Classic (which used a fixed US-locale format): delegate to the
  // user's browser locale so dates render naturally in their local format.
  return d.toLocaleString(undefined, {
    year: 'numeric', month: 'long', day: 'numeric', hour: '2-digit', minute: '2-digit',
  });
}

interface Row {
  label: string;
  value: string;
}

/**
 * Read-only Plan Information panel (PlanInformationFormField parity). Fetches and aggregates
 * /v1/plan via planInformationMachine; renders 5 labelled values, N/A on empty, inline error on
 * failure. Display-only — carries no form value.
 */
export function PlanInformationWidget(): React.ReactElement {
  const [state] = useMachine(planInformationMachine);
  const { planCount, blobStoreCount, repositoryCount, startDate, endDate, error, truncated } = state.context;

  if (state.matches('loading')) {
    return (
      <Box className="dynamic-field plan-information plan-information--loading">
        <Flex align="center" gap="2">
          <Loader2 size={16} className="animate-spin" aria-hidden="true" />
          <Text size="2" color="gray">Loading plan information…</Text>
        </Flex>
      </Box>
    );
  }

  if (state.matches('error')) {
    return (
      <Box className="dynamic-field plan-information plan-information--error" role="alert">
        <Flex align="center" gap="2">
          <AlertCircle size={16} aria-hidden="true" />
          <Text size="2">{error ?? 'Failed to load plan information'}</Text>
        </Flex>
      </Box>
    );
  }

  const rows: Row[] = [
    { label: 'Plans', value: planCount > 0 ? String(planCount) : EMPTY },
    { label: 'Blob stores', value: blobStoreCount > 0 ? String(blobStoreCount) : EMPTY },
    { label: 'Repositories', value: repositoryCount > 0 ? String(repositoryCount) : EMPTY },
    { label: 'Start date', value: formatLongDateTime(startDate) },
    { label: 'End date', value: formatLongDateTime(endDate) },
  ];

  return (
    <Box className="dynamic-field plan-information">
      <Flex direction="column" gap="1">
        {rows.map((row) => (
          <Flex key={row.label} justify="between" gap="4">
            <Text size="2" weight="medium">{row.label}</Text>
            <Text size="2" color="gray" data-testid={`plan-info-${row.label.replace(/\s+/g, '-').toLowerCase()}`}>
              {row.value}
            </Text>
          </Flex>
        ))}
        {truncated && (
          <Flex align="center" gap="1" mt="1" role="alert" className="plan-information--truncated">
            <AlertTriangle size={14} aria-hidden="true" />
            <Text size="1" color="orange">Plan list exceeds maximum page limit — counts may be incomplete.</Text>
          </Flex>
        )}
      </Flex>
    </Box>
  );
}

export default PlanInformationWidget;
