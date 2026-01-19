
package org.globalart.astrum.extdirect.internal;

import com.google.gson.GsonBuilder;
import com.softwarementors.extjs.djn.config.GlobalConfiguration;
import com.softwarementors.extjs.djn.gson.DefaultGsonBuilderConfigurator;

/**
 * Additional GSon type adapters.
 *
 * @since 3.0
 */
public class ExtDirectGsonBuilderConfigurator
    extends DefaultGsonBuilderConfigurator
{

  @Override
  public void configure(final GsonBuilder builder, final GlobalConfiguration configuration) {
    if (configuration.getDebug()) {
      builder.setPrettyPrinting();
    }
    builder.serializeNulls();
    builder.disableHtmlEscaping();

    builder.setDateFormat("yyyy-MM-dd'T'HH:mm:ss.SSSXXX");
    super.registerEncodingTypeAdapters(builder, configuration);
  }
}
