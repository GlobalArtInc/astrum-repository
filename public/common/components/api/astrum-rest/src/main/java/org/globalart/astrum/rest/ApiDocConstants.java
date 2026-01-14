
package org.globalart.astrum.rest;

/**
 * Constants for REST API documentation.
 *
 * @since 3.20
 */
public class ApiDocConstants
{
  private ApiDocConstants() {
  }

  public static final String SUCCESS = "Success";

  public static final String AUTHENTICATION_REQUIRED = "Authentication required";

  public static final String INSUFFICIENT_PERMISSIONS = "Insufficient permissions";

  public static final String REPOSITORY_NOT_FOUND = "Repository not found";

  public static final String PROJECT_NOT_FOUND = "Project not found";

  public static final String REPOSITORY_CREATED = "Repository created";

  public static final String REPOSITORY_UPDATED = "Repository updated";

  public static final String REPOSITORY_DELETED = "Repository deleted";

  public static final String API_REPOSITORY_MANAGEMENT = "Repository Management";

  public static final String API_BLOB_STORE = "Blob store";

  public static final String S3_BLOB_STORE_CREATED = "S3 blob store created";

  public static final String S3_BLOB_STORE_UPDATED = "S3 blob store updated";

  public static final String S3_BLOB_STORE_DELETED = "S3 blob store deleted";

  public static final String UNKNOWN_S3_BLOB_STORE = "Specified S3 blob store doesn't exist";

  public static final String BLOBSTORE_NOT_FOUND = "Blob store not found";

  public static final String BLOBSTORE_CHANGE_NOT_ALLOWED =
      "You cannot change a repository’s blob store through the API";

  public static final String BAD_REQUEST = "Bad request";

  public static final String DISABLED_IN_HIGH_AVAILABILITY = "Feature is disabled in High Availability";

  public static final String API_HOSTED_REPOSITORY_EVALUATION = "Hosted Repository Evaluation";

  public static final String MONITORING_ENABLED = "Monitoring enabled successfully";

  public static final String INVALID_REPOSITORY_IDS = "Invalid request (empty list, invalid UUID format)";
}
