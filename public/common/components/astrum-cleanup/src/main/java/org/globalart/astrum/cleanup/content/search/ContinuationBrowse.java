
package org.globalart.astrum.cleanup.content.search;

import org.globalart.astrum.common.entity.Continuation;

@FunctionalInterface
public interface ContinuationBrowse<T>
{
  Continuation<T> browse(Integer limit, String continuationToken);
}
