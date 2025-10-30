
package org.globalart.astrum.common.template;

import java.net.URL;

/**
 * Helper to render a template.
 *
 * @since 3.0
 */
public interface TemplateHelper
{
  /**
   * Returns default parameters.
   */
  TemplateParameters parameters();

  /**
   * Render template with given parameters.
   */
  String render(URL template, TemplateParameters parameters);
}
