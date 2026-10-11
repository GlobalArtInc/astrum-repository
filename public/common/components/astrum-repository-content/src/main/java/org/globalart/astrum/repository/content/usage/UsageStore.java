package org.globalart.astrum.repository.content.usage;

import java.time.OffsetDateTime;
import java.util.ArrayList;
import java.util.Collection;
import java.util.List;
import java.util.regex.Pattern;
import javax.annotation.Nullable;

import org.globalart.astrum.datastore.ConfigStoreSupport;
import org.globalart.astrum.datastore.api.DataSessionSupplier;
import org.globalart.astrum.transaction.Transactional;

import com.google.common.collect.Iterables;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkArgument;

/**
 * Reads what the content of the repositories of each format takes up, from the tables of the format.
 *
 * The format names the tables to read, so only names made of letters, digits and underscores are accepted.
 */
@Component
public class UsageStore
    extends ConfigStoreSupport<UsageDAO>
{
  static final int BATCH_SIZE = 1000;

  private static final Pattern FORMAT = Pattern.compile("[A-Za-z0-9_]+");

  @Autowired
  public UsageStore(final DataSessionSupplier sessionSupplier) {
    super(sessionSupplier, UsageDAO.class);
  }

  /**
   * Counts the components and assets of every content repository of a format and adds up their sizes.
   */
  @Transactional
  public List<RepositoryContentUsage> browseContentUsage(final String format, final OffsetDateTime unusedSince) {
    return dao().browseContentUsage(checkFormat(format), unusedSince);
  }

  /**
   * Counts the components of every content repository of a format.
   */
  @Transactional
  public List<RepositoryContentUsage> countComponents(final String format) {
    return dao().countComponents(checkFormat(format));
  }

  /**
   * Returns the ids of the components added last to the given content repositories of a format, or to all of them
   * when {@code repositoryIds} is {@code null}, last added first.
   */
  @Transactional
  public List<Integer> browseLatestComponentIds(
      final String format,
      @Nullable final Collection<Integer> repositoryIds,
      final int limit)
  {
    if (repositoryIds != null && repositoryIds.isEmpty()) {
      return List.of();
    }
    return dao().browseLatestComponentIds(checkFormat(format), repositoryIds, limit);
  }

  /**
   * Reads the given components of a format; components that no longer exist are left out.
   */
  @Transactional
  public List<ComponentSummary> browseComponents(final String format, final Collection<Integer> componentIds) {
    checkFormat(format);
    List<ComponentSummary> components = new ArrayList<>();
    for (List<Integer> batch : Iterables.partition(componentIds, BATCH_SIZE)) {
      components.addAll(dao().browseComponents(format, batch));
    }
    return components;
  }

  private static String checkFormat(final String format) {
    checkArgument(format != null && FORMAT.matcher(format).matches(), "Not a format: %s", format);
    return format;
  }
}
