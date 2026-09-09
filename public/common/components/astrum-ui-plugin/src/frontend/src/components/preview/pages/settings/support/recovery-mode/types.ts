

/**
 * A reconcile task as returned by the recovery-mode UI endpoint (TaskXO subset).
 */
export interface ReconcileTask {
  id: string;
  /** Task display name, e.g. "Repair - Data Repair Plan". */
  name: string;
  /** Task type id, e.g. "blobstore.planReconciliation". */
  type: string;
  /** Current run state, e.g. "WAITING", "RUNNING", "RUNNING: 45%". */
  currentState?: string | null;
  /** Last run start time (ISO string), or null if never run. */
  lastRun?: string | null;
  /** Result of the last run, e.g. "OK", or null if never run. */
  lastRunResult?: string | null;
}

/**
 * Response shape of GET service/rest/internal/ui/recovery-mode.
 */
export interface RecoveryModeData {
  enabled: boolean;
  /** True when one or more data repair plans have not been executed. */
  unexecutedPlans: boolean;
  /**
   * Task type names blocked while recovery mode is active. Returned by the
   * endpoint but intentionally not surfaced in the page (parity with the
   * Classic UI, which also does not display it).
   */
  blockedTaskNames: string[];
  /** Data repair (reconcile) tasks shown in the table. */
  reconcileTasks: ReconcileTask[];
}
