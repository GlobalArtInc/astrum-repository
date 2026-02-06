
package org.globalart.astrum.repository.content.store;

import java.util.Optional;
import java.util.function.Supplier;
import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.common.event.Event;
import org.globalart.astrum.common.event.EventManager;
import org.globalart.astrum.datastore.api.ContentDataAccess;
import org.globalart.astrum.datastore.api.DataSessionSupplier;
import org.globalart.astrum.datastore.api.SchemaTemplate;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.content.facet.ContentFacetFinder;

import static com.google.common.base.Preconditions.checkArgument;
import static com.google.common.base.Preconditions.checkNotNull;
import static java.util.Arrays.stream;
import static org.globalart.astrum.common.text.Strings2.lower;

/**
 * Support class for format content stores that send events.
 *
 * @since 3.27
 */
public abstract class ContentStoreEventSupport<T extends ContentDataAccess>
    extends ContentStoreSupport<T>
{
  protected final String format;

  protected EventManager eventManager;

  private ContentFacetFinder contentFacetFinder;

  protected ContentStoreEventSupport(
      final DataSessionSupplier sessionSupplier,
      final String contentStoreName,
      final Class<T> daoClass)
  {
    super(sessionSupplier, contentStoreName, daoClass);
    this.format = extractFormat(daoClass);
  }

  @Autowired
  protected void setDependencies(final ContentFacetFinder contentFacetFinder, final EventManager eventManager) {
    this.contentFacetFinder = checkNotNull(contentFacetFinder);
    this.eventManager = checkNotNull(eventManager);
  }

  public void preCommitEvent(final Supplier<Event> eventSupplier) {
    thisSession().preCommit(() -> postEvent(eventSupplier));
  }

  public void postCommitEvent(final Supplier<Event> eventSupplier) {
    thisSession().postCommit(() -> postEvent(eventSupplier));
  }

  private void postEvent(final Supplier<Event> eventSupplier) {
    Event event = eventSupplier.get();
    if (event instanceof ContentStoreEvent) {
      ContentStoreEvent cse = (ContentStoreEvent) event;
      cse.setRepositorySupplier(repositorySupplierFor(cse));
    }
    eventManager.post(event);
  }

  private Supplier<Optional<Repository>> repositorySupplierFor(final ContentStoreEvent event) {
    return () -> contentFacetFinder.findRepository(format, event.contentRepositoryId);
  }

  /**
   * Returns the lower-case form of the format after removing baseName from daoName.
   */
  private String extractFormat(final Class<T> daoClass) {
    // assume only one level between format and base DAO
    String formatDao = daoClass.getSimpleName();
    String dao = stream(daoClass.getInterfaces())
        .filter(c -> c.isAnnotationPresent(SchemaTemplate.class))
        .map(Class::getSimpleName)
        .findFirst()
        .orElseThrow(() -> new IllegalArgumentException("Cannot determine format"));

    String prefix = lower(formatDao.substring(0, formatDao.length() - dao.length()));
    checkArgument(!prefix.isEmpty(), "%s must add a prefix to %s", formatDao, dao);
    return prefix;
  }
}
