
package org.globalart.astrum.repository.view.handlers;

import java.util.List;
import java.util.ListIterator;

import javax.annotation.Nonnull;
import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Handler;
import org.globalart.astrum.repository.view.Response;

import com.google.common.annotations.VisibleForTesting;
import org.springframework.context.annotation.Lazy;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.repository.view.Router.LOCAL_ATTRIBUTE_PREFIX;
import org.springframework.stereotype.Component;

/**
 * A {@link Handler} which diverts request processing through any {@link ContributedHandler}s that are active.
 *
 * @since 3.1
 */
@Component
public class HandlerContributor
    implements Handler
{
  private final List<ContributedHandler> contributedHandlers;

  @VisibleForTesting
  static final String EXTENDED_MARKER =
      LOCAL_ATTRIBUTE_PREFIX + HandlerContributor.class.getName() + ".extended";

  @Autowired
  public HandlerContributor(@Lazy final List<ContributedHandler> contributedHandlers) {
    this.contributedHandlers = checkNotNull(contributedHandlers);
  }

  @Nonnull
  @Override
  public Response handle(@Nonnull final Context context) throws Exception {
    // Ensure the extra handlers are inserted only once, in the case that a handler higher
    // on the stack calls proceed() twice for some reason
    if (!isMarkedExtended(context)) {
      ListIterator<ContributedHandler> handlerIterator = contributedHandlers.listIterator(contributedHandlers.size());
      while (handlerIterator.hasPrevious()) {
        context.insertHandler(handlerIterator.previous());
      }
      markExtended(context);
    }

    return context.proceed();
  }

  private boolean isMarkedExtended(final Context context) {
    return context.getAttributes().contains(EXTENDED_MARKER);
  }

  private void markExtended(final Context context) {
    context.getAttributes().set(EXTENDED_MARKER, Boolean.TRUE);
  }
}
