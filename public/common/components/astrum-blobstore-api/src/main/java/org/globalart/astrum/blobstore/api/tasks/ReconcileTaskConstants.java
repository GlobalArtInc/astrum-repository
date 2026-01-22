
package org.globalart.astrum.blobstore.api.tasks;

/**
 * Shared constants for reconcile tasks
 */
public final class ReconcileTaskConstants
{
  public static final String TASK_SCOPE = "taskScope";

  public static final String TASK_SCOPE_DURATION = "duration";

  public static final String SINCE_DAYS = "sinceDays";

  public static final String SINCE_HOURS = "sinceHours";

  public static final String SINCE_MINUTES = "sinceMinutes";

  public static final String TASK_SCOPE_DATES = "dates";

  public static final String START_DATE = "reconcileStartDate";

  public static final String END_DATE = "reconcileEndDate";

  public static final String PLAN_RECONCILE_TYPE_ID = "blobstore.planReconciliation";

  public static final String EXECUTE_RECONCILE_PLAN_TYPE_ID = "blobstore.executeReconciliationPlan";

  public static final String UNABLE_TO_RUN_TASK_LOG_ERROR = "Unable to run task {}";

  public static final String ENTRY_ALL_BLOB_STORES = "(All Blob Stores)";

  public static final String BLOB_STORE_NAME_FIELD_ID = "blobstoreName";

  public static final String REPOSITORY_NAME_FIELD_ID = "repositoryName";

  public static final String PLAN_INFORMATION_FIELD_ID = "planInformation";

  public static final String ONLY_NOTIFY = "onlyNotify";

  private ReconcileTaskConstants() {
  }
}
