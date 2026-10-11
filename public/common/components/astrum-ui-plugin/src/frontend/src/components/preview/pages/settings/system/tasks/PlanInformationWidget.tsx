

import React from 'react';
import { useMachine } from '@xstate/react';
import { Box, Flex, Text } from '@radix-ui/themes';
import { Loader2, AlertCircle, AlertTriangle } from 'lucide-react';
import { planInformationMachine } from './planInformationMachine';
import TasksStrings from '../../../../constants/pages/settings/system/TasksStrings';

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
          <Text size="2" color="gray">{TasksStrings.PLAN_INFORMATION.LOADING_PLAN_INFORMATION}</Text>
        </Flex>
      </Box>
    );
  }

  if (state.matches('error')) {
    return (
      <Box className="dynamic-field plan-information plan-information--error" role="alert">
        <Flex align="center" gap="2">
          <AlertCircle size={16} aria-hidden="true" />
          <Text size="2">{error ?? TasksStrings.PLAN_INFORMATION.FAILED_LOAD_PLAN_INFORMATION}</Text>
        </Flex>
      </Box>
    );
  }

  const rows: Row[] = [
    { label: TasksStrings.PLAN_INFORMATION.PLANS, value: planCount > 0 ? String(planCount) : EMPTY },
    { label: TasksStrings.PLAN_INFORMATION.BLOB_STORES, value: blobStoreCount > 0 ? String(blobStoreCount) : EMPTY },
    { label: TasksStrings.PLAN_INFORMATION.REPOSITORIES, value: repositoryCount > 0 ? String(repositoryCount) : EMPTY },
    { label: TasksStrings.PLAN_INFORMATION.START_DATE, value: formatLongDateTime(startDate) },
    { label: TasksStrings.PLAN_INFORMATION.END_DATE, value: formatLongDateTime(endDate) },
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
            <Text size="1" color="orange">{TasksStrings.PLAN_INFORMATION.PLAN_LIST_EXCEEDS_MAXIMUM_PAGE}</Text>
          </Flex>
        )}
      </Flex>
    </Box>
  );
}

export default PlanInformationWidget;
