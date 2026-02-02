
package org.globalart.astrum.repository.view.matchers.token;

import java.util.Map;
import java.util.Optional;

import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Matcher;
import org.globalart.astrum.repository.view.Request;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * A {@link Matcher} that examines the {@link Request#getPath() request path} and attempts to parse it using the
 * {@link TokenParser}.
 *
 * If there is a match, the tokens are stored in the context under key {@link TokenMatcher.State}.
 *
 * @since 3.0
 */
public class TokenMatcher
    implements Matcher
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  public interface State
  {
    String pattern();

    Map<String, String> getTokens();
  }

  private final TokenParser parser;

  private final String pattern;

  public TokenMatcher(final String pattern) {
    this.pattern = checkNotNull(pattern);
    this.parser = new TokenParser(pattern);
  }

  @Override
  public boolean matches(final Context context) {
    checkNotNull(context);

    Optional<State> state = match(context.getRequest().getPath());

    if (state.isEmpty()) {
      return false;
    }

    // matched expose tokens in context
    context.getAttributes().set(State.class, state.get());
    return true;
  }

  public Optional<State> match(final String path) {
    log.debug("Matching: {}~={}", path, parser);
    final Map<String, String> tokens = parser.parse(path);
    if (tokens == null) {
      // There was no match.
      return Optional.empty();
    }

    // matched expose tokens in context
    return Optional.of(new State()
    {
      @Override
      public String pattern() {
        return pattern;
      }

      @Override
      public Map<String, String> getTokens() {
        return tokens;
      }
    });
  }

  @Override
  public String toString() {
    return "TokenMatcher [pattern=" + pattern + "]";
  }
}
