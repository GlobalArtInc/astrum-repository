
package org.globalart.astrum.repository;

import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.common.db.DatabaseCheck;
import org.globalart.astrum.repository.recipe.RouterBuilder;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Support for {@link Recipe} implementations.
 *
 * @since 3.0
 */
public abstract class RecipeSupport
    implements Recipe
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final Format format;

  private final Type type;

  private BrowseUnsupportedHandler browseUnsupportedHandler;

  private HighAvailabilitySupportChecker highAvailabilitySupportChecker;

  private DatabaseCheck databaseCheck;

  protected RecipeSupport(final Type type, final Format format) {
    this.type = checkNotNull(type);
    this.format = checkNotNull(format);
  }

  @Override
  public Format getFormat() {
    return format;
  }

  @Override
  public Type getType() {
    return type;
  }

  @Override
  public String toString() {
    return getClass().getSimpleName() + "{" +
        "format=" + format +
        ", type=" + type +
        '}';
  }

  @Autowired
  public void setHighAvailabilitySupportChecker(final HighAvailabilitySupportChecker highAvailabilitySupportChecker) {
    this.highAvailabilitySupportChecker = highAvailabilitySupportChecker;
  }

  @Autowired
  public void setBrowseUnsupportedHandler(final BrowseUnsupportedHandler browseUnsupportedHandler) {
    this.browseUnsupportedHandler = checkNotNull(browseUnsupportedHandler);
  }

  @Autowired
  public void setDatabaseCheck(final DatabaseCheck databaseCheck) {
    this.databaseCheck = databaseCheck;
  }

  /**
   * Adds route to redirect access directly with a browser to a handler with links to the repo's components and
   * assets.
   */
  protected void addBrowseUnsupportedRoute(RouterBuilder builder) {
    builder.route(browseUnsupportedHandler.getRoute());
  }

  @Override
  public boolean isFeatureEnabled() {
    if (databaseCheck != null && !databaseCheck.isAllowedByVersion(getClass())) {
      return false;
    }
    if (highAvailabilitySupportChecker != null) {
      return highAvailabilitySupportChecker.isSupported(getFormat().getValue());
    }
    log.error("HighAvailabilitySupportChecker not found - Format {} will not be enabled", getFormat());
    return false;
  }
}
