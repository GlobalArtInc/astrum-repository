
package org.globalart.astrum.common.db;

public interface DatabaseCheck
{
  public static final String POSTGRE_SQL = "PostgreSQL";

  boolean isPostgresql();

  /**
   * To be used during startup to determine if minimum schema versions >= currently running schema
   * 
   * @param annotatedClass class to check for <code>@AvailabilityVersion(from = "1.0")</code>
   * @return true if the class is allowed to start based on current database schema
   */
  boolean isAllowedByVersion(Class<?> annotatedClass);

  /**
   * To be used at runtime, to execute code paths according to the schema version
   * 
   * @param version the version to verify against
   * @return true if the database schema version is >= than the argument
   */
  boolean isAtLeast(final String version);
}
