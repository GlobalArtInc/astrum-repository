package org.globalart.astrum.repository.content.usage;

import java.time.OffsetDateTime;
import java.util.Collection;
import java.util.List;
import javax.annotation.Nullable;

import org.globalart.astrum.datastore.api.DataAccess;

import org.apache.ibatis.annotations.Param;

/**
 * Reads what the content of the repositories of a format takes up from the tables of that format; owns no tables.
 *
 * The format names the tables, so it must be the value of a registered format.
 */
public interface UsageDAO
    extends DataAccess
{
  /**
   * Counts the assets of every content repository of a format and adds up their sizes, and those of the unused and
   * never downloaded components.
   */
  List<RepositoryContentUsage> browseContentUsage(
      @Param("format") String format,
      @Param("unusedSince") OffsetDateTime unusedSince);

  /**
   * Counts the components of every content repository of a format.
   */
  List<RepositoryContentUsage> countComponents(@Param("format") String format);

  /**
   * Returns the ids of the components added last to the given content repositories of a format, or to all of them,
   * last added first.
   */
  List<Integer> browseLatestComponentIds(
      @Param("format") String format,
      @Nullable @Param("repositoryIds") Collection<Integer> repositoryIds,
      @Param("limit") int limit);

  /**
   * Reads the given components of a format; components that no longer exist are left out.
   */
  List<ComponentSummary> browseComponents(
      @Param("format") String format,
      @Param("componentIds") Collection<Integer> componentIds);
}
