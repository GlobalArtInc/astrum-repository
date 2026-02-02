
package org.globalart.astrum.repository.view;

import java.util.List;

import javax.annotation.Nonnull;

import com.google.common.collect.Lists;

import static com.google.common.base.Preconditions.checkArgument;
import static com.google.common.base.Preconditions.checkNotNull;
import static com.google.common.base.Preconditions.checkState;

/**
 * View route.
 *
 * @since 3.0
 */
public class Route
    implements org.globalart.astrum.repository.recipe.Route
{
  private final Matcher matcher;

  private final List<Handler> handlers;

  public Route(final Matcher matcher, final List<Handler> handlers) {
    this.matcher = checkNotNull(matcher, "Missing matcher");
    checkNotNull(handlers, "Missing handlers");
    checkArgument(!handlers.isEmpty(), "At least one handler is required");
    this.handlers = handlers;
  }

  @Nonnull
  public Matcher getMatcher() {
    return matcher;
  }

  @Nonnull
  public List<Handler> getHandlers() {
    return handlers;
  }

  @Override
  public String toString() {
    return getClass().getSimpleName() + "{" +
        "matcher=" + matcher +
        ", handlers=" + handlers +
        '}';
  }

  //
  // Builder
  //

  /**
   * View {@link Route} builder.
   */
  public static class Builder
  {
    private Matcher matcher;

    private List<Handler> handlers = Lists.newArrayList();

    public Builder matcher(final Matcher matcher) {
      checkState(this.matcher == null, "Only one matcher allowed");
      this.matcher = checkNotNull(matcher);
      return this;
    }

    public Builder handler(final Handler handler) {
      checkNotNull(handler);
      return addHandler(handler);
    }

    // maps the new bridge interface from repository-config onto the view API
    public Builder handler(final org.globalart.astrum.repository.recipe.Handler handler) {
      checkNotNull(handler);
      if (handler instanceof Handler) {
        return addHandler((Handler) handler);
      }
      else {
        // partial mock that can be ignored
        return this;
      }
    }

    private Builder addHandler(final Handler handler) {
      handlers.add(handler);
      return this;
    }

    public Route create() {
      return new Route(matcher, handlers);
    }
  }
}
