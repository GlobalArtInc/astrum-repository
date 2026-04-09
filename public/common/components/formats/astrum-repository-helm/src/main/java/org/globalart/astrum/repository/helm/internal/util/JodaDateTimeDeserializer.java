
package org.globalart.astrum.repository.helm.internal.util;

import com.fasterxml.jackson.datatype.joda.cfg.FormatConfig;
import com.fasterxml.jackson.datatype.joda.deser.DateTimeDeserializer;
import org.joda.time.DateTime;

public class JodaDateTimeDeserializer
    extends DateTimeDeserializer
{
  public JodaDateTimeDeserializer() {
    // no arg constructor providing default values for super call
    super(DateTime.class, FormatConfig.DEFAULT_DATETIME_PARSER);
  }
}
