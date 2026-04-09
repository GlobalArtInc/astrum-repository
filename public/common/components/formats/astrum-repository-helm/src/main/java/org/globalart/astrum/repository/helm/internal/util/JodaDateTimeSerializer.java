
package org.globalart.astrum.repository.helm.internal.util;

import com.fasterxml.jackson.datatype.joda.cfg.FormatConfig;
import com.fasterxml.jackson.datatype.joda.ser.DateTimeSerializer;

public class JodaDateTimeSerializer
    extends DateTimeSerializer
{
  public JodaDateTimeSerializer() {
    // no arg constructor providing default values for super call
    super(FormatConfig.DEFAULT_DATETIME_PRINTER, 0);
  }
}
