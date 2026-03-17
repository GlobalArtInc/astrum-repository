
package org.globalart.astrum.content.raw.internal;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.content.director.ContentDirector;
import org.globalart.astrum.repository.content.fluent.FluentComponent;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * @since 3.27
 */
@Component
@Qualifier("raw")
public class RawContentDirector
    implements ContentDirector
{
  @Override
  public boolean allowMoveTo(final Repository destination) {
    return true;
  }

  @Override
  public boolean allowMoveTo(final FluentComponent component, final Repository destination) {
    return true;
  }

  @Override
  public boolean allowMoveFrom(final Repository source) {
    return true;
  }
}
