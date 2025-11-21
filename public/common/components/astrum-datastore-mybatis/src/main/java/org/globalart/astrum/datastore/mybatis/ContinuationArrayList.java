
package org.globalart.astrum.datastore.mybatis;

import java.util.ArrayList;

import org.globalart.astrum.common.entity.Continuation;
import org.globalart.astrum.common.entity.ContinuationAware;

import static com.google.common.base.Preconditions.checkState;

/**
 * Collection of elements with a token that can be used to request the next set of results.
 *
 * @since 3.20
 */
public class ContinuationArrayList<E extends ContinuationAware>
    extends ArrayList<E>
    implements Continuation<E>
{
  private static final long serialVersionUID = -8278643802740770499L;

  @Override
  public String nextContinuationToken() {
    checkState(!isEmpty(), "No more results");
    return get(size() - 1).nextContinuationToken();
  }
}
