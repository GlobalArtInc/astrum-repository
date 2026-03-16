
package org.globalart.astrum.repository.search.sql.store;

/**
 * Data class to hold unique repositories in the SQL search tables
 */
public record SearchRepositoryData(
    String repositoryName,
    Integer repositoryId,
    String repositoryFormat)
{
}
