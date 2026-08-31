

import { useCallback, useEffect, useState } from 'react';
import { restClient, parseApiError } from '../../../../../../interface/api';
import type { TaskInfo, CapabilityInfo } from '../profile/types';

const TASKS_URL = '/service/rest/v1/tasks';
const CAPABILITIES_URL = '/service/rest/v1/capabilities';

interface RawTask {
  id: string;
  name: string;
  typeId: string;
  schedule?: string;
  lastRun?: string;
  lastRunResult?: string;
  nextRun?: string;
  currentState?: string;
  properties?: { repositoryName?: string };
}

interface UseRepositoryTasksCapabilitiesResult {
  tasks: TaskInfo[];
  capabilities: CapabilityInfo[];
  loading: boolean;
  error: string | null;
  refetch: () => void;
}

interface UseRepositoryTasksCapabilitiesOptions {
  /** When false, the tasks REST call is skipped. Defaults to true. */
  canReadTasks?: boolean;
  /** When false, the capabilities REST call is skipped. Defaults to true. */
  canReadCapabilities?: boolean;
}

/**
 * Hook for fetching repo-scoped tasks and capabilities for the Repository
 * Settings page. Endpoints the caller is allowed to read are fetched in
 * parallel; results are filtered to items explicitly targeting the given
 * repository. Capabilities without a `repository`/`repositoryName` property
 * are excluded (they are instance-wide and not meaningful in a repo-scoped
 * tab). Pass `canReadTasks`/`canReadCapabilities` to skip fetches the user
 * would receive 403s for.
 */
export function useRepositoryTasksCapabilities(
  repositoryName: string,
  { canReadTasks = true, canReadCapabilities = true }: UseRepositoryTasksCapabilitiesOptions = {}
): UseRepositoryTasksCapabilitiesResult {
  const [tasks, setTasks] = useState<TaskInfo[]>([]);
  const [capabilities, setCapabilities] = useState<CapabilityInfo[]>([]);
  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);
  const [refetchTrigger, setRefetchTrigger] = useState(0);

  const refetch = useCallback(() => {
    setRefetchTrigger((n) => n + 1);
  }, []);

  useEffect(() => {
    let cancelled = false;

    setLoading(true);
    setError(null);

    (async () => {
      try {
        const [tasksResponse, capabilitiesResponse] = await Promise.all([
          canReadTasks
            ? restClient.get<{ items?: RawTask[] }>(TASKS_URL)
            : Promise.resolve(undefined),
          canReadCapabilities
            ? // Typed as `unknown` because the REST endpoint isn't schema-guaranteed
              // to hand back an array in all deployments — the Array.isArray guard
              // below is the runtime contract.
              restClient.get<unknown>(CAPABILITIES_URL)
            : Promise.resolve(undefined),
        ]);

        if (cancelled) return;

        const allTasks = tasksResponse?.items ?? [];
        // NOTE: intentional rename. The REST API exposes the task classification
        // as `typeId`; the UI-facing `TaskInfo` type calls it `type` to match how
        // it's rendered in the table.
        const filteredTasks: TaskInfo[] = allTasks
          .filter((t) => t.properties?.repositoryName === repositoryName)
          .map((t) => ({
            id: t.id,
            name: t.name,
            type: t.typeId,
            schedule: t.schedule,
            lastRun: t.lastRun,
            lastRunResult: t.lastRunResult,
            nextRun: t.nextRun,
            currentState: t.currentState,
          }));

        const allCapabilities: CapabilityInfo[] = Array.isArray(capabilitiesResponse)
          ? (capabilitiesResponse as CapabilityInfo[])
          : [];
        const filteredCapabilities = allCapabilities.filter(
          (c) =>
            c.properties?.repository === repositoryName ||
            c.properties?.repositoryName === repositoryName
        );

        setTasks(filteredTasks);
        setCapabilities(filteredCapabilities);
      } catch (err) {
        if (cancelled) return;
        setError(parseApiError(err as Error).message);
        setTasks([]);
        setCapabilities([]);
      } finally {
        if (!cancelled) {
          setLoading(false);
        }
      }
    })();

    return () => {
      cancelled = true;
    };
  }, [repositoryName, refetchTrigger, canReadTasks, canReadCapabilities]);

  return { tasks, capabilities, loading, error, refetch };
}

export default useRepositoryTasksCapabilities;
