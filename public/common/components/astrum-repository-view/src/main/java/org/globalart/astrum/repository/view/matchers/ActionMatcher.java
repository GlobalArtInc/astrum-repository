
package org.globalart.astrum.repository.view.matchers;

import java.util.List;

import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Matcher;
import org.globalart.astrum.repository.view.Request;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkArgument;
import static com.google.common.base.Preconditions.checkNotNull;
import static java.util.Arrays.asList;

/**
 * Matches if the {@link Request#getAction() request action} is in the allowed list.
 *
 * @since 3.0
 */
public class ActionMatcher
    implements Matcher
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final List<String> allowedActions;

  public ActionMatcher(final String... allowedActions) {
    checkNotNull(allowedActions);
    checkArgument(allowedActions.length > 0, "at least one allowed action must be specified");
    this.allowedActions = asList(allowedActions);
  }

  @Override
  public boolean matches(final Context context) {
    final String action = context.getRequest().getAction();
    return allowedActions.contains(action);
  }

  @Override
  public String toString() {
    return "ActionMatcher [allowedActions=" + allowedActions + "]";
  }
}
