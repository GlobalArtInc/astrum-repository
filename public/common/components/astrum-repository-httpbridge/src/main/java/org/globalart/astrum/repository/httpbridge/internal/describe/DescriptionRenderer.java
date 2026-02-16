
package org.globalart.astrum.repository.httpbridge.internal.describe;

/**
 * Renders {@link Description} into HTML or JSON.
 *
 * @since 3.0
 */
public interface DescriptionRenderer
{
  String renderHtml(Description description);

  String renderJson(Description description);
}
